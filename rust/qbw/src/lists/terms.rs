//! Payment terms.

use chrono::{Datelike, Months, NaiveDate, TimeDelta};
use sa17::{Decimal, Source};

use crate::codes::TermsKind;
use crate::error::Result;
use crate::ids::TermsId;
use crate::list::{List, ListEntry, sealed};
use crate::lists::reading;
use crate::origin::Origin;
use crate::tables::lists::TERMS;

/// When an invoice or a bill is to be paid, and what paying early saves.
///
/// The figures of both kinds are kept, and [`Self::kind`] says which count.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Terms {
    /// What the terms are known by.
    pub id: TermsId,
    /// Their name.
    pub name: String,
    /// Whether they count days, or go by a day of the month.
    pub kind: TermsKind,
    /// In how many days they are due, for standard terms.
    pub due_days: i32,
    /// For how many days there is a discount, for standard terms.
    pub discount_days: i32,
    /// The discount in percent.
    pub discount_percent: Option<Decimal>,
    /// The day of the month they are due on, for date-driven terms.
    pub due_day: i32,
    /// How many days before that day a form has to be dated, for date-driven
    /// terms. One dated later is due the month after.
    pub next_month_days: i32,
    /// The day of the month up to which there is a discount, for date-driven
    /// terms.
    pub discount_day: i32,
    /// Whether they can be used in new transactions.
    pub is_active: bool,
    /// The stored row.
    pub origin: Origin,
}

impl Terms {
    /// The day that a form dated `from` is due, which a user may change.
    /// `None` when the calendar has no such day, and for a kind of terms that
    /// this version does not know.
    ///
    /// For date-driven terms this is inferred from what QuickBooks asks for
    /// when they are set up. Two companies have forms with such terms. Of those
    /// dated just as many days before the due day as the terms ask for, four are
    /// due that day and none a month later.
    #[must_use]
    pub fn due_date(&self, from: NaiveDate) -> Option<NaiveDate> {
        match self.kind {
            TermsKind::Standard => {
                let days = TimeDelta::try_days(self.due_days.into())?;
                from.checked_add_signed(days)
            }
            TermsKind::DateDriven => self.date_driven(from),
            TermsKind::Other(_) => None,
        }
    }

    /// The next due day after `from`, or the one after it when that is too
    /// soon.
    fn date_driven(&self, from: NaiveDate) -> Option<NaiveDate> {
        let mut due = on_day(from, self.due_day)?;
        if due < from {
            due = on_day(next_month(from)?, self.due_day)?;
        }
        let notice = due.signed_duration_since(from).num_days();
        if notice < self.next_month_days.into() {
            due = on_day(next_month(due)?, self.due_day)?;
        }
        Some(due)
    }
}

impl sealed::Sealed for Terms {}

impl ListEntry for Terms {
    type Id = TermsId;

    fn id(&self) -> TermsId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn origin(&self) -> Origin {
        self.origin
    }
}

/// The day `day` of the month that `date` is in, or the last one of a month
/// that is shorter.
fn on_day(date: NaiveDate, day: i32) -> Option<NaiveDate> {
    let last = next_month(date)?.pred_opt()?;
    let day = u32::try_from(day).ok()?;
    last.with_day(day.min(last.day()))
}

/// The first day of the month after the one that `date` is in.
fn next_month(date: NaiveDate) -> Option<NaiveDate> {
    date.with_day(1)?.checked_add_months(Months::new(1))
}

/// Reads the payment terms.
///
/// # Errors
///
/// Returns an error when their table lacks a column.
pub(crate) fn read(source: &dyn Source) -> Result<List<Terms>> {
    reading::by_name(source, &TERMS, |origin, row| {
        (!row.is_deleted).then(|| Terms {
            id: row.id,
            name: row.name.unwrap_or_default(),
            kind: row.kind.unwrap_or_default(),
            due_days: row.due_days.unwrap_or_default(),
            discount_days: row.discount_days.unwrap_or_default(),
            discount_percent: row.discount_percent,
            due_day: row.due_day.unwrap_or_default(),
            next_month_days: row.next_month_days.unwrap_or_default(),
            discount_day: row.discount_day.unwrap_or_default(),
            is_active: !row.is_hidden,
            origin,
        })
    })
}

#[cfg(test)]
mod tests {
    use sa17::RowId;

    use super::*;

    fn terms(kind: TermsKind, due_days: i32, due_day: i32, next_month_days: i32) -> Terms {
        Terms {
            id: TermsId::new(1),
            name: "Terms".to_owned(),
            kind,
            due_days,
            discount_days: 0,
            discount_percent: None,
            due_day,
            next_month_days,
            discount_day: 0,
            is_active: true,
            origin: Origin::new("terms", RowId::Added(1)),
        }
    }

    fn due(terms: &Terms, from: &str) -> Option<String> {
        let due = terms.due_date(from.parse().unwrap());
        due.map(|due| due.to_string())
    }

    fn day(text: &str) -> Option<String> {
        Some(text.to_owned())
    }

    #[test]
    fn standard_terms_count_days() {
        let terms_of = |days| terms(TermsKind::Standard, days, 0, 0);
        assert_eq!(due(&terms_of(30), "2024-01-31"), day("2024-03-01"));
        assert_eq!(due(&terms_of(30), "2023-01-31"), day("2023-03-02"));
        assert_eq!(due(&terms_of(0), "2024-02-29"), day("2024-02-29"));
        assert_eq!(due(&terms_of(60), "2023-12-31"), day("2024-02-29"));
        let last = NaiveDate::MAX;
        assert_eq!(terms_of(1).due_date(last), None);
    }

    #[test]
    fn date_driven_terms_are_due_on_a_day_of_the_month() {
        let fifth = terms(TermsKind::DateDriven, 0, 5, 10);
        assert_eq!(due(&fifth, "2024-01-20"), day("2024-02-05"));
        // Ten days before is in time, and nine are not.
        assert_eq!(due(&fifth, "2024-01-26"), day("2024-02-05"));
        assert_eq!(due(&fifth, "2024-01-27"), day("2024-03-05"));
        assert_eq!(due(&fifth, "2024-01-05"), day("2024-02-05"));
        assert_eq!(due(&fifth, "2023-12-30"), day("2024-02-05"));
    }

    #[test]
    fn a_month_without_the_day_is_due_on_its_last() {
        let last = terms(TermsKind::DateDriven, 0, 31, 5);
        assert_eq!(due(&last, "2024-02-10"), day("2024-02-29"));
        assert_eq!(due(&last, "2023-02-25"), day("2023-03-31"));
        assert_eq!(due(&last, "2024-03-31"), day("2024-04-30"));
        assert_eq!(due(&last, "2024-12-31"), day("2025-01-31"));
    }

    #[test]
    fn a_day_that_no_month_has_is_no_due_date() {
        let nothing = terms(TermsKind::DateDriven, 0, 0, 0);
        assert_eq!(due(&nothing, "2024-01-20"), None);
        let before = terms(TermsKind::DateDriven, 0, -1, 0);
        assert_eq!(due(&before, "2024-01-20"), None);
        let unknown = terms(TermsKind::Other(7), 30, 5, 0);
        assert_eq!(due(&unknown, "2024-01-20"), None);
        assert_eq!(
            terms(TermsKind::DateDriven, 0, 5, 0).due_date(NaiveDate::MAX),
            None
        );
    }
}

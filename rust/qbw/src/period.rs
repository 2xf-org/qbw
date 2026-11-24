//! Spans of days.

use std::iter;
use std::ops::RangeInclusive;

use chrono::{Datelike, Month, NaiveDate};

/// A span of days, both ends included. A period without an end is open on
/// that side.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Period {
    first: Option<NaiveDate>,
    last: Option<NaiveDate>,
}

impl Period {
    /// Every day there is.
    #[must_use]
    pub const fn all() -> Self {
        Self {
            first: None,
            last: None,
        }
    }

    /// The days from `first` to `last`. It has no day when `last` comes first.
    #[must_use]
    pub const fn between(first: NaiveDate, last: NaiveDate) -> Self {
        Self {
            first: Some(first),
            last: Some(last),
        }
    }

    /// Every day from `first` on.
    #[must_use]
    pub const fn starting(first: NaiveDate) -> Self {
        Self {
            first: Some(first),
            last: None,
        }
    }

    /// Every day up to `last`.
    #[must_use]
    pub const fn until(last: NaiveDate) -> Self {
        Self {
            first: None,
            last: Some(last),
        }
    }

    /// The calendar month that `date` is in.
    #[must_use]
    pub fn month_of(date: NaiveDate) -> Self {
        Self::from_month(date.year(), date.month(), 1)
    }

    /// The quarter that `date` is in, for a company whose years start with the
    /// month `start`.
    #[must_use]
    pub fn quarter_of(date: NaiveDate, start: Month) -> Self {
        let start = start.number_from_month();
        let since_start = (date.month0() + 13 - start) % 12;
        let into_quarter = since_start % 3;
        if date.month() > into_quarter {
            Self::from_month(date.year(), date.month() - into_quarter, 3)
        } else {
            Self::from_month(date.year() - 1, date.month() + 12 - into_quarter, 3)
        }
    }

    /// The year that `date` is in, for a company whose years start with the
    /// month `start`.
    #[must_use]
    pub fn year_of(date: NaiveDate, start: Month) -> Self {
        let start = start.number_from_month();
        let year = if date.month() < start {
            date.year() - 1
        } else {
            date.year()
        };
        Self::from_month(year, start, 12)
    }

    /// So many months from the first day of one on. It is open where the calendar
    /// ends.
    fn from_month(year: i32, month: u32, months: u32) -> Self {
        let first = NaiveDate::from_ymd_opt(year, month, 1);
        // The year's end can be in the calendar where its start is not.
        let (after_year, after_month) = match month + months {
            after @ ..=12 => (year, after),
            after => (year + 1, after - 12),
        };
        let after = NaiveDate::from_ymd_opt(after_year, after_month, 1);
        Self {
            first,
            last: after.and_then(|after| after.pred_opt()),
        }
    }

    /// The calendar months of the period, one after the other. The first and the
    /// last are cut to it. None of a period that is open on a side.
    ///
    /// ```
    /// use qbw::Period;
    /// use qbw::chrono::{Month, NaiveDate};
    ///
    /// let date: NaiveDate = "2024-05-17".parse()?;
    /// let year = Period::year_of(date, Month::January);
    /// assert_eq!(year.months().count(), 12);
    /// assert_eq!(year.months().nth(4), Some(Period::month_of(date)));
    /// # Ok::<(), qbw::chrono::ParseError>(())
    /// ```
    pub fn months(self) -> impl Iterator<Item = Self> {
        let ends = self.first.zip(self.last);
        let mut next = ends.map(|(first, _)| first);
        iter::from_fn(move || {
            let (_, last) = ends?;
            let first = next.filter(|&first| first <= last)?;
            let end = Self::month_of(first).last.filter(|&end| end < last);
            next = end.and_then(|end| end.succ_opt());
            Some(Self::between(first, end.unwrap_or(last)))
        })
    }

    /// The first day, unless the period is open there.
    #[must_use]
    pub const fn first(self) -> Option<NaiveDate> {
        self.first
    }

    /// The last day, unless the period is open there.
    #[must_use]
    pub const fn last(self) -> Option<NaiveDate> {
        self.last
    }

    /// Whether `date` is one of the days.
    #[must_use]
    pub fn contains(self, date: NaiveDate) -> bool {
        self.first.is_none_or(|first| first <= date) && self.last.is_none_or(|last| date <= last)
    }
}

impl From<RangeInclusive<NaiveDate>> for Period {
    fn from(days: RangeInclusive<NaiveDate>) -> Self {
        let (first, last) = days.into_inner();
        Self::between(first, last)
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use super::*;

    fn day(text: &str) -> NaiveDate {
        text.parse().unwrap()
    }

    #[test]
    fn both_ends_are_included() {
        let period = Period::from(day("2024-01-01")..=day("2024-01-31"));
        assert!(period.contains(day("2024-01-01")) && period.contains(day("2024-01-31")));
        assert!(!period.contains(day("2023-12-31")) && !period.contains(day("2024-02-01")));
        assert!(Period::all().contains(day("1600-01-01")));
        assert!(Period::starting(day("2024-01-01")).contains(day("2999-12-31")));
        assert!(!Period::until(day("2024-01-01")).contains(day("2024-01-02")));
        let backwards = Period::between(day("2024-01-31"), day("2024-01-01"));
        assert!(!backwards.contains(day("2024-01-15")));
    }

    #[test]
    fn a_year_starts_with_the_company_s_first_month() {
        let calendar = Period::year_of(day("2024-02-29"), Month::January);
        assert_eq!(
            calendar,
            Period::between(day("2024-01-01"), day("2024-12-31"))
        );
        let before = Period::year_of(day("2024-03-31"), Month::April);
        assert_eq!(
            before,
            Period::between(day("2023-04-01"), day("2024-03-31"))
        );
        let after = Period::year_of(day("2024-04-01"), Month::April);
        assert_eq!(after, Period::between(day("2024-04-01"), day("2025-03-31")));
    }

    #[test]
    fn a_year_at_the_end_of_the_calendar_is_open() {
        let last = Period::year_of(NaiveDate::MAX, Month::January);
        assert_eq!(last.last(), None);
        assert!(last.contains(NaiveDate::MAX));
        assert!(Period::year_of(NaiveDate::MIN, Month::December).contains(NaiveDate::MIN));
    }

    #[test]
    fn a_quarter_is_three_months_of_a_year() {
        let quarter = |date, start| Period::quarter_of(day(date), start);
        let first = Period::between(day("2024-01-01"), day("2024-03-31"));
        assert_eq!(quarter("2024-03-31", Month::January), first);
        let last = Period::between(day("2023-11-01"), day("2024-01-31"));
        assert_eq!(quarter("2024-01-15", Month::February), last);
        assert_eq!(quarter("2023-12-31", Month::February), last);
        let next = Period::between(day("2024-02-01"), day("2024-04-30"));
        assert_eq!(quarter("2024-02-01", Month::February), next);
        // Every day is in the quarter that it is said to be in.
        for days in 0..800 {
            let date = day("2023-01-01") + TimeDelta::days(days);
            for start in [Month::January, Month::May, Month::December] {
                let quarter = Period::quarter_of(date, start);
                let year = Period::year_of(date, start);
                assert!(quarter.contains(date), "{date}");
                assert!(year.first() <= quarter.first() && quarter.last() <= year.last());
            }
        }
    }

    #[test]
    fn a_period_is_gone_through_month_by_month() {
        let months = |first, last| -> Vec<Period> {
            Period::between(day(first), day(last)).months().collect()
        };
        assert_eq!(
            months("2024-01-15", "2024-03-01"),
            [
                Period::between(day("2024-01-15"), day("2024-01-31")),
                Period::between(day("2024-02-01"), day("2024-02-29")),
                Period::between(day("2024-03-01"), day("2024-03-01")),
            ]
        );
        assert_eq!(months("2024-01-31", "2024-01-31").len(), 1);
        assert_eq!(months("2024-02-01", "2024-01-31"), []);
        assert_eq!(Period::until(day("2024-01-31")).months().count(), 0);
        let end = Period::between(day("2024-01-01"), NaiveDate::MAX);
        assert!(end.months().count() > 3_000_000);
    }
}

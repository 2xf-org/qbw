//! What is left of something that is owed, and when it was due.

use chrono::NaiveDate;

use crate::amount::{Amount, Total};
use crate::codes::TransactionKind;

/// How much is left of a line on an account for what customers owe or what
/// vendors are owed, as the file keeps it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Rest {
    /// Nothing: it is paid, or used up.
    Nothing,
    /// So much, with the sign of the line.
    Part(Amount),
    /// All of it.
    All,
}

impl Rest {
    /// What is left of a line, by its flag and by the balance that the file keeps.
    pub(crate) const fn of(is_paid: bool, open_balance: Option<Amount>) -> Self {
        match (is_paid, open_balance) {
            (true, _) => Self::Nothing,
            (false, Some(balance)) => Self::Part(balance),
            (false, None) => Self::All,
        }
    }

    /// What is left of a line of so much.
    pub(crate) fn of_line(self, whole: Amount) -> Amount {
        match self {
            Self::Nothing => Amount::default(),
            Self::Part(amount) => amount,
            Self::All => whole,
        }
    }
}

/// When a line of a transaction of `kind` is due, which keeps the day `stored`.
/// Some checks keep a day in the same place, and what it means is not known.
pub(crate) const fn due_date(
    kind: TransactionKind,
    stored: Option<NaiveDate>,
) -> Option<NaiveDate> {
    match kind {
        TransactionKind::Check | TransactionKind::SalesTaxPayment => None,
        _ => stored,
    }
}

/// Whether something is overdue on the day `date`: it is owed, to the company or
/// by it, and was due before. What takes off what is owed, such as a credit, is
/// below zero and never overdue, and neither is anything that has no day.
pub(crate) fn is_overdue(owed: Total, due: Option<NaiveDate>, date: NaiveDate) -> bool {
    let is_owed = !owed.is_zero() && !owed.is_negative();
    is_owed && due.is_some_and(|due| due < date)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn amount(text: &str) -> Amount {
        Amount::new(text.parse().unwrap()).unwrap()
    }

    #[test]
    fn what_is_left_is_all_a_part_or_nothing() {
        let whole = amount("202");
        let left = |is_paid, open| Rest::of(is_paid, open).of_line(whole);
        assert_eq!(left(false, None), whole);
        assert_eq!(left(false, Some(amount("45"))), amount("45"));
        assert_eq!(left(true, None), Amount::default());
        // The file keeps the last balance of some lines that are paid.
        assert_eq!(left(true, Some(amount("45"))), Amount::default());
    }

    #[test]
    fn something_is_overdue_the_day_after_it_was_due() {
        let day = |text: &str| text.parse::<NaiveDate>().unwrap();
        let due = Some(day("2024-01-31"));
        let late = |owed: &str, date| is_overdue(amount(owed).into(), due, day(date));
        assert!(!late("10", "2024-01-31"));
        assert!(late("10", "2024-02-01"));
        assert!(!late("-10", "2024-02-01"));
        assert!(!late("0", "2024-02-01"));
        assert!(!is_overdue(amount("10").into(), None, day("2024-02-01")));
    }

    #[test]
    fn a_check_is_never_due() {
        let day = "2024-01-31".parse().ok();
        assert_eq!(due_date(TransactionKind::Bill, day), day);
        assert_eq!(due_date(TransactionKind::Check, day), None);
    }
}

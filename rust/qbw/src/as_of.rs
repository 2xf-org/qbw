//! The moment of a balance.

use chrono::NaiveDate;

use crate::period::Period;

/// The moment of a balance, or of what is open. Everything that is of a moment
/// takes one, or a date, which is the end of that day.
///
/// ```
/// use qbw::chrono::NaiveDate;
/// use qbw::{AccountId, AsOf, Company};
///
/// # let file = qbw_synthetic::company::books().build();
/// let company = Company::from_bytes(file)?;
/// let (ledger, checking) = (company.ledger()?, AccountId::new(1));
/// let day: NaiveDate = "2023-12-31".parse()?;
/// let then = ledger.balance(checking, day).ok_or("no such account")?;
/// let now = ledger.balance(checking, AsOf::Latest).ok_or("no such account")?;
/// assert_eq!((then.amount.to_string(), now.amount.to_string()), ("800.00".to_owned(), "837.00".to_owned()));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AsOf {
    /// Now: everything in the file, and what is dated in the future, too. What
    /// is open is then what the file keeps as open. Where a day is needed, such
    /// as for what is overdue, it is the last day that something is posted on.
    Latest,
    /// The end of a day: everything dated up to it.
    Date(NaiveDate),
}

impl AsOf {
    /// The time up to the moment.
    pub(crate) const fn period(self) -> Period {
        match self {
            Self::Latest => Period::all(),
            Self::Date(date) => Period::until(date),
        }
    }
}

impl From<NaiveDate> for AsOf {
    fn from(date: NaiveDate) -> Self {
        Self::Date(date)
    }
}

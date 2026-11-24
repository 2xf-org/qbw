//! A reconciliation that is not finished.

use chrono::NaiveDate;

use crate::money::Money;

/// A reconciliation of an account that was started and is not finished.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Reconciling {
    /// The date of the statement.
    pub statement_date: Option<NaiveDate>,
    /// The balance that the statement ends with, in the account's currency. A
    /// debit is positive, as in the books: what is owed on a card is below zero,
    /// though its statement prints it above. The files of three companies have two
    /// such accounts that are being reconciled.
    /// [`AccountType::as_kept`](crate::accounts::AccountType::as_kept) turns it.
    pub statement_balance: Money,
}

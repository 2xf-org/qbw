//! What is left of a transaction.

use chrono::NaiveDate;

use crate::amount::Total;
use crate::codes::TransactionKind;
use crate::ids::{AccountId, NameId, TransactionId};
use crate::money::Money;
use crate::origin::Origin;
use crate::rest;

/// A transaction that is not settled: an invoice or a bill that is not paid in
/// full, a payment or a credit that is not used up, or a journal entry on the
/// account of one.
///
/// Amounts are above zero for what is owed, to the company or by it, and below
/// zero for what takes off that, such as a credit.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OpenItem {
    /// The transaction.
    pub transaction: TransactionId,
    /// Its kind.
    pub kind: TransactionKind,
    /// Its date.
    pub date: NaiveDate,
    /// Its number, as the form shows it.
    pub number: Option<String>,
    /// The customer or the vendor.
    pub name: Option<NameId>,
    /// The account.
    pub account: AccountId,
    /// When it is due. Nothing on a payment, a credit of a vendor and a journal
    /// entry.
    pub due_date: Option<NaiveDate>,
    /// All of it, in the account's currency.
    pub total: Money,
    /// What is left of it, in the account's currency.
    pub open: Money,
    /// What is left of it, in the company's currency: all of it, less what was
    /// settled at the rate of the transaction, to the cent.
    pub home_open: Money,
    /// The stored row of its line.
    pub origin: Origin,
    /// What is left of it, to add it to others.
    pub(super) sums: OpenSums,
}

/// What is left of an item as sums.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct OpenSums {
    pub(crate) open: Total,
    pub(crate) home_open: Total,
}

impl OpenItem {
    /// How many days after it was due the day `date` is. Below zero before it
    /// is due, and `None` where nothing says when it is.
    #[must_use]
    pub fn days_overdue(&self, date: NaiveDate) -> Option<i64> {
        let due = self.due_date?;
        Some(date.signed_duration_since(due).num_days())
    }

    /// Whether it is owed and was due before the day `date`. A credit is never
    /// overdue.
    #[must_use]
    pub fn is_overdue(&self, date: NaiveDate) -> bool {
        rest::is_overdue(self.sums.open, self.due_date, date)
    }

    /// What is left of it, to add it to others.
    pub(crate) fn sums(&self) -> OpenSums {
        self.sums
    }
}

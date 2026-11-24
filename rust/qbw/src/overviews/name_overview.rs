//! A customer, a vendor, an employee or another name.

use crate::as_of::AsOf;
use crate::left_out::LeftOut;
use crate::money::Money;
use crate::names::Name;
use crate::open_items::Owed;
use crate::query::TransactionPage;
use crate::reports::{NameBalanceRow, OwedBalance};

/// What a person sees on selecting a customer or a job, a vendor, an employee or
/// another name.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct NameOverview<'company> {
    /// The record.
    pub record: Name<'company>,
    /// The moment.
    pub as_of: AsOf,
    /// What a customer owes without its jobs, or what a vendor is owed with the
    /// sales tax among it. `None` for an employee and another name, which have
    /// no balance.
    pub balance: Option<OwedBalance>,
    /// The balance with those of the jobs, in the company's currency.
    pub total: Option<Money>,
    /// What is open and what of it is overdue, with the jobs'. `None` where
    /// [`Self::balance`] is.
    pub owed: Option<Owed>,
    /// The latest transactions that are with the name or with its jobs, the
    /// newest first. The names of their lines do not count.
    pub transactions: TransactionPage,
    /// How much could not be read that the balance may lack.
    pub left_out: LeftOut,
}

impl<'company> NameOverview<'company> {
    /// What there is about a name at `as_of`. The balance and what is owed are
    /// there for a customer and a vendor.
    pub(crate) fn new(
        record: Name<'company>,
        as_of: AsOf,
        balance: Option<NameBalanceRow>,
        owed: Option<Owed>,
        (transactions, left_out): (TransactionPage, LeftOut),
    ) -> Self {
        Self {
            record,
            as_of,
            balance: balance.map(|row| row.balance),
            total: balance.map(|row| row.total),
            owed,
            transactions,
            left_out,
        }
    }
}

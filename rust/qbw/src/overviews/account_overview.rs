//! An account.

use crate::accounts::Account;
use crate::as_of::AsOf;
use crate::ledger::Balance;
use crate::money::Money;
use crate::registers::Register;
use crate::reports::AccountListing;

/// What a person sees on selecting an account.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct AccountOverview<'company> {
    /// The record.
    pub record: &'company Account,
    /// The moment.
    pub as_of: AsOf,
    /// The balance of an account of the balance sheet, without those below it.
    /// `None` for an account of profit and loss, which starts again each year,
    /// and for one that is not part of the books.
    pub balance: Option<Balance>,
    /// The balance with those of the accounts below it, in the company's
    /// currency.
    pub total: Option<Money>,
    /// The latest rows of the register, the newest last as a register has them.
    /// It is of all the time up to the moment, so that of an account of profit
    /// and loss runs on over the years.
    pub register: Register,
}

impl<'company> AccountOverview<'company> {
    pub(crate) fn new(
        record: &'company Account,
        listing: &AccountListing,
        register: Register,
    ) -> Self {
        let mut listed = listing.rows.iter();
        let listed = listed.find(|row| row.account == record.id);
        Self {
            record,
            as_of: listing.as_of,
            balance: listed.and_then(|row| row.balance),
            total: listed.and_then(|row| row.total),
            register,
        }
    }
}

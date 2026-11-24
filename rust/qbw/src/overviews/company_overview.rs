//! The company.

use crate::accounts::{Account, AccountType};
use crate::as_of::AsOf;
use crate::left_out::LeftOut;
use crate::list::List;
use crate::open_items::Owed;
use crate::query::TransactionPage;
use crate::reports::{AccountListing, AccountListingRow};

/// What a person sees on opening the company: what there is in the bank, what is
/// owed, and what happened last.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CompanyOverview {
    /// The moment.
    pub as_of: AsOf,
    /// The active bank accounts with their balances, in the list's order.
    pub bank: Vec<AccountListingRow>,
    /// What customers owe, and what of it is overdue.
    pub receivable: Owed,
    /// What vendors are owed, and what of it is overdue.
    pub payable: Owed,
    /// The latest transactions, the newest first.
    pub transactions: TransactionPage,
    /// How much could not be read that the balances of the bank accounts may
    /// lack.
    pub left_out: LeftOut,
}

impl CompanyOverview {
    pub(crate) fn new(
        accounts: &List<Account>,
        listing: AccountListing,
        [receivable, payable]: [Owed; 2],
        transactions: TransactionPage,
    ) -> Self {
        let is_bank = |row: &AccountListingRow| {
            let account = accounts.get(row.account);
            account.is_some_and(|account| {
                account.is_active && account.account_type == AccountType::Bank
            })
        };
        Self {
            as_of: listing.as_of,
            bank: listing.rows.into_iter().filter(is_bank).collect(),
            receivable,
            payable,
            transactions,
            left_out: listing.left_out,
        }
    }
}

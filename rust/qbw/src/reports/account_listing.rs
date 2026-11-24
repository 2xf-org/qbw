//! The account listing.

use std::collections::HashMap;

use crate::accounts::Statement;
use crate::amount::Total;
use crate::as_of::AsOf;
use crate::ids::AccountId;
use crate::ledger::Balance;
use crate::left_out::LeftOut;
use crate::money::Money;
use crate::period::Period;
use crate::reports::Books;

/// The chart of accounts with its balances.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AccountListing {
    /// The moment of the balances.
    pub as_of: AsOf,
    /// Every account of the list, in its order.
    pub rows: Vec<AccountListingRow>,
    /// How much could not be read.
    pub left_out: LeftOut,
}

/// An account of an [`AccountListing`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AccountListingRow {
    /// The account.
    pub account: AccountId,
    /// The balance of an account of the balance sheet, without those below it.
    /// `None` for an account of profit and loss, which starts again each year,
    /// and for one that is not part of the books.
    pub balance: Option<Balance>,
    /// The balance with those of the accounts below it, in the company's
    /// currency. A debit is positive.
    pub total: Option<Money>,
}

pub(crate) fn account_listing(books: Books<'_>, as_of: AsOf) -> AccountListing {
    let period = as_of.period();
    let totals = totals(books, period);
    let rows = books.accounts.iter().map(|account| {
        let on_balance_sheet = account.account_type.statement() == Some(Statement::BalanceSheet);
        let entries = books.ledger.entries_during(account.id, period);
        let total = totals.get(&account.id).copied().unwrap_or_default();
        AccountListingRow {
            account: account.id,
            balance: on_balance_sheet.then(|| books.ledger.sum(account.id, entries)),
            total: on_balance_sheet.then(|| books.money(total)),
        }
    });
    AccountListing {
        as_of,
        rows: rows.collect(),
        left_out: books.left_out(),
    }
}

/// The balances of the accounts, each with those of the accounts below it.
fn totals(books: Books<'_>, period: Period) -> HashMap<AccountId, Total> {
    let mut totals: HashMap<AccountId, Total> = HashMap::new();
    for account in books.on(Statement::BalanceSheet) {
        let balance = books.ledger.total(account.id, period);
        for id in books.accounts.lineage(account.id) {
            *totals.entry(id).or_default() += balance;
        }
    }
    totals
}

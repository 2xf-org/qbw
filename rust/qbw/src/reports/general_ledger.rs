//! The general ledger.

use crate::amount::Total;
use crate::ids::AccountId;
use crate::ledger::{Entry, Ledger, Posting};
use crate::left_out::LeftOut;
use crate::money::Money;
use crate::period::Period;
use crate::reports::Books;

/// What is posted to every account in a period, with the balance it opens and
/// closes with.
///
/// An account of profit and loss opens with what was posted to it since the
/// fiscal year of the period's first day started, and Retained Earnings opens
/// with the profit of the years before. Within the period no year is closed.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct GeneralLedger<'company> {
    /// The period.
    pub period: Period,
    /// The accounts that have a balance or that something is posted to, in the
    /// list's order. Those that the list does not have come last.
    pub accounts: Vec<AccountLedger<'company>>,
    /// How much could not be read.
    pub left_out: LeftOut,
}

/// An account of a [`GeneralLedger`]. Amounts are in the company's currency,
/// and a debit is positive.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct AccountLedger<'company> {
    /// The account.
    pub account: AccountId,
    /// The balance before the period.
    pub opening: Money,
    /// The balance after it.
    pub closing: Money,
    opening_total: Total,
    ledger: &'company Ledger,
    entries: &'company [Entry],
}

/// A row of an [`AccountLedger`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct LedgerRow {
    /// The posting.
    pub posting: Posting,
    /// The account's balance after it, in the company's currency.
    pub balance: Money,
}

impl AccountLedger<'_> {
    /// How many rows the account has in the period.
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.entries.len()
    }

    /// The rows of the account, by date and by line. Each is put together when
    /// the iterator comes to it.
    pub fn rows(&self) -> impl Iterator<Item = LedgerRow> {
        let balances = self
            .entries
            .iter()
            .scan(self.opening_total, |balance, entry| {
                *balance += entry.home.into();
                Some(*balance)
            });
        let rows = self.entries.iter().zip(balances);
        rows.map(|(entry, balance)| LedgerRow {
            posting: self.ledger.posting(entry),
            balance: balance.money(self.ledger.home()),
        })
    }
}

pub(crate) fn general_ledger(books: Books<'_>, period: Period) -> GeneralLedger<'_> {
    let listed = books.accounts.iter().map(|account| account.id);
    let accounts = listed.chain(books.unlisted());
    let accounts = accounts.map(|account| account_ledger(books, account, period));
    let shown = accounts.filter(|account| !account.opening.is_zero() || account.row_count() > 0);
    GeneralLedger {
        period,
        accounts: shown.collect(),
        left_out: books.left_out(),
    }
}

fn account_ledger(books: Books<'_>, account: AccountId, period: Period) -> AccountLedger<'_> {
    let entries = books.ledger.entries_during(account, period);
    let opening = opening(books, account, period);
    let posted: Total = entries.iter().map(|entry| entry.home).sum();
    AccountLedger {
        account,
        opening: books.money(opening),
        closing: books.money(opening + posted),
        opening_total: opening,
        ledger: books.ledger,
        entries,
    }
}

/// The balance of an account before a period.
fn opening(books: Books<'_>, account: AccountId, period: Period) -> Total {
    let before = books.before(account, period);
    let balance = before.map_or(Total::default(), |before| {
        books.ledger.total(account, before)
    });
    balance + books.brought_forward(account, period)
}

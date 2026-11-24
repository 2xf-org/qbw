//! The trial balance.

use chrono::NaiveDate;

use crate::accounts::Statement;
use crate::amount::Total;
use crate::ids::AccountId;
use crate::left_out::LeftOut;
use crate::money::Money;
use crate::period::Period;
use crate::reports::Books;

/// The balance of every account at the end of a day. Debits and credits come
/// to the same.
///
/// An account of profit and loss has what was posted to it in the fiscal year
/// so far, and the profit of earlier years is part of Retained Earnings.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TrialBalance {
    /// The day. Of everything in the file, it is the last that something is
    /// posted on.
    pub as_of: NaiveDate,
    /// The accounts with a balance, in the list's order. Those that the list
    /// does not have come last.
    pub rows: Vec<TrialBalanceRow>,
    /// What the debit balances come to.
    pub debits: Money,
    /// What the credit balances come to.
    pub credits: Money,
    /// How much could not be read. Where that is not nothing, debits and
    /// credits can differ.
    pub left_out: LeftOut,
}

/// An account of a [`TrialBalance`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TrialBalanceRow {
    /// The account. `None` for the profit of earlier years in a file that has
    /// no account for it.
    pub account: Option<AccountId>,
    /// The balance in the company's currency. A debit is positive.
    pub balance: Money,
}

impl TrialBalanceRow {
    /// The balance, when it is a debit.
    #[must_use]
    pub fn debit(&self) -> Option<Money> {
        (!self.balance.is_negative()).then_some(self.balance)
    }

    /// The balance without its sign, when it is a credit.
    #[must_use]
    pub fn credit(&self) -> Option<Money> {
        self.balance.is_negative().then(|| self.balance.abs())
    }
}

pub(crate) fn trial_balance(books: Books<'_>, as_of: NaiveDate) -> TrialBalance {
    let mut balances = balances(books, as_of);
    balances.retain(|(_, balance)| !balance.is_zero());
    let (credits, debits): (Vec<Total>, Vec<Total>) = balances
        .iter()
        .map(|(_, balance)| *balance)
        .partition(|balance| balance.is_negative());
    let rows = balances
        .into_iter()
        .map(|(account, balance)| TrialBalanceRow {
            account,
            balance: books.money(balance),
        });
    TrialBalance {
        as_of,
        rows: rows.collect(),
        debits: books.money(debits.into_iter().sum()),
        credits: books.money(-credits.into_iter().sum::<Total>()),
        left_out: books.left_out(),
    }
}

fn balances(books: Books<'_>, as_of: NaiveDate) -> Vec<(Option<AccountId>, Total)> {
    let retained = books.retained_earnings();
    let earlier_profit = books.earlier_profit(as_of);
    let up_to = Period::until(as_of);
    let mut balances: Vec<(Option<AccountId>, Total)> = Vec::new();
    for account in books.accounts {
        let period = match account.account_type.statement() {
            Some(Statement::ProfitAndLoss) => books.year_to(as_of),
            Some(Statement::BalanceSheet) | None => up_to,
        };
        let mut balance = books.ledger.total(account.id, period);
        if Some(account.id) == retained {
            balance += -earlier_profit;
        }
        balances.push((Some(account.id), balance));
    }
    let unlisted = books.unlisted();
    balances.extend(unlisted.map(|account| (Some(account), books.ledger.total(account, up_to))));
    if retained.is_none() {
        balances.push((None, -earlier_profit));
    }
    balances
}

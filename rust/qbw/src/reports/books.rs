//! What the statements are worked out from.

use chrono::{Month, NaiveDate};

use crate::accounts::{Account, Statement};
use crate::amount::Total;
use crate::codes::{AccountType, SpecialAccount};
use crate::ids::AccountId;
use crate::ledger::Ledger;
use crate::left_out::LeftOut;
use crate::list::List;
use crate::money::Money;
use crate::period::Period;

/// What the statements are worked out from, and what they have in common.
#[derive(Clone, Copy)]
pub(crate) struct Books<'company> {
    pub(crate) ledger: &'company Ledger,
    pub(crate) accounts: &'company List<Account>,
    /// The month that the company's years start with.
    pub(crate) year_start: Month,
}

impl<'company> Books<'company> {
    /// The fiscal year that `date` is in.
    pub(super) fn year_of(self, date: NaiveDate) -> Period {
        Period::year_of(date, self.year_start)
    }

    /// The fiscal year of `date` up to it.
    pub(super) fn year_to(self, date: NaiveDate) -> Period {
        let first = self.year_of(date).first();
        first.map_or(Period::until(date), |first| Period::between(first, date))
    }

    /// Everything before the fiscal year of `date`. Nothing when that year is
    /// the first of the calendar.
    pub(super) fn before_year_of(self, date: NaiveDate) -> Option<Period> {
        let last = self.year_of(date).first()?.pred_opt()?;
        Some(Period::until(last))
    }

    /// What counts into the balance that an account opens a period with. One of
    /// profit and loss starts again with each fiscal year, so it opens with what
    /// was posted since the year of the first day started. Nothing for a period
    /// that has no first day.
    pub(crate) fn before(self, account: AccountId, period: Period) -> Option<Period> {
        let first = period.first()?;
        let before = first.pred_opt()?;
        match self.statement(account) {
            Some(Statement::ProfitAndLoss) => {
                let year = self.year_of(first).first()?;
                Some(Period::between(year, before))
            }
            Some(Statement::BalanceSheet) | None => Some(Period::until(before)),
        }
    }

    /// What an account opens a period with besides what is posted to it: on
    /// Retained Earnings, the profit of the years before the one of the first
    /// day. A debit is positive.
    pub(crate) fn brought_forward(self, account: AccountId, period: Period) -> Total {
        match period.first() {
            Some(first) if self.retained_earnings() == Some(account) => -self.earlier_profit(first),
            _ => Total::default(),
        }
    }

    /// The profit of a period: what was earned less what was spent. A loss is
    /// below zero.
    pub(super) fn profit(self, period: Period) -> Total {
        let accounts = self.on(Statement::ProfitAndLoss);
        -accounts
            .map(|account| self.ledger.total(account.id, period))
            .sum::<Total>()
    }

    /// The profit of every fiscal year before the one that `date` is in.
    pub(super) fn earlier_profit(self, date: NaiveDate) -> Total {
        let period = self.before_year_of(date);
        period.map_or(Total::default(), |period| self.profit(period))
    }

    /// The account of equity for the profit of earlier years.
    pub(super) fn retained_earnings(self) -> Option<AccountId> {
        let mut accounts = self.accounts.iter();
        let account = accounts.find(|account| {
            account.special == Some(SpecialAccount::RetainedEarnings)
                && account.account_type == AccountType::Equity
        });
        account.map(|account| account.id)
    }

    /// What is posted in a period to accounts that are on no statement: ones
    /// that cannot be read, that are of a type this version does not know, or
    /// that are not part of the books.
    pub(super) fn unplaced(self, period: Period) -> Total {
        let posted = self.ledger.posted_accounts();
        let unplaced = posted.filter(|&account| self.statement(account).is_none());
        unplaced
            .map(|account| self.ledger.total(account, period))
            .sum()
    }

    /// The accounts that something is posted to and that are not in the list.
    pub(super) fn unlisted(self) -> impl Iterator<Item = AccountId> + 'company {
        let posted = self.ledger.posted_accounts();
        posted.filter(move |&account| self.accounts.get(account).is_none())
    }

    /// The statement that an account is on.
    pub(super) fn statement(self, account: AccountId) -> Option<Statement> {
        self.accounts.get(account)?.account_type.statement()
    }

    /// The accounts on a statement, in the list's order.
    pub(super) fn on(self, statement: Statement) -> impl Iterator<Item = &'company Account> {
        let accounts = self.accounts.iter();
        accounts.filter(move |account| account.account_type.statement() == Some(statement))
    }

    /// A sum in the company's currency.
    pub(super) fn money(self, total: Total) -> Money {
        total.money(self.ledger.home())
    }

    /// How much the statements lack.
    pub(super) fn left_out(self) -> LeftOut {
        self.ledger.left_out()
    }
}

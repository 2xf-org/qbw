//! What the own line of a transaction says of it.

use chrono::NaiveDate;

use crate::currencies::Currencies;
use crate::ids::{AccountId, CurrencyId};
use crate::money::Money;
use crate::rest;
use crate::tables::lines::SummaryLineRow;
use crate::transactions::Header;
use crate::transactions::form_side::FormSide;

/// The own line of a transaction, with what decides how to read it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct OwnSide<'own> {
    header: &'own Header,
    line: &'own SummaryLineRow,
    currencies: Currencies,
}

impl<'own> OwnSide<'own> {
    pub(crate) fn new(header: &'own Header, line: &'own SummaryLineRow, home: CurrencyId) -> Self {
        Self {
            header,
            line,
            currencies: Currencies::of_line(line.currency, home),
        }
    }

    /// The currency of the transaction.
    pub(crate) const fn currency(self) -> CurrencyId {
        self.currencies.own()
    }

    pub(crate) fn account(self) -> Option<AccountId> {
        self.line.account
    }

    /// The amount in the transaction's currency, a debit as positive.
    pub(crate) fn amount(self) -> Option<Money> {
        let line = self.line;
        self.currencies
            .own_amount(line.home_amount, line.foreign_amount)
    }

    /// The amount in the company's currency, a debit as positive.
    pub(crate) fn home_amount(self) -> Option<Money> {
        self.currencies.home_amount(self.line.home_amount)
    }

    /// The total that the form shows, which is zero where the own line has no
    /// amount.
    pub(crate) fn total(self) -> Option<Money> {
        let amount = self.amount();
        self.shown(amount.unwrap_or_else(|| Money::zero(self.currencies.own())))
    }

    /// The total in the company's currency.
    pub(crate) fn home_total(self) -> Option<Money> {
        let amount = self.home_amount();
        self.shown(amount.unwrap_or_else(|| Money::zero(self.currencies.home())))
    }

    fn shown(self, amount: Money) -> Option<Money> {
        let total = FormSide::of(self.header.kind).total(amount)?;
        Some(if self.header.is_without_money {
            // The own line is the discount.
            Money::zero(total.currency())
        } else {
            total
        })
    }

    /// When it is due, on a form that shows it.
    pub(crate) fn due_date(self) -> Option<NaiveDate> {
        rest::due_date(self.header.kind, self.line.due_date)
    }
}

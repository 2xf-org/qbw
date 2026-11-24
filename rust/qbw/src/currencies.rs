//! The two currencies of an amount.

use crate::amount::Amount;
use crate::ids::CurrencyId;
use crate::money::Money;

/// The currency of something, such as a line or an account, and the one that the
/// company keeps its books in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Currencies {
    own: CurrencyId,
    home: CurrencyId,
}

impl Currencies {
    /// `own` in the books of a company that keeps them in `home`.
    pub(crate) const fn new(own: CurrencyId, home: CurrencyId) -> Self {
        Self { own, home }
    }

    /// The currencies of a line. The lines of a company of one currency say
    /// nothing of it.
    pub(crate) fn of_line(stored: Option<CurrencyId>, home: CurrencyId) -> Self {
        Self::new(stored.unwrap_or(home), home)
    }

    pub(crate) const fn own(self) -> CurrencyId {
        self.own
    }

    pub(crate) const fn home(self) -> CurrencyId {
        self.home
    }

    /// Which of two stored amounts is the one in the own currency: the one in
    /// the company's, or the one next to it.
    ///
    /// Next to an amount in the company's currency can be something else: what
    /// an account in another currency gets of a transfer.
    pub(crate) fn own_of<T>(self, home: T, foreign: T) -> T {
        if self.own == self.home { home } else { foreign }
    }

    /// The amount in the own currency.
    pub(crate) fn own_amount(self, home: Option<Amount>, foreign: Option<Amount>) -> Option<Money> {
        let amount = self.own_of(home, foreign);
        amount.map(|amount| amount.money(self.own))
    }

    /// The amount in the company's currency.
    pub(crate) fn home_amount(self, home: Option<Amount>) -> Option<Money> {
        home.map(|amount| amount.money(self.home))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOME: CurrencyId = CurrencyId::new(1);
    const EURO: CurrencyId = CurrencyId::new(2);

    fn amount(text: &str) -> Option<Amount> {
        Amount::new(text.parse().unwrap())
    }

    fn own(currency: Option<CurrencyId>, home: &str, foreign: Option<Amount>) -> Option<String> {
        let currencies = Currencies::of_line(currency, HOME);
        let own = currencies.own_amount(amount(home), foreign);
        own.map(|own| format!("{own} in {}", own.currency()))
    }

    #[test]
    fn the_other_amount_is_the_own_one_only_in_another_currency() {
        let invoice = own(Some(EURO), "110", amount("100"));
        assert_eq!(invoice.as_deref(), Some("100.00 in 2"));
        let transfer = own(Some(HOME), "-55", amount("-50"));
        assert_eq!(transfer.as_deref(), Some("-55.00 in 1"));
        let one_currency = own(None, "7", amount("7"));
        assert_eq!(one_currency.as_deref(), Some("7.00 in 1"));
        assert_eq!(own(Some(EURO), "-10", None), None);
    }
}

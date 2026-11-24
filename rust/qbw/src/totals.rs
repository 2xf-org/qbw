//! Sums that keep currencies apart.

use std::collections::BTreeMap;

use crate::ids::CurrencyId;
use crate::money::Money;
use crate::money_error::MoneyError;

/// A sum of amounts of any currencies, one sum for each.
///
/// In a company that keeps its books in one currency there is at most one
/// sum, which [`Self::single`] gives.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Totals {
    sums: BTreeMap<CurrencyId, Money>,
}

impl Totals {
    /// The sum of no amounts.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The sums `sums`, each of a currency of its own.
    pub(crate) fn of(sums: impl IntoIterator<Item = Money>) -> Self {
        let sums = sums.into_iter().map(|sum| (sum.currency(), sum));
        Self {
            sums: sums.collect(),
        }
    }

    /// Adds `amount` to the sum of its currency.
    ///
    /// # Errors
    ///
    /// Returns an error when the sum has more digits than a number holds, and
    /// leaves it as it was.
    pub fn add(&mut self, amount: Money) -> Result<(), MoneyError> {
        let sum = self.get(amount.currency()).checked_add(amount)?;
        self.sums.insert(amount.currency(), sum);
        Ok(())
    }

    /// The sum of the amounts in `currency`, which is zero when there were none.
    #[must_use]
    pub fn get(&self, currency: CurrencyId) -> Money {
        let sum = self.sums.get(&currency);
        sum.copied().unwrap_or_else(|| Money::zero(currency))
    }

    /// The sum of each currency that an amount was in, ordered by currency.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = Money> {
        self.sums.values().copied()
    }

    /// The only sum, when every amount was in one currency.
    #[must_use]
    pub fn single(&self) -> Option<Money> {
        let mut sums = self.iter();
        sums.next().filter(|_| sums.next().is_none())
    }

    /// Whether no amount was added.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sums.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOME: CurrencyId = CurrencyId::new(1);
    const FOREIGN: CurrencyId = CurrencyId::new(2);

    fn money(text: &str, currency: CurrencyId) -> Money {
        Money::new(text.parse().unwrap(), currency)
    }

    #[test]
    fn each_currency_has_its_own_sum() {
        let mut totals = Totals::new();
        assert!(totals.is_empty() && totals.single().is_none());
        totals.add(money("10", FOREIGN)).unwrap();
        totals.add(money("1.50", HOME)).unwrap();
        assert_eq!(totals.single(), None);
        totals.add(money("-0.50", HOME)).unwrap();
        let sums: Vec<Money> = totals.iter().collect();
        assert_eq!(sums, [money("1", HOME), money("10", FOREIGN)]);
        assert_eq!(
            totals.get(CurrencyId::new(3)),
            Money::zero(CurrencyId::new(3))
        );
    }

    #[test]
    fn one_currency_has_a_single_sum() {
        let mut totals = Totals::new();
        for amount in ["1", "2"] {
            totals.add(money(amount, CurrencyId::COMPANY)).unwrap();
        }
        assert_eq!(totals.single(), Some(money("3", CurrencyId::COMPANY)));
    }

    #[test]
    fn a_sum_that_does_not_fit_stays_as_it_was() {
        let largest = Money::new(sa17::Decimal::new(i128::MAX, 0).unwrap(), HOME);
        let mut totals = Totals::new();
        totals.add(largest).unwrap();
        assert_eq!(totals.add(money("1", HOME)), Err(MoneyError::TooLarge));
        assert_eq!(totals.single(), Some(largest));
    }
}

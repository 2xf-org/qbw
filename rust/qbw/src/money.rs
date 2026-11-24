//! Amounts of one currency.

use std::cmp::Ordering;
use std::fmt;
use std::ops::Neg;

use sa17::Decimal;

use crate::ids::CurrencyId;
use crate::money_error::MoneyError;

/// The digits after the point of a cent: what an amount is written with at least,
/// and what one that was worked out is rounded to.
pub(crate) const CENTS: u16 = 2;

/// An exact amount in one currency.
///
/// Amounts of two currencies do not add, and neither is the larger one. In a
/// company that keeps its books in one currency every amount is in
/// [`CurrencyId::COMPANY`], so there is nothing to mix.
///
/// ```
/// use qbw::{CurrencyId, Money};
///
/// let money = |text: &str| Some(Money::new(text.parse().ok()?, CurrencyId::COMPANY));
/// let (goods, tax) = (money("150.00").ok_or("goods")?, money("12.00").ok_or("tax")?);
/// assert_eq!(goods.checked_add(tax)?.to_string(), "162.00");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Money {
    amount: Decimal,
    currency: CurrencyId,
}

impl Money {
    /// `amount` in `currency`.
    #[must_use]
    pub const fn new(amount: Decimal, currency: CurrencyId) -> Self {
        Self { amount, currency }
    }

    /// Nothing, in `currency`.
    #[must_use]
    pub fn zero(currency: CurrencyId) -> Self {
        Self::new(Decimal::default(), currency)
    }

    /// The number, without its currency.
    #[must_use]
    pub const fn amount(self) -> Decimal {
        self.amount
    }

    /// The currency.
    #[must_use]
    pub const fn currency(self) -> CurrencyId {
        self.currency
    }

    /// Whether the amount is zero.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.amount.is_zero()
    }

    /// Whether the amount is below zero.
    #[must_use]
    pub const fn is_negative(self) -> bool {
        self.amount.is_negative()
    }

    /// The amount without its sign.
    #[must_use]
    pub const fn abs(self) -> Self {
        Self::new(self.amount.abs(), self.currency)
    }

    /// The sum.
    ///
    /// # Errors
    ///
    /// Returns an error for amounts of two currencies, and for a sum of more
    /// digits than a number holds.
    pub fn checked_add(self, other: Self) -> Result<Self, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::Currencies {
                left: self.currency,
                right: other.currency,
            });
        }
        let sum = self.amount.checked_add(other.amount);
        sum.map(|amount| Self::new(amount, self.currency))
            .ok_or(MoneyError::TooLarge)
    }

    /// The difference.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::checked_add`].
    pub fn checked_sub(self, other: Self) -> Result<Self, MoneyError> {
        self.checked_add(-other)
    }
}

impl Neg for Money {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.amount, self.currency)
    }
}

impl PartialOrd for Money {
    /// Orders amounts of one currency. Those of two have no order.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (self.currency == other.currency).then(|| self.amount.cmp(&other.amount))
    }
}

impl fmt::Display for Money {
    /// Writes the number, without its currency: with two digits after the
    /// point, or with more where it has them.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let amount = self.amount.normalize();
        let padded = amount.rescale(amount.scale().max(CENTS));
        padded.unwrap_or(amount).fmt(formatter)
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
    fn amounts_of_one_currency_add_exactly() {
        let sum = money("0.10", HOME).checked_add(money("0.2", HOME)).unwrap();
        assert_eq!(
            (sum, sum.to_string()),
            (money("0.3", HOME), "0.30".to_owned())
        );
        let difference = money("1", HOME).checked_sub(money("1.00", HOME)).unwrap();
        assert!(difference.is_zero() && !difference.is_negative());
        assert_eq!(-money("5", HOME).abs(), money("-5", HOME));
    }

    #[test]
    fn an_amount_is_written_with_cents_or_more() {
        // The file keeps five digits after the point.
        assert_eq!(money("837.00000", HOME).to_string(), "837.00");
        assert_eq!(money("0.12500", HOME).to_string(), "0.125");
        assert_eq!(Money::zero(HOME).to_string(), "0.00");
        assert_eq!(format!("{:>8}", money("-1.5", HOME)), "   -1.50");
    }

    #[test]
    fn amounts_of_two_currencies_do_not_mix() {
        let (home, foreign) = (money("1", HOME), money("1", FOREIGN));
        let mixed = MoneyError::Currencies {
            left: HOME,
            right: FOREIGN,
        };
        assert_eq!(home.checked_add(foreign), Err(mixed));
        assert_eq!(home.partial_cmp(&foreign), None);
        assert_ne!(home, foreign);
        assert!(home < money("2", HOME));
    }

    #[test]
    fn a_sum_past_the_largest_number_is_an_error() {
        let largest = Money::new(Decimal::new(i128::MAX, 0).unwrap(), HOME);
        assert_eq!(
            largest.checked_add(money("1", HOME)),
            Err(MoneyError::TooLarge)
        );
        assert_eq!(Money::zero(HOME).checked_add(largest), Ok(largest));
    }
}

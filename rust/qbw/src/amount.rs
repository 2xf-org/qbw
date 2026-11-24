//! Amounts as they are stored, to add up many of them.

use std::iter;
use std::ops::{Add, AddAssign, Neg, Sub};

use sa17::Decimal;

use crate::ids::CurrencyId;
use crate::money::{CENTS, Money};

/// The digits after the point that the file keeps amounts with.
const SCALE: u16 = 5;

/// An amount as the file keeps it: a whole number of hundred-thousandths.
///
/// It is a quarter of the size of a [`Decimal`], and reaches 92 trillion to
/// either side.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Amount(i64);

impl Amount {
    /// One.
    pub(crate) const ONE: Self = Self(100_000);

    /// The amount `number`, or `None` for one that is too large or has more
    /// digits after the point.
    pub(crate) fn new(number: Decimal) -> Option<Self> {
        let units = number.rescale(SCALE)?.mantissa();
        let units = i64::try_from(units).ok();
        // Every amount has a negative.
        units.filter(|&units| units != i64::MIN).map(Self)
    }

    pub(crate) const fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub(crate) const fn is_negative(self) -> bool {
        self.0 < 0
    }

    /// The amount as money.
    pub(crate) fn money(self, currency: CurrencyId) -> Money {
        Total::from(self).money(currency)
    }

    /// The amount as a quantity, which the file keeps with as many digits.
    pub(crate) fn quantity(self) -> Decimal {
        Total::from(self).quantity()
    }

    /// The amount at an exchange rate, to the cent. Half a cent goes away from
    /// zero. `None` when it is too large, or the rate is no number.
    pub(crate) fn at_rate(self, rate: f32) -> Option<Self> {
        // The shortest number that the float stands for is the rate that was
        // typed.
        let rate: Decimal = rate.to_string().parse().ok()?;
        let amount = Total::from(self).number();
        Self::new(amount.checked_mul_round(rate, CENTS)?)
    }
}

/// A sum of amounts.
///
/// It holds more of the largest of them than there is memory for, so it needs
/// no check for a sum that does not fit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Total(i128);

impl Total {
    /// The sum as money.
    pub(crate) fn money(self, currency: CurrencyId) -> Money {
        Money::new(self.number(), currency)
    }

    /// The sum as a quantity. It has no zeros at the end of its fraction, so it
    /// prints as a person writes it.
    pub(crate) fn quantity(self) -> Decimal {
        self.number().normalize()
    }

    fn number(self) -> Decimal {
        // Only a mantissa that no sum reaches is not a number.
        Decimal::new(self.0, SCALE).unwrap_or_default()
    }

    /// The sum as one amount, unless it is too large.
    pub(crate) fn amount(self) -> Option<Amount> {
        let units = i64::try_from(self.0).ok();
        units.filter(|&units| units != i64::MIN).map(Amount)
    }

    pub(crate) const fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub(crate) const fn is_negative(self) -> bool {
        self.0 < 0
    }
}

impl Neg for Amount {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl From<Amount> for Total {
    fn from(amount: Amount) -> Self {
        Self(amount.0.into())
    }
}

impl Add for Total {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl AddAssign for Total {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sub for Total {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self + -other
    }
}

impl Neg for Total {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl iter::Sum for Total {
    fn sum<I: Iterator<Item = Self>>(totals: I) -> Self {
        totals.fold(Self::default(), Add::add)
    }
}

impl iter::Sum<Amount> for Total {
    fn sum<I: Iterator<Item = Amount>>(amounts: I) -> Self {
        amounts.map(Self::from).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn amount(text: &str) -> Option<Amount> {
        Amount::new(text.parse().unwrap())
    }

    #[test]
    fn amounts_add_up_exactly() {
        let amounts = ["0.10", "0.20", "-1.30", "0.00001"];
        let total: Total = amounts.iter().filter_map(|text| amount(text)).sum();
        assert_eq!(total.money(CurrencyId::COMPANY).to_string(), "-0.99999");
        assert!(total.is_negative() && (total - total).is_zero());
        assert_eq!(-total + total, Total::default());
    }

    #[test]
    fn a_number_that_the_file_has_no_room_for_is_no_amount() {
        assert!(amount("92233720368547.75807").is_some());
        assert_eq!(amount("92233720368547.75808"), None);
        assert_eq!(amount("-92233720368547.75808"), None);
        assert_eq!(amount("0.000001"), None);
        assert_eq!(amount("0.1000000"), amount("0.1"));
    }

    #[test]
    fn an_amount_at_a_rate_is_rounded_to_the_cent() {
        let at_rate = |text: &str, rate| {
            let amount = amount(text)?.at_rate(rate)?;
            Some(amount.money(CurrencyId::COMPANY).to_string())
        };
        assert_eq!(at_rate("50", 1.1).as_deref(), Some("55.00"));
        assert_eq!(at_rate("-33.33", 1.1).as_deref(), Some("-36.66"));
        // Half a cent.
        assert_eq!(at_rate("0.01", 0.5).as_deref(), Some("0.01"));
        assert_eq!(at_rate("-0.01", 0.5).as_deref(), Some("-0.01"));
        assert_eq!(at_rate("1", f32::NAN), None);
        assert_eq!(at_rate("92233720368547", f32::MAX), None);
    }

    #[test]
    fn a_sum_is_an_amount_where_it_fits() {
        let largest = Amount(i64::MAX);
        assert_eq!(Total::from(largest).amount(), Some(largest));
        assert_eq!((Total::from(largest) + Total(1)).amount(), None);
        assert_eq!(Total(i128::from(i64::MIN)).amount(), None);
    }

    #[test]
    fn a_sum_of_the_largest_amounts_fits() {
        let largest = Amount(i64::MAX);
        let total: Total = iter::repeat_n(largest, 1000).sum();
        assert_eq!(total, Total(i128::from(i64::MAX) * 1000));
    }
}

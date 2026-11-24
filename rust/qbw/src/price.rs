//! Prices.

use sa17::Decimal;

use crate::money::Money;

/// What something is sold for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Price {
    /// So much for one.
    Each(Money),
    /// A share of the lines that it goes by, in percent: 8 for a sales tax of 8%.
    Percent(Decimal),
}

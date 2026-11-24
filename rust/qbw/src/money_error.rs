//! Amounts that cannot be added.

use crate::ids::CurrencyId;

/// Why two amounts have no sum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum MoneyError {
    /// They are of two currencies.
    #[error("an amount in currency {left} does not add to one in currency {right}")]
    Currencies {
        /// The currency of the first amount.
        left: CurrencyId,
        /// The currency of the other amount.
        right: CurrencyId,
    },
    /// The sum has more digits than a number holds, about 38.
    #[error("the sum has too many digits")]
    TooLarge,
}

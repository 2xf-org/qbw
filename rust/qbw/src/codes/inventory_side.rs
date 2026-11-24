//! The two lines that an item in stock adds to a transaction.

use crate::number::FromNumber;

/// Which of the two lines it is that a movement of stock adds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InventorySide {
    /// The change of the stock's value.
    Asset,
    /// The change of what the goods sold had cost.
    CostOfGoodsSold,
    /// A line that this version has no name for. It posts all the same.
    Other,
}

impl FromNumber for InventorySide {
    fn from_number(number: i64) -> Option<Self> {
        Some(match number {
            1 => Self::Asset,
            2 => Self::CostOfGoodsSold,
            _ => Self::Other,
        })
    }
}

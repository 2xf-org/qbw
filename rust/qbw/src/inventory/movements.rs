//! What happened to an item.

use crate::ids::ItemId;
use crate::inventory::StockMovement;
use crate::left_out::LeftOut;

/// What happened to the stock of an item.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Movements {
    /// The item.
    pub item: ItemId,
    /// Every line that changed how much there is of it, or what that is worth,
    /// in the order QuickBooks counts them: by date, and within a day by line.
    pub rows: Vec<StockMovement>,
    /// How much could not be read. A movement after one that could not be read
    /// has the change of both.
    pub left_out: LeftOut,
}

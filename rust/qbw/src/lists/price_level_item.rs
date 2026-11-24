//! The price of an item at a price level.

use crate::ids::ItemId;
use crate::origin::Origin;
use crate::price::Price;

/// The price of one item at a price level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct PriceLevelItem {
    /// The item.
    pub item: ItemId,
    /// What it is sold for, or by how many percent its price changes. The
    /// percentages are unchecked.
    pub price: Option<Price>,
    /// The stored row.
    pub origin: Origin,
}

//! What a posting comes from.

use crate::codes::InventorySide;

/// What a posting comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum PostingSource {
    /// The transaction's own side: the bank's of a check, or what the customer
    /// owes of an invoice.
    OwnLine,
    /// Another line of the transaction.
    Line,
    /// The change of the stock's value, by a line of an item in stock.
    StockValue,
    /// The cost of what was sold, by a line of an item in stock.
    CostOfGoodsSold,
    /// What else a line of an item in stock posts. Nothing in the files that
    /// this crate was checked against.
    OtherStock,
}

impl From<InventorySide> for PostingSource {
    fn from(side: InventorySide) -> Self {
        match side {
            InventorySide::Asset => Self::StockValue,
            InventorySide::CostOfGoodsSold => Self::CostOfGoodsSold,
            InventorySide::Other => Self::OtherStock,
        }
    }
}

//! What there is in stock now.

use crate::ids::ItemId;
use crate::inventory::StockRow;
use crate::left_out::LeftOut;
use crate::money::Money;

/// What there is in stock now, and what is ordered and planned.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Stock {
    /// Every item in stock in the order of the list, inactive ones among them.
    pub rows: Vec<StockRow>,
    /// What all of it is worth, in the company's currency.
    pub value: Money,
    /// How much could not be read.
    pub left_out: LeftOut,
}

impl Stock {
    /// What there is of `item`. `None` for one that is not kept in stock.
    #[must_use]
    pub fn get(&self, item: ItemId) -> Option<&StockRow> {
        self.rows.iter().find(|row| row.item == item)
    }
}

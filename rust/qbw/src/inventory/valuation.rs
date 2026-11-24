//! What the stock was worth on a day.

use crate::as_of::AsOf;
use crate::ids::ItemId;
use crate::inventory::ValuationRow;
use crate::left_out::LeftOut;
use crate::money::Money;

/// What there was in stock at a moment, and what it was worth.
///
/// What was on order or planned that day is not here. The file keeps of an
/// order how much is left of it now, and not how much was left on a day.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Valuation {
    /// The moment.
    pub as_of: AsOf,
    /// Every item in stock in the order of the list, inactive ones among them.
    pub rows: Vec<ValuationRow>,
    /// What all of it was worth, in the company's currency.
    pub value: Money,
    /// How much could not be read.
    pub left_out: LeftOut,
}

impl Valuation {
    /// What there was of `item`. `None` for one that is not kept in stock.
    #[must_use]
    pub fn get(&self, item: ItemId) -> Option<&ValuationRow> {
        self.rows.iter().find(|row| row.item == item)
    }
}

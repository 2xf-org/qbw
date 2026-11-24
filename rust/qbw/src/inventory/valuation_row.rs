//! What an item was worth on a day.

use sa17::Decimal;

use crate::ids::ItemId;
use crate::money::Money;

/// What there was of an item at the end of a day, and what it was worth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ValuationRow {
    /// The item.
    pub item: ItemId,
    /// How much there was. Below zero when more was sold than there was.
    pub on_hand: Decimal,
    /// What it was worth, in the company's currency.
    pub value: Money,
    /// What one was worth on average. `None` before the history of the item
    /// starts.
    pub average_cost: Option<Money>,
}

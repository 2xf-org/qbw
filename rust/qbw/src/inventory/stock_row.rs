//! What there is of an item now.

use sa17::Decimal;

use crate::ids::ItemId;
use crate::inventory::SiteStock;
use crate::items::StoredStock;
use crate::money::Money;

/// What there is of an item now, and what is ordered and planned.
///
/// Values and costs are in the company's currency.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct StockRow {
    /// The item.
    pub item: ItemId,
    /// How much there is, after the last line that the history has. Below zero
    /// when more was sold than there was.
    pub on_hand: Decimal,
    /// What it is worth.
    pub value: Money,
    /// What one is worth on average. `None` for an item that nothing happened
    /// to yet.
    pub average_cost: Option<Money>,
    /// What the lines of transactions add up to, which is worked out apart
    /// from the history and should be [`Self::on_hand`].
    pub counted: Decimal,
    /// How much is still to come from vendors on purchase orders.
    pub on_order: Decimal,
    /// How much customers are still to get on sales orders.
    pub on_sales_orders: Decimal,
    /// How much builds that are pending will add.
    pub to_be_built: Decimal,
    /// How much builds that are pending will take.
    pub required_for_builds: Decimal,
    /// How much of what there is can still be promised: less what is on sales
    /// orders and what builds need, as the company's preferences say.
    ///
    /// This is inferred from what QuickBooks asks in the preferences. The file
    /// keeps no such figure.
    pub available: Decimal,
    /// The quantity at which more is to be ordered.
    pub reorder_point: Option<Decimal>,
    /// How much there is at each site and in each bin that has had some, in
    /// the order of the sites.
    pub sites: Vec<SiteStock>,
    /// What QuickBooks keeps with the item, to check the figures above.
    pub stored: Option<StoredStock>,
}

impl StockRow {
    /// Whether more is to be ordered: what there is and what is on order
    /// together are not more than the reorder point. Never for an item without
    /// one.
    ///
    /// This is inferred from what QuickBooks' reminders are about. The file
    /// does not say which items they list.
    #[must_use]
    pub fn needs_reorder(&self) -> bool {
        let coming = self.on_hand.checked_add(self.on_order);
        let coming = coming.zip(self.reorder_point);
        coming.is_some_and(|(coming, reorder_point)| coming <= reorder_point)
    }
}

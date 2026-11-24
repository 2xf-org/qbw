//! The stock that QuickBooks keeps.

use sa17::Decimal;

use crate::money::Money;

/// The stock that QuickBooks keeps with an item, over all sites.
///
/// It serves to check what the movements of the stock give.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct StoredStock {
    /// How much there is.
    pub on_hand: Decimal,
    /// What it is worth.
    pub value: Option<Money>,
    /// What one is worth on average.
    pub average_cost: Option<Money>,
    /// How much is ordered from vendors.
    pub on_order: Option<Decimal>,
    /// How much customers have ordered.
    pub on_sales_orders: Option<Decimal>,
    /// How much pending builds will add.
    pub to_be_built: Option<Decimal>,
    /// How much pending builds will take.
    pub required_for_builds: Option<Decimal>,
}

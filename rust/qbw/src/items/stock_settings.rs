//! How an item is kept in stock.

use sa17::Decimal;

use crate::ids::AccountId;

/// How an item is kept in stock.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct StockSettings {
    /// The account that the value of the stock is in.
    pub asset_account: Option<AccountId>,
    /// The quantity at which more is to be ordered.
    pub reorder_point: Option<Decimal>,
    /// The quantity at which more of an assembly is to be built.
    pub build_point: Option<Decimal>,
}

//! How a price level changes prices.

use crate::code::codes;

codes! {
    /// How a price level changes prices.
    #[derive(Default)]
    pub enum PriceLevelKind {
        /// Every price by the same share.
        #[default]
        FixedPercentage = 0,
        /// Each item has a price of its own.
        PerItem = 1,
    }
}

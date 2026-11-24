//! What there is of an item at a site.

use sa17::Decimal;

use crate::ids::SiteId;

/// What there is of an item at a site or in a bin of one. What is at a site has
/// what is in its bins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SiteStock {
    /// The site or the bin.
    pub site: SiteId,
    /// How much there is: what the lines of transactions add up to.
    pub on_hand: Decimal,
    /// The quantity at which more is to be ordered for the site.
    pub reorder_point: Option<Decimal>,
    /// What QuickBooks keeps as the quantity, to check it.
    pub stored: Option<Decimal>,
}

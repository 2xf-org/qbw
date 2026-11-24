//! The stock of an item at a site.

use sa17::Decimal;

use crate::ids::SiteId;
use crate::origin::Origin;

/// The stock of an item at a site or in a bin, as QuickBooks keeps it. A
/// site's is that of its bins together.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ItemSite {
    /// The site or the bin. Nothing on a row of neither, whose quantity is
    /// zero.
    pub site: Option<SiteId>,
    /// How much there is.
    pub on_hand: Option<Decimal>,
    /// The quantity at which more is to be ordered.
    pub reorder_point: Option<Decimal>,
    /// The stored row.
    pub origin: Origin,
}

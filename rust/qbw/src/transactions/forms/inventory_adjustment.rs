//! What a change of the stock has.

use crate::ids::{NameId, SiteId};
use crate::tables::headers::AdjustmentRow;
use crate::tables::lines::LineRow;

/// What a change of the quantities or the values in stock has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct InventoryAdjustment {
    /// The site it is at.
    pub site: Option<SiteId>,
    /// The customer or the job that it is for.
    pub customer: Option<NameId>,
}

impl InventoryAdjustment {
    pub(crate) fn new(row: &AdjustmentRow, own: Option<&LineRow>) -> Self {
        Self {
            site: row.site,
            customer: own.and_then(|own| own.customer),
        }
    }
}

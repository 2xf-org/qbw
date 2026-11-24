//! What a build of an assembly has.

use sa17::Decimal;

use crate::amount::Amount;
use crate::ids::{ItemId, SiteId};
use crate::tables::headers::BuildRow;
use crate::tables::lines::LineRow;

/// What a build of an assembly has. Its lines are what went into it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Build {
    /// The assembly that was built.
    pub item: Option<ItemId>,
    /// How many.
    pub quantity: Option<Decimal>,
    /// The site it was built at.
    pub site: Option<SiteId>,
}

impl Build {
    pub(crate) fn new(row: &BuildRow, own: Option<&LineRow>) -> Self {
        Self {
            item: row.item,
            quantity: own.and_then(|own| own.quantity).map(Amount::quantity),
            site: own.and_then(|own| own.site),
        }
    }
}

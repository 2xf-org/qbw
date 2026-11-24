//! What a group or an assembly is made of.

use sa17::Decimal;

use crate::ids::ItemId;
use crate::origin::Origin;

/// An item that a group, an assembly or a tax group is made of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Member {
    /// The item.
    pub item: ItemId,
    /// How much of it there is in one. A tax has none.
    pub quantity: Option<Decimal>,
    /// The stored row.
    pub origin: Origin,
}

//! Ways of shipping.

use sa17::Source;

use crate::error::Result;
use crate::ids::ShipMethodId;
use crate::list::{List, ListEntry, sealed};
use crate::lists::reading;
use crate::origin::Origin;
use crate::tables::lists::SHIP_METHODS;

/// A way that goods are shipped.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ShipMethod {
    /// What the method is known by.
    pub id: ShipMethodId,
    /// Its name.
    pub name: String,
    /// Whether it can be used in new transactions.
    pub is_active: bool,
    /// The stored row.
    pub origin: Origin,
}

impl sealed::Sealed for ShipMethod {}

impl ListEntry for ShipMethod {
    type Id = ShipMethodId;

    fn id(&self) -> ShipMethodId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn origin(&self) -> Origin {
        self.origin
    }
}

/// Reads the ways of shipping.
///
/// # Errors
///
/// Returns an error when their table lacks a column.
pub(crate) fn read(source: &dyn Source) -> Result<List<ShipMethod>> {
    reading::by_name(source, &SHIP_METHODS, |origin, row| {
        (!row.is_deleted).then(|| ShipMethod {
            id: row.id,
            name: row.name.unwrap_or_default(),
            is_active: !row.is_hidden,
            origin,
        })
    })
}

//! Types of vendors.

use sa17::Source;

use crate::error::Result;
use crate::ids::VendorTypeId;
use crate::list::List;
use crate::lists::reading::Place;
use crate::lists::{Label, reading};
use crate::tables::lists::{VENDOR_TYPES, VendorTypeRow};

/// A type of vendors.
pub type VendorType = Label<VendorTypeId>;

/// Reads the vendor types.
///
/// # Errors
///
/// Returns an error when their table lacks a column.
pub(crate) fn read(source: &dyn Source) -> Result<List<VendorType>> {
    let place = |row: &VendorTypeRow| Place::of(row.id.number(), row.next);
    reading::in_stored_order(source, &VENDOR_TYPES, place, |origin, row| {
        (!row.is_deleted).then(|| Label {
            id: row.id,
            name: row.name.unwrap_or_default(),
            parent: row.parent,
            is_active: !row.is_hidden,
            origin,
        })
    })
}

//! Types of customers.

use sa17::Source;

use crate::error::Result;
use crate::ids::CustomerTypeId;
use crate::list::List;
use crate::lists::reading::Place;
use crate::lists::{Label, reading};
use crate::tables::lists::{CUSTOMER_TYPES, CustomerTypeRow};

/// A type of customers.
pub type CustomerType = Label<CustomerTypeId>;

/// Reads the customer types.
///
/// # Errors
///
/// Returns an error when their table lacks a column.
pub(crate) fn read(source: &dyn Source) -> Result<List<CustomerType>> {
    let place = |row: &CustomerTypeRow| Place::of(row.id.number(), row.next);
    reading::in_stored_order(source, &CUSTOMER_TYPES, place, |origin, row| {
        (!row.is_deleted).then(|| Label {
            id: row.id,
            name: row.name.unwrap_or_default(),
            parent: row.parent,
            is_active: !row.is_hidden,
            origin,
        })
    })
}

//! Categories of items.

use sa17::Source;

use crate::error::Result;
use crate::ids::CategoryId;
use crate::list::List;
use crate::lists::reading::Place;
use crate::lists::{Label, reading};
use crate::tables::lists::{CATEGORIES, CategoryRow};

/// A category of items.
pub type Category = Label<CategoryId>;

/// Reads the categories.
///
/// # Errors
///
/// Returns an error when their table lacks a column.
pub(crate) fn read(source: &dyn Source) -> Result<List<Category>> {
    let place = |row: &CategoryRow| Place::of(row.id.number(), row.next);
    reading::in_stored_order(source, &CATEGORIES, place, |origin, row| {
        (!row.is_deleted).then(|| Label {
            id: row.id,
            name: row.name.unwrap_or_default(),
            parent: row.parent,
            is_active: !row.is_hidden,
            origin,
        })
    })
}

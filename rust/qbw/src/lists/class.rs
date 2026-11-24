//! Classes.

use sa17::Source;

use crate::error::Result;
use crate::ids::ClassId;
use crate::list::List;
use crate::lists::reading::Place;
use crate::lists::{Label, reading};
use crate::tables::lists::{CLASSES, ClassRow};

/// A class: what lines are sorted by besides their accounts. Unchecked.
pub type Class = Label<ClassId>;

/// Reads the classes.
///
/// # Errors
///
/// Returns an error when their table lacks a column.
pub(crate) fn read(source: &dyn Source) -> Result<List<Class>> {
    let place = |row: &ClassRow| Place::of(row.id.number(), row.next);
    reading::in_stored_order(source, &CLASSES, place, |origin, row| {
        (!row.is_deleted).then(|| Label {
            id: row.id,
            name: row.name.unwrap_or_default(),
            parent: row.parent,
            is_active: !row.is_hidden,
            origin,
        })
    })
}

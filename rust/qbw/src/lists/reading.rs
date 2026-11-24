//! The reading of lists that are alike.

use sa17::Source;

use crate::case;
use crate::damage::Damage;
use crate::error::Result;
use crate::list::{List, ListEntry, order};
use crate::origin::Origin;
use crate::tables::{StoredRow, Table};

/// Every row of `table` that can be read, or none when the source lacks it.
///
/// # Errors
///
/// Returns an error when the table lacks a column.
pub(super) fn rows<R: StoredRow>(
    source: &dyn Source,
    table: &Table<R>,
    damage: &mut Vec<Damage>,
) -> Result<Vec<(Origin, R)>> {
    Ok(match table.find(source)? {
        Some(table) => table.scan(damage).collect(),
        None => Vec::new(),
    })
}

/// Reads a list whose order the file does not keep. `entry` makes an entry of
/// a row that is not deleted.
///
/// # Errors
///
/// Returns an error when the table lacks a column.
pub(super) fn by_name<R: StoredRow, T: ListEntry>(
    source: &dyn Source,
    table: &Table<R>,
    entry: impl Fn(Origin, R) -> Option<T>,
) -> Result<List<T>> {
    let mut damage = Vec::new();
    let rows = rows(source, table, &mut damage)?.into_iter();
    let entries = rows.filter_map(|(origin, row)| entry(origin, row));
    Ok(sorted_by_name(entries.collect(), damage))
}

/// The list of `entries`, ordered by name.
pub(super) fn sorted_by_name<T: ListEntry>(mut entries: Vec<T>, damage: Vec<Damage>) -> List<T> {
    entries.sort_by_cached_key(|entry| case::folded(entry.name()));
    List::new(entries, damage)
}

/// Where a row is in the order of its list.
pub(super) struct Place {
    number: i32,
    /// The number of the row after it.
    next: Option<i32>,
}

impl Place {
    pub(super) const fn of(number: i32, next: Option<i32>) -> Self {
        Self { number, next }
    }
}

/// Reads a list whose order the file keeps, by the place of every row, deleted
/// ones among them. `entry` makes an entry of a row that is not deleted.
///
/// # Errors
///
/// Returns an error when the table lacks a column.
pub(super) fn in_stored_order<R: StoredRow, T: ListEntry>(
    source: &dyn Source,
    table: &Table<R>,
    place: impl Fn(&R) -> Place,
    entry: impl Fn(Origin, R) -> Option<T>,
) -> Result<List<T>> {
    let mut damage = Vec::new();
    let mut links = Vec::new();
    let mut placed = Vec::new();
    for (origin, row) in rows(source, table, &mut damage)? {
        let Place { number, next } = place(&row);
        links.extend(next.map(|next| (number, next)));
        placed.extend(entry(origin, row).map(|entry| (number, entry)));
    }
    order::sort(&mut placed, links, |&(number, _)| number);
    let entries = placed.into_iter().map(|(_, entry)| entry);
    Ok(List::new(entries.collect(), damage))
}

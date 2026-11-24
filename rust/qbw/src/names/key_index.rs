//! Where the rows of a long table are stored, by their keys.

use std::collections::HashMap;
use std::hash::Hash;

use sa17::{RowId, Source};

use crate::damage::Damage;
use crate::error::{Error, Result};
use crate::tables::{StoredRow, Table};

/// Where every row of a table is stored, by its key. The rows themselves stay
/// in the file, since a company has tens of thousands of old addresses.
#[derive(Debug)]
pub(crate) struct KeyIndex<K> {
    rows: HashMap<K, RowId>,
    damage: Vec<Damage>,
}

impl<K: Copy + Eq + Hash> KeyIndex<K> {
    /// Reads the keys of `table`, or none when the source lacks it.
    ///
    /// # Errors
    ///
    /// Returns an error when the table lacks a column.
    pub(super) fn read<R: StoredRow>(
        source: &dyn Source,
        table: &Table<R>,
        key: impl Fn(&R) -> K,
    ) -> Result<Self> {
        let mut damage = Vec::new();
        let rows = match table.find(source)? {
            Some(table) => table
                .scan(&mut damage)
                .map(|(origin, row)| (key(&row), origin.row))
                .collect(),
            None => HashMap::new(),
        };
        Ok(Self { rows, damage })
    }

    /// Where the row with `key` is, or `None` when there is none.
    ///
    /// # Errors
    ///
    /// Returns what could not be read of the table, when the row may be part
    /// of it.
    pub(crate) fn get(&self, key: K) -> Result<Option<RowId>> {
        match (self.rows.get(&key), self.damage.first()) {
            (None, Some(damage)) => Err(Error::Damaged(damage.clone())),
            (found, _) => Ok(found.copied()),
        }
    }
}

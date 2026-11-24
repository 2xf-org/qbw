//! Tables that rows are looked up in one at a time.

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

use sa17::{Columns, Source};

use crate::error::{Error, Result};
use crate::tables::{OpenTable, StoredRow, Table};

/// The tables of one source that were opened so far. Finding a table and its
/// columns by their names takes longer than looking a row up, and a form is read
/// with a few lookups.
#[derive(Debug, Default)]
pub(crate) struct OpenedTables {
    /// By the name of a table and what its rows are read as.
    found: Mutex<HashMap<(&'static str, TypeId), Found>>,
}

/// Where a table is, and the columns that are read of it.
#[derive(Clone, Debug)]
struct Found {
    id: u32,
    columns: Columns,
}

impl OpenedTables {
    /// The table of `source`, which has to be the same each time. `None` when the
    /// source has no such table.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Table::find`].
    pub(crate) fn find<'source, R: StoredRow + 'static>(
        &self,
        table: &Table<R>,
        source: &'source dyn Source,
    ) -> Result<Option<OpenTable<'source, R>>> {
        let key = (table.name(), TypeId::of::<R>());
        let found = self.found.lock().unwrap_or_else(PoisonError::into_inner);
        let known = found.get(&key).cloned();
        drop(found);
        let known = known.and_then(|found| {
            let definition = source.definition_by_id(found.id)?;
            Some(table.opened(source, definition, found.columns))
        });
        if known.is_some() {
            return Ok(known);
        }
        let opened = table.find(source)?;
        if let Some(opened) = &opened {
            let (id, columns) = (opened.id(), opened.columns());
            let mut found = self.found.lock().unwrap_or_else(PoisonError::into_inner);
            found.insert(key, Found { id, columns });
        }
        Ok(opened)
    }

    /// The table of `source` that every file has.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoTable`] when the source has no such table, and the
    /// errors of [`Self::find`].
    pub(crate) fn open<'source, R: StoredRow + 'static>(
        &self,
        table: &Table<R>,
        source: &'source dyn Source,
    ) -> Result<OpenTable<'source, R>> {
        let name = table.name();
        self.find(table, source)?
            .ok_or(Error::NoTable { table: name })
    }
}

//! Memorized transactions.

use sa17::Source;

use crate::error::Result;
use crate::ids::{MemorizedId, TransactionId};
use crate::list::{List, ListEntry, sealed};
use crate::lists::reading;
use crate::lists::reading::Place;
use crate::origin::Origin;
use crate::tables::lists::{MEMORIZED, MemorizedRow};

/// A transaction that is kept to be made again, or a group of them.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Memorized {
    /// What the entry is known by.
    pub id: MemorizedId,
    /// What the company calls it.
    pub name: String,
    /// The group it is in. Unchecked.
    pub parent: Option<MemorizedId>,
    /// The transaction that is made again. A group has none.
    pub transaction: Option<TransactionId>,
    /// Whether it is in use.
    pub is_active: bool,
    /// The stored row.
    pub origin: Origin,
}

impl sealed::Sealed for Memorized {}

impl ListEntry for Memorized {
    type Id = MemorizedId;

    fn id(&self) -> MemorizedId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn parent(&self) -> Option<MemorizedId> {
        self.parent
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn origin(&self) -> Origin {
        self.origin
    }
}

/// Reads the memorized transactions.
///
/// # Errors
///
/// Returns an error when their table lacks a column.
pub(crate) fn read(source: &dyn Source) -> Result<List<Memorized>> {
    let place = |row: &MemorizedRow| Place::of(row.id.number(), row.next);
    reading::in_stored_order(source, &MEMORIZED, place, |origin, row| {
        (!row.is_deleted).then(|| Memorized {
            id: row.id,
            name: row.name.unwrap_or_default(),
            parent: row.parent,
            transaction: row.transaction,
            is_active: !row.is_hidden,
            origin,
        })
    })
}

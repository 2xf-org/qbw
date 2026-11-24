//! A line as it is stored.

use crate::chain::Linked;
use crate::ids::LineId;
use crate::origin::Origin;
use crate::tables::lines::LineRow;

/// A line of a transaction as it is stored.
#[derive(Debug)]
pub(super) struct StoredLine {
    pub(super) row: LineRow,
    pub(super) origin: Origin,
}

impl Linked for StoredLine {
    fn id(&self) -> LineId {
        self.row.id
    }

    fn next(&self) -> Option<LineId> {
        self.row.next
    }
}

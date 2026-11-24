//! Where a record came from.

use std::fmt;

use sa17::RowId;

/// The stored row that a record came from.
///
/// Every record has one, and
/// [`Company::stored_row`](crate::Company::stored_row) goes down to the row
/// itself, with the columns that the record leaves out. An origin means
/// something only in the company it came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct Origin {
    /// The name of the table.
    pub table: &'static str,
    /// Which row of the table.
    pub row: RowId,
}

impl Origin {
    pub(crate) const fn new(table: &'static str, row: RowId) -> Self {
        Self { table, row }
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "table {}, {}", self.table, self.row)
    }
}

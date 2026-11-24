//! What could not be read.

use std::fmt;

use sa17::{Error as DatabaseError, RowId, ValueProblem};

use crate::origin::Origin;

/// A part of the file that could not be read, or that is not what it has to be.
///
/// Reading goes on without it, and whatever was worked out meanwhile says so
/// with a [`LeftOut`](crate::LeftOut).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Damage {
    /// The name of the table.
    pub table: &'static str,
    /// Which row of the table, when it is known.
    pub row: Option<RowId>,
    /// What is wrong.
    pub problem: Problem,
}

/// Where something could not be read, as far as it is known. Parts that read the
/// same table lack the same places.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Place {
    table: &'static str,
    row: Option<RowId>,
    /// The page, where none of it could be read.
    pub(crate) page: Option<u32>,
}

impl Damage {
    pub(crate) const fn new(table: &'static str, row: Option<RowId>, problem: Problem) -> Self {
        Self {
            table,
            row,
            problem,
        }
    }

    /// Where it is.
    pub(crate) fn place(&self) -> Place {
        let page = match self.problem {
            Problem::Page { page, .. } => Some(page),
            _ => None,
        };
        Place {
            table: self.table,
            row: self.row,
            page,
        }
    }

    /// That a row before the one at `origin` has the same key.
    pub(crate) const fn repeated(origin: Origin) -> Self {
        Self::new(origin.table, Some(origin.row), Problem::Repeated)
    }

    /// What `error`, of a scan of `table`, leaves unread.
    pub(crate) fn of_scan(table: &'static str, error: &DatabaseError) -> Self {
        match error {
            DatabaseError::TablePage { page, problem, .. } => {
                let (page, reason) = (*page, problem.to_string());
                Self::new(table, None, Problem::Page { page, reason })
            }
            DatabaseError::Row {
                address, problem, ..
            } => {
                let row = RowId::Stored(*address);
                let reason = problem.to_string();
                Self::new(table, Some(row), Problem::Row { reason })
            }
            error => {
                let reason = error.to_string();
                Self::new(table, None, Problem::Row { reason })
            }
        }
    }
}

impl fmt::Display for Damage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            table,
            row,
            problem,
        } = self;
        match row {
            Some(row) => write!(formatter, "table {table}, {row}: {problem}"),
            None => write!(formatter, "table {table}: {problem}"),
        }
    }
}

/// What is wrong with a part of the file.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum Problem {
    /// A page cannot be read, and with it every row it holds.
    #[error("page {page}: {reason}")]
    Page {
        /// The page's position in the file.
        page: u32,
        /// What the database reader found.
        reason: String,
    },
    /// A row cannot be read.
    #[error("{reason}")]
    Row {
        /// What the database reader found.
        reason: String,
    },
    /// A column of the row does not hold what it has to.
    #[error("column {column}: {found}")]
    Column {
        /// The column's name.
        column: &'static str,
        /// How its value differs.
        found: ValueProblem,
    },
    /// A row before it has the same key. It counts once.
    #[error("a row before it has the same key")]
    Repeated,
    /// What a line of an item in stock posts, whose line cannot be read.
    #[error("the line that it belongs to cannot be read")]
    NoLine,
    /// The header of a transaction, which has lines.
    #[error("the transaction's header cannot be read")]
    Header,
    /// The lines of a transaction are not chained from its header to the last.
    #[error("the chain of the transaction's lines is broken")]
    LineChain,
    /// A transaction has no line of its own side.
    #[error("the transaction has no line for its own side")]
    OwnLine,
    /// A transaction's lines are in more than one currency.
    #[error("the transaction's lines are in more than one currency")]
    Currencies,
}

#[cfg(test)]
mod tests {
    use sa17::{PageProblem, RowAddress, RowProblem};

    use super::*;

    #[test]
    fn damage_names_the_row_or_the_page() {
        let row = DatabaseError::Row {
            table: "lines".to_owned(),
            address: RowAddress::new(12, 3),
            problem: RowProblem::Columns,
        };
        let damage = Damage::of_scan("lines", &row);
        assert_eq!(damage.row, Some(RowId::Stored(RowAddress::new(12, 3))));
        let expected = format!("table lines, page 12, slot 3: {}", RowProblem::Columns);
        assert_eq!(damage.to_string(), expected);

        let page = DatabaseError::TablePage {
            table: "lines".to_owned(),
            page: 12,
            problem: PageProblem::Directory,
        };
        let damage = Damage::of_scan("lines", &page);
        assert!(damage.row.is_none());
        assert!(matches!(damage.problem, Problem::Page { page: 12, .. }));

        let checksum = DatabaseError::TablePage {
            table: "lines".to_owned(),
            page: 13,
            problem: PageProblem::Checksum {
                stored: 1,
                computed: 2,
            },
        };
        let damage = Damage::of_scan("lines", &checksum);
        let expected = "table lines: page 13: it fails its checksum";
        assert_eq!(damage.to_string(), expected);
    }
}

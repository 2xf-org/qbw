//! Where the lines of each transaction are.

use sa17::{RowId, Source};

use crate::damage::Damage;
use crate::error::Result;
use crate::ids::TransactionId;
use crate::tables::Form;
use crate::tables::lines::LineKeyRow;

/// Where the lines of each transaction of one form are.
#[derive(Debug)]
pub(crate) struct LineIndex {
    /// Ordered by transaction.
    lines: Vec<Place>,
}

#[derive(Debug)]
struct Place {
    transaction: TransactionId,
    row: RowId,
}

impl LineIndex {
    /// Reads where the lines of `form` are.
    ///
    /// A line that cannot be read is of a transaction that nobody knows. The
    /// chain of that transaction's other lines shows that one is lost.
    ///
    /// # Errors
    ///
    /// Returns an error when the table lacks a column that it has in every
    /// file.
    pub(crate) fn read(source: &dyn Source, form: Form) -> Result<Self> {
        let mut lines: Vec<Place> = Vec::new();
        if let Some(table) = form.lines_in::<LineKeyRow>(source)? {
            let mut damage: Vec<Damage> = Vec::new();
            let rows = table.scan(&mut damage);
            lines.extend(rows.map(|(origin, row)| Place {
                transaction: row.transaction,
                row: origin.row,
            }));
        }
        lines.sort_by_key(|line| line.transaction);
        Ok(Self { lines })
    }

    /// The rows of the lines of `transaction`.
    pub(crate) fn rows(&self, transaction: TransactionId) -> impl ExactSizeIterator<Item = RowId> {
        let start = self.before(|of| of < transaction);
        let end = self.before(|of| of <= transaction);
        let lines = self.lines.get(start..end).unwrap_or_default();
        lines.iter().map(|line| line.row)
    }

    /// The number of lines whose transaction `is_before`.
    fn before(&self, is_before: impl Fn(TransactionId) -> bool) -> usize {
        self.lines
            .partition_point(|line| is_before(line.transaction))
    }
}

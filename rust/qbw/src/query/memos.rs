//! The memos of transactions.

use sa17::Source;

use crate::damage::Damage;
use crate::error::{Error, Result};
use crate::tables::headers::MemoRow;
use crate::tables::{Form, OpenTable};
use crate::transactions::Header;

/// The memos of transactions, read one by one.
pub(super) struct Memos<'company> {
    source: &'company dyn Source,
    /// The table of the last memo.
    table: Option<(Form, OpenTable<'company, MemoRow>)>,
    pub(super) damage: Vec<Damage>,
}

impl<'company> Memos<'company> {
    pub(super) fn new(source: &'company dyn Source) -> Self {
        Self {
            source,
            table: None,
            damage: Vec::new(),
        }
    }

    /// The memo of a transaction. One that cannot be read is nothing, and what
    /// is wrong is among the damage.
    pub(super) fn get(&mut self, header: &Header) -> Result<Option<String>> {
        let table = match self.table.take() {
            Some((form, table)) if form == header.form => table,
            _ => header.form.headers().open(self.source)?,
        };
        let (_, table) = self.table.insert((header.form, table));
        match table.fetch(header.row) {
            Ok(row) => Ok(row.and_then(|row| row.memo)),
            Err(Error::Damaged(damage)) => {
                self.damage.push(damage);
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }
}

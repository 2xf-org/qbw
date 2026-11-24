//! What a line shows that the ledger does not keep.

use std::collections::HashMap;

use sa17::Source;

use crate::damage::Damage;
use crate::error::{Error, Result};
use crate::ids::{LineId, TransactionId};
use crate::ledger::Ledger;
use crate::left_out::LeftOut;
use crate::origin::Origin;
use crate::tables::OpenTable;
use crate::tables::postings;
use crate::tables::postings::RegisterLineRow;

/// Reads lines one by one.
pub(super) struct Lines<'company> {
    source: &'company dyn Source,
    ledger: &'company Ledger,
    tables: HashMap<&'static str, Option<OpenTable<'company, RegisterLineRow>>>,
    damage: Vec<Damage>,
}

impl<'company> Lines<'company> {
    pub(super) fn new(source: &'company dyn Source, ledger: &'company Ledger) -> Self {
        Self {
            source,
            ledger,
            tables: HashMap::new(),
            damage: Vec::new(),
        }
    }

    /// The line that posted what is in the row at `origin`: a line of a
    /// transaction. `None` when it cannot be read.
    ///
    /// # Errors
    ///
    /// Returns an error when the table of the line lacks a column.
    pub(super) fn read(
        &mut self,
        transaction: TransactionId,
        line: LineId,
        origin: Origin,
    ) -> Result<Option<RegisterLineRow>> {
        // What a line posts besides is a row of another table.
        let mut of_transaction = self.ledger.transaction_entries(transaction);
        let line = of_transaction.find(|entry| entry.is_line() && entry.line == line);
        let Origin { table, row } = line.map_or(origin, |line| self.ledger.origin(line));
        if !self.tables.contains_key(table) {
            let found = postings::register_lines(table).find(self.source)?;
            self.tables.insert(table, found);
        }
        let Some(Some(table)) = self.tables.get(table) else {
            return Ok(None);
        };
        match table.fetch(row) {
            Err(Error::Damaged(damage)) => {
                self.damage.push(damage);
                Ok(None)
            }
            read => read,
        }
    }

    /// How much could not be read. A line that posts several times counts once.
    pub(super) fn left_out(&self) -> LeftOut {
        LeftOut::of_places(self.damage.iter().map(Damage::place))
    }
}

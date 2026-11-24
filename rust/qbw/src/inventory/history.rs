//! The history that QuickBooks keeps of every item in stock.

use chrono::NaiveDate;
use sa17::{RowId, Source};

use crate::amount::Amount;
use crate::as_of::AsOf;
use crate::damage::Damage;
use crate::error::Result;
use crate::ids::{ItemId, LineId, TransactionId};
use crate::left_out::LeftOut;
use crate::origin::Origin;
use crate::tables::stock::{HISTORY, HistoryRow};

/// What there was of every item in stock after each line that changed it.
#[derive(Debug)]
pub(crate) struct History {
    /// By item and in order, each item's start first.
    entries: Box<[Entry]>,
    damage: Vec<Damage>,
}

/// What there was of an item after a line of it.
#[derive(Debug)]
pub(crate) struct Entry {
    item: ItemId,
    sequence: i32,
    pub(crate) date: NaiveDate,
    /// Nothing at the start of an item.
    pub(crate) cause: Option<Cause>,
    pub(crate) held: Held,
    row: RowId,
}

/// The line that changed what there is of an item.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Cause {
    pub(crate) transaction: TransactionId,
    pub(crate) line: LineId,
}

/// What there is of an item, and what it is worth.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Held {
    pub(crate) on_hand: Amount,
    pub(crate) value: Amount,
    pub(crate) average_cost: Amount,
}

impl History {
    /// Reads the history.
    ///
    /// # Errors
    ///
    /// Returns an error when the file lacks the table or a column of it.
    pub(crate) fn read(source: &dyn Source) -> Result<Self> {
        let mut damage = Vec::new();
        let table = HISTORY.open(source)?;
        let mut entries: Vec<Entry> = table
            .scan(&mut damage)
            .map(|(origin, row)| Entry::new(origin, &row))
            .collect();
        // Of rows with one key the first stays.
        entries.sort_by_key(|entry| (entry.item, entry.sequence));
        let key = |entry: &Entry| (entry.item, entry.sequence);
        let repeated = entries.windows(2).filter_map(|pair| match pair {
            [before, entry] if key(before) == key(entry) => Some(entry.origin()),
            _ => None,
        });
        damage.extend(repeated.map(Damage::repeated));
        entries.dedup_by_key(|entry| key(entry));
        // Orders and what is pending have rows that change nothing.
        entries.dedup_by(|entry, before| entry.item == before.item && entry.held == before.held);
        Ok(Self {
            entries: entries.into_boxed_slice(),
            damage,
        })
    }

    /// What there is of `item`, unless nothing is known of it.
    pub(crate) fn held(&self, item: ItemId) -> Option<Held> {
        self.of(item).last().map(|entry| entry.held)
    }

    /// What there was of `item` at a moment.
    pub(crate) fn held_at(&self, item: ItemId, as_of: AsOf) -> Option<Held> {
        let AsOf::Date(day) = as_of else {
            return self.held(item);
        };
        let entries = self.of(item);
        let later = entries.partition_point(|entry| entry.date <= day);
        entries.get(..later)?.last().map(|entry| entry.held)
    }

    /// The history of `item`, in order.
    pub(crate) fn of(&self, item: ItemId) -> &[Entry] {
        let start = self.entries.partition_point(|entry| entry.item < item);
        let rest = self.entries.get(start..).unwrap_or_default();
        let own = rest.partition_point(|entry| entry.item == item);
        rest.get(..own).unwrap_or_default()
    }

    /// How much could not be read.
    pub(crate) fn damage(&self) -> &[Damage] {
        &self.damage
    }

    pub(crate) fn left_out(&self) -> LeftOut {
        LeftOut::of(&self.damage)
    }
}

impl Entry {
    fn new(origin: Origin, row: &HistoryRow) -> Self {
        let cause = row.transaction.zip(row.line);
        Self {
            item: row.item,
            sequence: row.sequence,
            date: row.date,
            cause: cause.map(|(transaction, line)| Cause { transaction, line }),
            held: Held {
                on_hand: row.on_hand.unwrap_or_default(),
                value: row.value.unwrap_or_default(),
                average_cost: row.average_cost.unwrap_or_default(),
            },
            row: origin.row,
        }
    }

    pub(crate) fn origin(&self) -> Origin {
        Origin::new(HISTORY.name(), self.row)
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;

    #[test]
    fn an_entry_is_small() {
        // The largest of the books that were looked at has 113 000.
        assert!(mem::size_of::<Entry>() <= 64);
    }
}

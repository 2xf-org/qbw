//! What lists of transactions show of their lines.

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use sa17::Source;

use crate::accounts::Account;
use crate::damage::Damage;
use crate::error::Result;
use crate::ids::{AccountId, CurrencyId, TransactionId};
use crate::list::List;
use crate::side::Side;
use crate::tables::lines::{OrderOrTransfer, OrderOrTransferRow, SummaryLineRow};
use crate::tables::{Form, OpenTable, StoredRow};
use crate::transactions::left::Left;

/// The own line of each transaction of one form, and what is left of it.
#[derive(Debug)]
pub(crate) struct OwnLines {
    /// The currency that the books are kept in.
    home: CurrencyId,
    /// The accounts for what customers owe and what vendors are owed.
    owed: Vec<(AccountId, Side)>,
    lines: HashMap<TransactionId, SummaryLineRow>,
    /// What is left of the lines that are not a transaction's own.
    left_of_others: HashMap<TransactionId, Left>,
    /// What is shown besides of orders and transfers.
    orders_and_transfers: HashMap<TransactionId, OrderOrTransfer>,
    damage: Vec<Damage>,
}

impl OwnLines {
    /// Reads the own lines of `form` of a company that keeps its books in `home`.
    ///
    /// # Errors
    ///
    /// Returns an error when the table lacks a column that it has in every
    /// file.
    pub(crate) fn read(
        source: &dyn Source,
        form: Form,
        accounts: &List<Account>,
        home: CurrencyId,
    ) -> Result<Self> {
        let owed = accounts.iter().filter_map(|account| {
            let side = Side::of_type(account.account_type)?;
            Some((account.id, side))
        });
        let mut read = Self {
            home,
            owed: owed.collect(),
            lines: HashMap::new(),
            left_of_others: HashMap::new(),
            orders_and_transfers: HashMap::new(),
            damage: Vec::new(),
        };
        if form.has_orders_or_transfers() {
            let table = form.lines_in::<OrderOrTransferRow>(source)?;
            read.add_lines(table, |read, row| {
                if row.is_own {
                    let orders_and_transfers = &mut read.orders_and_transfers;
                    let own = orders_and_transfers.entry(row.transaction);
                    own.or_insert_with(|| OrderOrTransfer::from(&row));
                }
                SummaryLineRow::from(&row)
            });
        } else {
            let table = form.lines_in::<SummaryLineRow>(source)?;
            read.add_lines(table, |_, line| line);
        }
        Ok(read)
    }

    /// Adds the lines of a table, of which `line` gives what every list shows.
    fn add_lines<R: StoredRow>(
        &mut self,
        table: Option<OpenTable<'_, R>>,
        line: impl Fn(&mut Self, R) -> SummaryLineRow,
    ) {
        let Some(table) = table else {
            return;
        };
        let mut damage = Vec::new();
        let mut repeated = Vec::new();
        let mut seen = HashSet::new();
        for (origin, row) in table.scan(&mut damage) {
            let line = line(self, row);
            if line.is_own {
                match self.lines.entry(line.transaction) {
                    Entry::Vacant(place) => {
                        place.insert(line);
                        continue;
                    }
                    Entry::Occupied(first) if first.get().id == line.id => {
                        repeated.push(Damage::repeated(origin));
                        continue;
                    }
                    // Something can be left of a second own line, too.
                    Entry::Occupied(_) => {}
                }
            }
            let Some(side) = self.side(&line) else {
                continue;
            };
            if !seen.insert(line.id) {
                repeated.push(Damage::repeated(origin));
                continue;
            }
            let left = Left::of_line(&line, side, self.home);
            self.left_of_others
                .entry(line.transaction)
                .and_modify(|before| *before = before.with(left))
                .or_insert(left);
        }
        self.damage.append(&mut damage);
        self.damage.append(&mut repeated);
    }

    /// The side of the account of a line that something can be left of.
    fn side(&self, line: &SummaryLineRow) -> Option<Side> {
        let account = line.account?;
        let mut owed = self.owed.iter();
        let found = owed.find(|(owed, _)| *owed == account);
        found.map(|&(_, side)| side)
    }

    pub(crate) fn get(&self, transaction: TransactionId) -> Option<&SummaryLineRow> {
        self.lines.get(&transaction)
    }

    /// What is shown besides of the own line of an order or a transfer.
    pub(crate) fn order_or_transfer(&self, transaction: TransactionId) -> Option<&OrderOrTransfer> {
        self.orders_and_transfers.get(&transaction)
    }

    /// What is left of a transaction, when it has a line that something can be
    /// left of.
    pub(crate) fn left(&self, transaction: TransactionId) -> Option<Left> {
        let others = self.left_of_others.get(&transaction).copied();
        let own = self.get(transaction).and_then(|line| {
            let side = self.side(line)?;
            Some(Left::of_line(line, side, self.home))
        });
        match (own, others) {
            (Some(own), Some(others)) => Some(own.with(others)),
            (own, others) => own.or(others),
        }
    }

    /// What could not be read of the lines, which may be an own one.
    pub(crate) fn damage(&self) -> &[Damage] {
        &self.damage
    }
}

//! What the lines of transactions say about items.

use std::collections::{HashMap, HashSet};

use sa17::Source;

use crate::amount::{Amount, Total};
use crate::codes::TransactionKind;
use crate::damage::Damage;
use crate::error::Result;
use crate::ids::{ItemId, LineId, SiteId, TransactionId};
use crate::inventory::quantity;
use crate::left_out::LeftOut;
use crate::posting_state::PostingState;
use crate::tables::stock::ItemLineRow;
use crate::tables::{Form, stock, transactions};

/// What the lines of every transaction say about items, added up.
#[derive(Debug, Default)]
pub(crate) struct ItemLines {
    items: HashMap<ItemId, ItemSums>,
    /// The lines of orders that something is open of, form by form and as they
    /// are stored.
    open: Vec<OpenLine>,
    damage: Vec<Damage>,
    /// Tables of lines that this version does not read.
    unread_tables: u32,
}

/// What the lines of an item add up to.
#[derive(Debug, Default)]
pub(crate) struct ItemSums {
    /// What posted lines put into the stock, less what they took out.
    pub(crate) on_hand: Total,
    /// The same by site and by bin. A site's has that of its bins.
    pub(crate) sites: HashMap<SiteId, Total>,
    /// What is left of purchase orders.
    pub(crate) on_order: Total,
    /// What is left of sales orders.
    pub(crate) on_sales_orders: Total,
    /// What pending builds will put into the stock.
    pub(crate) to_be_built: Total,
    /// What pending builds will take out of it.
    pub(crate) required_for_builds: Total,
}

/// A line of an order that something is open of.
#[derive(Clone, Copy, Debug)]
pub(crate) struct OpenLine {
    pub(crate) line: LineId,
    pub(crate) order: TransactionId,
    pub(crate) kind: TransactionKind,
    pub(crate) ordered: Amount,
    pub(crate) open: Amount,
}

impl ItemLines {
    /// Reads every line of every transaction that can have an item.
    ///
    /// # Errors
    ///
    /// Returns an error when a table lacks a column that it has in every file.
    pub(crate) fn read(source: &dyn Source) -> Result<Self> {
        Ok(Self {
            unread_tables: transactions::unread(source),
            ..Self::of_forms(source, Form::all())?
        })
    }

    /// Reads the lines of the forms that orders of `kind` can be on.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::read`].
    pub(crate) fn of_orders(source: &dyn Source, kind: TransactionKind) -> Result<Self> {
        Self::of_forms(source, Form::all().filter(|form| form.can_hold(kind)))
    }

    fn of_forms(source: &dyn Source, forms: impl Iterator<Item = Form>) -> Result<Self> {
        let mut lines = Self::default();
        let mut damage = Vec::new();
        let mut repeated = Vec::new();
        let mut seen = HashSet::new();
        for table in stock::item_lines(source, forms)? {
            for (origin, line) in table.scan(&mut damage) {
                if line.item.is_none() || seen.insert(line.id) {
                    lines.add(&line);
                } else {
                    repeated.push(Damage::repeated(origin));
                }
            }
        }
        damage.append(&mut repeated);
        lines.damage = damage;
        Ok(lines)
    }

    /// What the lines of `item` add up to, unless it has none.
    pub(crate) fn of(&self, item: ItemId) -> Option<&ItemSums> {
        self.items.get(&item)
    }

    /// The lines of orders of `kind` that something is open of.
    pub(crate) fn open(&self, kind: TransactionKind) -> impl Iterator<Item = &OpenLine> {
        self.open.iter().filter(move |line| line.kind == kind)
    }

    /// How much could not be read.
    pub(crate) fn damage(&self) -> &[Damage] {
        &self.damage
    }

    pub(crate) fn left_out(&self) -> LeftOut {
        LeftOut::of(&self.damage).with(LeftOut::of_unread_tables(self.unread_tables))
    }

    fn add(&mut self, line: &ItemLineRow) {
        let Some(item) = line.item else {
            return;
        };
        let state = line.flags().state();
        if state == PostingState::Memorized {
            return;
        }
        if let Some(open) = quantity::open(line).filter(|_| !line.is_own) {
            self.add_open(item, line, open);
        } else if state == PostingState::Posted {
            self.sums(item).add_posted(line);
        } else if line.kind == TransactionKind::AssemblyBuild {
            self.sums(item).add_pending_build(line);
        }
    }

    fn add_open(&mut self, item: ItemId, line: &ItemLineRow, open: Amount) {
        let sums = self.sums(item);
        match line.kind {
            TransactionKind::PurchaseOrder => sums.on_order += open.into(),
            _ => sums.on_sales_orders += open.into(),
        }
        self.open.push(OpenLine {
            line: line.id,
            order: line.transaction,
            kind: line.kind,
            ordered: quantity::ordered(line),
            open,
        });
    }

    fn sums(&mut self, item: ItemId) -> &mut ItemSums {
        self.items.entry(item).or_default()
    }
}

impl ItemSums {
    fn add_posted(&mut self, line: &ItemLineRow) {
        let moved = Total::from(quantity::moved(line));
        self.on_hand += moved;
        for site in [line.site, line.bin].into_iter().flatten() {
            *self.sites.entry(site).or_default() += moved;
        }
    }

    /// Adds a line of a build that is pending: the assembly on its own line,
    /// and what it is made of on the others.
    fn add_pending_build(&mut self, line: &ItemLineRow) {
        let moved = Total::from(quantity::moved(line));
        if line.is_own {
            self.to_be_built += moved;
        } else {
            self.required_for_builds += -moved;
        }
    }
}

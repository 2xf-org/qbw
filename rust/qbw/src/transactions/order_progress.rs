//! How far an order is filled.

use sa17::Decimal;

use crate::amount::{Amount, Total};
use crate::codes::{ItemKind, TransactionKind};
use crate::tables::lines::{LineRow, OrderOrTransfer};
use crate::transactions::form_side::FormSide;

/// How far a line of a purchase order or a sales order is filled, or the whole
/// order.
///
/// An order is filled by the bills and receipts that bring its goods, or by
/// the invoices that charge for them. More can come than was ordered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OrderProgress {
    /// The quantity that is still to come, below zero when more came than was
    /// ordered. A line that does not say how many is of one. `None` for a
    /// discount that does not say, and for a whole order.
    pub open: Option<Decimal>,
    /// The quantity that was received or invoiced.
    pub filled: Option<Decimal>,
    /// Whether nothing more is to come: the quantity did, or somebody closed
    /// the line or the order.
    pub is_done: bool,
    /// Whether somebody closed it by hand.
    pub is_closed: bool,
}

/// What the file keeps of a line of a transaction that may be an order.
#[derive(Clone, Copy, Debug)]
pub(crate) struct OrderLine {
    pub(crate) kind: TransactionKind,
    pub(crate) quantity: Option<Amount>,
    pub(crate) open_quantity: Option<Amount>,
    pub(crate) is_received: bool,
    pub(crate) is_invoiced: bool,
}

impl OrderLine {
    /// Whether nothing more is to come. `None` on what is no order.
    pub(crate) const fn is_done(self) -> Option<bool> {
        match self.kind {
            TransactionKind::PurchaseOrder => Some(self.is_received),
            TransactionKind::SalesOrder => Some(self.is_invoiced),
            _ => None,
        }
    }

    /// How many of its item it asks for. A line that does not say asks for one.
    pub(crate) fn ordered(self) -> Amount {
        let shown = |quantity| self.shown(quantity);
        self.quantity.map_or(Amount::ONE, shown)
    }

    /// How many of them are left: all of them, unless the file says how many.
    pub(crate) fn open(self) -> Amount {
        let shown = |quantity| self.shown(quantity);
        self.open_quantity.map_or_else(|| self.ordered(), shown)
    }

    fn shown(self, quantity: Amount) -> Amount {
        FormSide::of(self.kind).shown(quantity)
    }
}

impl OrderProgress {
    /// How far a line of a transaction of `kind` is filled, whose item is of kind
    /// `item`. `None` on what is no order.
    pub(super) fn of_line(
        row: &LineRow,
        kind: TransactionKind,
        item: Option<ItemKind>,
    ) -> Option<Self> {
        let whole = Self::of_whole(&row.into(), kind)?;
        if row.quantity.is_none() && item == Some(ItemKind::Discount) {
            return Some(whole);
        }
        let line = OrderLine {
            kind,
            quantity: row.quantity,
            open_quantity: row.open_quantity,
            is_received: row.is_received,
            is_invoiced: row.is_invoiced,
        };
        let filled = Total::from(line.ordered()) - line.open().into();
        Some(Self {
            open: Some(line.open().quantity()),
            filled: Some(filled.quantity()),
            ..whole
        })
    }

    /// How far the whole of a transaction of `kind` is filled, by its own line.
    /// `None` on what is no order.
    pub(crate) fn of_whole(row: &OrderOrTransfer, kind: TransactionKind) -> Option<Self> {
        let line = OrderLine {
            kind,
            quantity: None,
            open_quantity: None,
            is_received: row.is_received,
            is_invoiced: row.is_invoiced,
        };
        Some(Self {
            open: None,
            filled: None,
            is_done: line.is_done()?,
            is_closed: row.is_closed,
        })
    }
}

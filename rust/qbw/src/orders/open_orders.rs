//! Orders that are not filled yet.

use std::collections::BTreeMap;
use std::mem;

use crate::ids::LineId;
use crate::inventory::OpenLine;
use crate::left_out::LeftOut;
use crate::orders::OpenOrderLine;
use crate::side::Side;
use crate::transactions::{LineRole, Transaction};

/// The open lines of an order, by line.
pub(crate) type OpenLines<'lines> = BTreeMap<LineId, &'lines OpenLine>;

/// The lines of orders that something is left of.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct OpenOrders {
    /// Whether they are what customers ordered, or what is ordered from
    /// vendors.
    pub side: Side,
    /// The lines by the date of their order, and those of one order as the
    /// form has them.
    pub lines: Vec<OpenOrderLine>,
    /// How much could not be read. A header that is lost counts once, whatever
    /// its order has open.
    pub left_out: LeftOut,
}

impl OpenOrders {
    /// No orders on `side`.
    pub(crate) fn new(side: Side, left_out: LeftOut) -> Self {
        Self {
            side,
            lines: Vec::new(),
            left_out,
        }
    }

    /// Adds the lines of `order` that are in `open`. What repeats the sum of other
    /// lines is not among them.
    pub(crate) fn add(&mut self, mut order: Transaction, mut open: OpenLines<'_>) {
        for line in mem::take(&mut order.lines) {
            let Some(open) = open.remove(&line.id) else {
                continue;
            };
            if line.role == LineRole::Item {
                self.lines.push(OpenOrderLine::new(&order, line, open));
            }
        }
        // What is still there can be read no further than what is left of it.
        let lost = open.into_values().map(|_| 1).sum();
        self.lose(LeftOut::of_rows(lost));
    }

    /// Adds that so much cannot be read of orders that something is left of.
    pub(crate) fn lose(&mut self, left_out: LeftOut) {
        self.left_out = self.left_out.with(left_out);
    }

    /// The same, with the lines in order.
    pub(crate) fn sorted(mut self) -> Self {
        self.lines.sort_by_key(|line| line.date);
        self
    }
}

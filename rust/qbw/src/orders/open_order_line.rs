//! A line of an order that something is left of.

use chrono::NaiveDate;
use sa17::Decimal;

use crate::amount::Total;
use crate::ids::{NameId, TransactionId};
use crate::inventory::OpenLine;
use crate::money::{CENTS, Money};
use crate::price::Price;
use crate::transactions::{Line, Transaction};

/// A line of a sales order or a purchase order that something is left of.
///
/// A line counts as open until it is filled or somebody closes it. It can be
/// open in an order that is done, and QuickBooks counts it as on order then,
/// too.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct OpenOrderLine {
    /// The order.
    pub transaction: TransactionId,
    /// The date of the order.
    pub date: NaiveDate,
    /// The number of the order.
    pub number: Option<String>,
    /// The customer or the vendor.
    pub name: Option<NameId>,
    /// Whether the order itself is done, or somebody closed it.
    pub is_order_done: bool,
    /// How many were ordered. A line that does not say is of one.
    pub ordered: Decimal,
    /// How many of them were received or invoiced.
    pub filled: Decimal,
    /// How many are left.
    pub open: Decimal,
    /// The part of the line's amount that is left, in the order's currency: all
    /// of it when all of the quantity is. Else it is inferred, as what is left
    /// at the line's price, to the cent, and `None` on a line without one.
    pub open_amount: Option<Money>,
    /// The line, with its item, its price and what it says.
    pub line: Line,
}

impl OpenOrderLine {
    /// The line `line` of `order`, of which `open` is left.
    pub(crate) fn new(order: &Transaction, line: Line, open: &OpenLine) -> Self {
        let progress = order.own_line.as_ref().and_then(|own| own.order);
        Self {
            transaction: order.id,
            date: order.date,
            number: order.number.clone(),
            name: order.name,
            is_order_done: progress.is_some_and(|order| order.is_done || order.is_closed),
            ordered: open.ordered.quantity(),
            filled: (Total::from(open.ordered) - open.open.into()).quantity(),
            open: open.open.quantity(),
            open_amount: left_of(&line, open),
            line,
        }
    }
}

/// What is left of the amount of `line`.
fn left_of(line: &Line, open: &OpenLine) -> Option<Money> {
    if open.open == open.ordered {
        return line.amount;
    }
    let Some(Price::Each(price)) = line.rate else {
        return None;
    };
    let open = open.open.quantity();
    let left = open.checked_mul_round(price.amount(), CENTS)?;
    Some(Money::new(left, price.currency()))
}

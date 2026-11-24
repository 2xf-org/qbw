//! The stock of a company, and the orders that are not filled yet.

use std::collections::BTreeMap;

use sa17::Source;

use crate::as_of::AsOf;
use crate::company::Company;
use crate::error::{Error, Result};
use crate::ids::{ItemId, TransactionId};
use crate::inventory;
use crate::inventory::{History, ItemLines, Movements, OpenLine, Stock, StockReader, Valuation};
use crate::left_out::LeftOut;
use crate::orders::{OpenLines, OpenOrders};
use crate::side::Side;
use crate::tables::Form;
use crate::transactions::ShownHeaders;

/// # Stock and orders
impl<S: Source> Company<S> {
    /// What there is in stock now, with what is ordered and planned.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::items`] and [`Self::preferences`], and an
    /// error when the file lacks a column of the stock's history or of the
    /// lines.
    pub fn stock(&self) -> Result<Stock> {
        let reader = StockReader {
            preferences: self.preferences()?,
            items: self.items()?,
            sites: self.sites()?,
            history: self.stock_history()?,
            lines: self.item_lines()?,
        };
        Ok(reader.stock())
    }

    /// What there was in stock at a moment, and what it was worth. What was on
    /// order or planned on a day is not known.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::items`] and [`Self::preferences`], and an
    /// error when the file lacks the stock's history or a column of it.
    pub fn valuation(&self, as_of: impl Into<AsOf>) -> Result<Valuation> {
        let home = self.preferences()?.home_currency;
        let (items, history) = (self.items()?, self.stock_history()?);
        Ok(inventory::valuation(items, history, home, as_of.into()))
    }

    /// Every line that changed how much there is of `item`, or what that is
    /// worth. `None` for an item that is not kept in stock, or that there is not.
    ///
    /// # Errors
    ///
    /// Returns an error when the file lacks the stock's history, a column of it
    /// or of the headers, or when the company's currency cannot be understood.
    pub fn movements(&self, item: ItemId) -> Result<Option<Movements>> {
        let known = self.items()?.get(item);
        if known.is_none_or(|item| !item.kind.is_stocked()) {
            return Ok(None);
        }
        let home = self.preferences()?.home_currency;
        let (history, headers) = (self.stock_history()?, self.header_index()?);
        let movements = inventory::movements(item, history, headers, home);
        Ok(Some(movements))
    }

    /// The lines of orders that something is left of: on the side of
    /// [`Side::Receivable`] what customers ordered, and on the side of
    /// [`Side::Payable`] what is ordered from vendors.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::transaction`]. An order that is damaged is
    /// counted in [`OpenOrders::left_out`] instead.
    pub fn open_orders(&self, side: Side) -> Result<OpenOrders> {
        let (item_lines, kind) = (self.item_lines()?, side.order_kind());
        let mut headers = ShownHeaders::new(self.header_index()?);
        let mut orders = OpenOrders::new(side, item_lines.left_out());
        for (id, lines) in by_order(item_lines.open(kind)) {
            let Some(header) = headers.of_kind(id, kind) else {
                continue;
            };
            match self.transaction_of(header) {
                Ok(order) => orders.add(order, lines),
                Err(Error::Damaged(damage)) => orders.lose(LeftOut::of(&[damage])),
                Err(error) => return Err(error),
            }
        }
        orders.lose(headers.left_out());
        Ok(orders.sorted())
    }

    /// Whether something is left of an order on `side`: whether
    /// [`Self::open_orders`] has a line. Only the lines of the orders are read,
    /// and the forms of them until one has a line.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::transaction`]. An order that is damaged
    /// counts for nothing.
    pub fn has_open_orders(&self, side: Side) -> Result<bool> {
        let kind = side.order_kind();
        let pieces = self.pieces();
        let lines = match (pieces.item_lines.get(), Form::of_kind(kind)) {
            (Some(lines), _) => lines,
            (None, Some(form)) => {
                let lines = pieces.order_lines.of(form);
                lines.get_or_build(|| ItemLines::of_orders(self.source(), kind))?
            }
            (None, None) => return Ok(false),
        };
        for (order, lines) in by_order(lines.open(kind)) {
            let mut orders = OpenOrders::new(side, LeftOut::default());
            match self.transaction(order) {
                Ok(Some(order)) => orders.add(order, lines),
                Ok(None) | Err(Error::Damaged(_)) => continue,
                Err(error) => return Err(error),
            }
            if !orders.lines.is_empty() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// The history of every item in stock.
    pub(super) fn stock_history(&self) -> Result<&History> {
        let pieces = self.pieces();
        pieces
            .stock_history
            .get_or_build(|| History::read(self.source()))
    }

    /// What the lines of every transaction say about items.
    pub(super) fn item_lines(&self) -> Result<&ItemLines> {
        let pieces = self.pieces();
        pieces
            .item_lines
            .get_or_build(|| ItemLines::read(self.source()))
    }
}

/// The open lines of each order.
fn by_order<'lines>(
    open: impl Iterator<Item = &'lines OpenLine>,
) -> BTreeMap<TransactionId, OpenLines<'lines>> {
    let mut orders: BTreeMap<TransactionId, OpenLines<'lines>> = BTreeMap::new();
    for line in open {
        let lines = orders.entry(line.order).or_default();
        lines.insert(line.line, line);
    }
    orders
}

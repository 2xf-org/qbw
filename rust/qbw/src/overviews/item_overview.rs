//! An item.

use std::cmp::Reverse;

use sa17::Decimal;

use crate::error::{Error, Result};
use crate::inventory::{Stock, StockRow};
use crate::items::Item;
use crate::ledger::{Entry, Ledger, Posting};
use crate::left_out::LeftOut;
use crate::price::Price;
use crate::transactions::{Header, Headers, Reader, ShownHeaders};

/// What a person sees on selecting an item.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct ItemOverview<'company> {
    /// The record.
    pub record: &'company Item,
    /// What there is of it now, and what is ordered and planned. `None` for an
    /// item that is not kept in stock.
    pub stock: Option<StockRow>,
    /// The latest lines with the item that are part of the books, the newest
    /// first. Orders and estimates are not: what is left of orders is in
    /// [`Company::open_orders`](crate::Company::open_orders).
    pub rows: Vec<ItemRow>,
    /// How many such lines there are.
    pub row_count: usize,
    /// How much could not be read.
    pub left_out: LeftOut,
}

/// A line with an item.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ItemRow {
    /// What the line posts, such as to the account of the sales.
    pub posting: Posting,
    /// The number of the transaction, as the form shows it.
    pub number: Option<String>,
    /// The quantity, as the form shows it.
    pub quantity: Option<Decimal>,
    /// What one costs, as the form shows it, or a share in percent.
    pub rate: Option<Price>,
}

impl<'company> ItemOverview<'company> {
    /// What there is about an item, with at most `rows` of its lines. `stock` is
    /// there for an item in stock. Only the lines that are shown are read.
    ///
    /// # Errors
    ///
    /// Returns an error when the table of a line lacks a column.
    pub(crate) fn read(
        record: &'company Item,
        stock: Option<&Stock>,
        (ledger, headers): (&Ledger, &Headers),
        reader: &Reader<'_>,
        rows: usize,
    ) -> Result<Self> {
        let mut entries: Vec<&Entry> = ledger.item_entries(record.id).collect();
        entries.sort_by_key(|entry| Reverse((entry.date, entry.line)));
        let mut headers = ShownHeaders::new(headers);
        let mut damage = Vec::new();
        let latest = entries.iter().take(rows).map(|entry| {
            let header = headers.of_kind(entry.transaction, entry.kind());
            let line = match header.map(|header| reader.line(header, entry.row)) {
                Some(Err(Error::Damaged(line))) => {
                    damage.push(line);
                    None
                }
                Some(line) => line?,
                None => None,
            };
            Ok(ItemRow {
                posting: ledger.posting(entry),
                number: header.and_then(Header::number),
                quantity: line.as_ref().and_then(|line| line.quantity),
                rate: line.and_then(|line| line.rate),
            })
        });
        let latest: Vec<ItemRow> = latest.collect::<Result<_>>()?;
        // The stock rests on the items, too, and on other columns of the lines.
        let stock_left_out = stock.map(|stock| stock.left_out).unwrap_or_default();
        let left_out = ledger
            .left_out()
            .with(reader.items.left_out())
            .union(stock_left_out)
            .union(LeftOut::of(&damage));
        Ok(Self {
            record,
            stock: stock.and_then(|stock| stock.get(record.id).cloned()),
            rows: latest,
            row_count: entries.len(),
            left_out: left_out.with(headers.left_out()),
        })
    }
}

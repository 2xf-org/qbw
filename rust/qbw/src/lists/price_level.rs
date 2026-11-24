//! Price levels.

use std::collections::HashMap;

use sa17::{Decimal, Source};

use crate::codes::PriceLevelKind;
use crate::error::Result;
use crate::ids::{CurrencyId, PriceLevelId};
use crate::list::{List, ListEntry, sealed};
use crate::lists::{PriceLevelItem, reading};
use crate::money::Money;
use crate::origin::Origin;
use crate::price::Price;
use crate::tables::lists::{PRICE_LEVEL_ITEMS, PRICE_LEVELS, PriceLevelItemRow, PriceLevelRow};

/// A set of prices for some customers.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct PriceLevel {
    /// What the level is known by.
    pub id: PriceLevelId,
    /// Its name.
    pub name: String,
    /// Whether it changes every price, or those of some items.
    pub kind: PriceLevelKind,
    /// By how many percent it changes every price: below zero where it lowers
    /// them. Nothing for a level with prices of its own.
    pub percent: Option<Decimal>,
    /// The currency its prices are in.
    pub currency: CurrencyId,
    /// The prices of a level that has its own, ordered by item.
    pub items: Vec<PriceLevelItem>,
    /// Whether it can be used in new transactions.
    pub is_active: bool,
    /// The stored row.
    pub origin: Origin,
}

impl sealed::Sealed for PriceLevel {}

impl ListEntry for PriceLevel {
    type Id = PriceLevelId;

    fn id(&self) -> PriceLevelId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn origin(&self) -> Origin {
        self.origin
    }
}

/// Reads the price levels of a company that keeps its books in `home`.
///
/// # Errors
///
/// Returns an error when a table of them lacks a column.
pub(crate) fn read(source: &dyn Source, home: CurrencyId) -> Result<List<PriceLevel>> {
    let mut damage = Vec::new();
    let mut items: HashMap<PriceLevelId, Vec<(Origin, PriceLevelItemRow)>> = HashMap::new();
    for (origin, row) in reading::rows(source, &PRICE_LEVEL_ITEMS, &mut damage)? {
        items.entry(row.level).or_default().push((origin, row));
    }
    let rows = reading::rows(source, &PRICE_LEVELS, &mut damage)?.into_iter();
    let levels = rows
        .filter(|(_, row)| !row.is_deleted)
        .map(|(origin, row)| {
            let items = items.remove(&row.id).unwrap_or_default();
            price_level(origin, row, items, home)
        });
    Ok(reading::sorted_by_name(levels.collect(), damage))
}

fn price_level(
    origin: Origin,
    row: PriceLevelRow,
    mut items: Vec<(Origin, PriceLevelItemRow)>,
    home: CurrencyId,
) -> PriceLevel {
    let currency = row.currency.unwrap_or(home);
    items.sort_by_key(|(_, item)| item.item);
    let items = items.into_iter().map(|(origin, item)| PriceLevelItem {
        item: item.item,
        price: price(&item, currency),
        origin,
    });
    PriceLevel {
        id: row.id,
        name: row.name.unwrap_or_default(),
        kind: row.kind.unwrap_or_default(),
        percent: row.percent,
        currency,
        items: items.collect(),
        is_active: !row.is_hidden,
        origin,
    }
}

fn price(row: &PriceLevelItemRow, currency: CurrencyId) -> Option<Price> {
    let each = row
        .price
        .map(|price| Price::Each(Money::new(price, currency)));
    each.or_else(|| row.percent.map(Price::Percent))
}

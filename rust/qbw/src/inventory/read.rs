//! Putting together what is known of the stock.

use std::collections::HashMap;

use crate::amount::Total;
use crate::as_of::AsOf;
use crate::ids::{CurrencyId, ItemId, SiteId};
use crate::inventory::history::{Entry, Held};
use crate::inventory::item_lines::ItemSums;
use crate::inventory::{
    History, ItemLines, Movements, SiteStock, Stock, StockMovement, StockRow, Valuation,
    ValuationRow,
};
use crate::items::{Item, ItemSite};
use crate::list::List;
use crate::lists::Site;
use crate::settings::Preferences;
use crate::transactions::{Header, Headers, ShownHeaders};

/// What putting the stock together takes.
pub(crate) struct StockReader<'company> {
    pub(crate) preferences: &'company Preferences,
    pub(crate) items: &'company List<Item>,
    pub(crate) sites: &'company List<Site>,
    pub(crate) history: &'company History,
    pub(crate) lines: &'company ItemLines,
}

impl StockReader<'_> {
    /// What there is in stock now.
    pub(crate) fn stock(&self) -> Stock {
        let home = self.preferences.home_currency;
        let items = in_stock(self.items);
        let held = items.filter_map(|item| self.history.held(item.id));
        let value: Total = held.map(|held| held.value).sum();
        Stock {
            rows: in_stock(self.items).map(|item| self.row(item)).collect(),
            value: value.money(home),
            left_out: self
                .items
                .left_out()
                .with(self.sites.left_out())
                .with(self.history.left_out())
                .with(self.lines.left_out()),
        }
    }

    fn row(&self, item: &Item) -> StockRow {
        let home = self.preferences.home_currency;
        let held = self.history.held(item.id);
        let nothing = ItemSums::default();
        let sums = self.lines.of(item.id).unwrap_or(&nothing);
        let settings = item.stock.as_ref();
        StockRow {
            item: item.id,
            on_hand: on_hand(held).quantity(),
            value: value(held).money(home),
            average_cost: held.map(|held| held.average_cost.money(home)),
            counted: sums.on_hand.quantity(),
            on_order: sums.on_order.quantity(),
            on_sales_orders: sums.on_sales_orders.quantity(),
            to_be_built: sums.to_be_built.quantity(),
            required_for_builds: sums.required_for_builds.quantity(),
            available: self.available(on_hand(held), sums).quantity(),
            reorder_point: settings.and_then(|settings| settings.reorder_point),
            sites: self.sites(item, sums),
            stored: item.stored,
        }
    }

    /// How much of what there is can be promised.
    fn available(&self, on_hand: Total, sums: &ItemSums) -> Total {
        let mut available = on_hand;
        if self.preferences.available_less_sales_orders {
            available = available - sums.on_sales_orders;
        }
        if self.preferences.available_less_builds {
            available = available - sums.required_for_builds;
        }
        available
    }

    /// What there is of an item at each site and in each bin that has had some.
    fn sites(&self, item: &Item, sums: &ItemSums) -> Vec<SiteStock> {
        let stored: HashMap<SiteId, &ItemSite> = item
            .sites
            .iter()
            .filter_map(|stored| Some((stored.site?, stored)))
            .collect();
        let sites = self.sites.iter().filter_map(|site| {
            let moved = sums.sites.get(&site.id).copied();
            let stored = stored.get(&site.id);
            let is_known = moved.is_some() || stored.is_some();
            is_known.then(|| SiteStock {
                site: site.id,
                on_hand: moved.unwrap_or_default().quantity(),
                reorder_point: stored.and_then(|stored| stored.reorder_point),
                stored: stored.and_then(|stored| stored.on_hand),
            })
        });
        sites.collect()
    }
}

/// What there was in stock at `as_of`, in the books of a company that keeps them
/// in `home`.
pub(crate) fn valuation(
    items: &List<Item>,
    history: &History,
    home: CurrencyId,
    as_of: AsOf,
) -> Valuation {
    let mut total = Total::default();
    let rows = in_stock(items).map(|item| {
        let held = history.held_at(item.id, as_of);
        total += value(held);
        ValuationRow {
            item: item.id,
            on_hand: on_hand(held).quantity(),
            value: value(held).money(home),
            average_cost: held.map(|held| held.average_cost.money(home)),
        }
    });
    Valuation {
        as_of,
        rows: rows.collect(),
        value: total.money(home),
        left_out: items.left_out().with(history.left_out()),
    }
}

/// What happened to the stock of `item`.
pub(crate) fn movements(
    item: ItemId,
    history: &History,
    headers: &Headers,
    home: CurrencyId,
) -> Movements {
    let mut headers = ShownHeaders::new(headers);
    let mut before = Held::default();
    let mut rows = Vec::new();
    for entry in history.of(item) {
        rows.extend(movement(entry, before, &mut headers, home));
        before = entry.held;
    }
    Movements {
        item,
        rows,
        left_out: history.left_out().with(headers.left_out()),
    }
}

/// The change from `before` to `entry`, unless that is the start of the item.
fn movement(
    entry: &Entry,
    before: Held,
    headers: &mut ShownHeaders<'_>,
    home: CurrencyId,
) -> Option<StockMovement> {
    let cause = entry.cause?;
    let header = headers.of_any_kind(cause.transaction);
    let held = entry.held;
    let quantity = Total::from(held.on_hand) - before.on_hand.into();
    let value = Total::from(held.value) - before.value.into();
    Some(StockMovement {
        date: entry.date,
        transaction: cause.transaction,
        line: cause.line,
        kind: header.map(|header| header.kind),
        number: header.and_then(Header::number),
        name: header.and_then(Header::name),
        quantity: quantity.quantity(),
        value: value.money(home),
        on_hand: held.on_hand.quantity(),
        value_on_hand: held.value.money(home),
        average_cost: held.average_cost.money(home),
        origin: entry.origin(),
    })
}

/// The items that are kept in stock.
fn in_stock(items: &List<Item>) -> impl Iterator<Item = &Item> {
    items.iter().filter(|item| item.kind.is_stocked())
}

fn on_hand(held: Option<Held>) -> Total {
    held.unwrap_or_default().on_hand.into()
}

fn value(held: Option<Held>) -> Total {
    held.unwrap_or_default().value.into()
}

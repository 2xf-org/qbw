//! The reading of the list of items.

use std::collections::HashMap;

use sa17::Source;

use crate::codes::ItemKind;
use crate::damage::Damage;
use crate::error::Result;
use crate::ids::{CurrencyId, ItemId};
use crate::items::{
    Item, ItemSite, Member, Purchase, Sales, StockSettings, StoredStock, TaxSettings,
};
use crate::list::{List, order};
use crate::money::Money;
use crate::origin::Origin;
use crate::price::Price;
use crate::tables::items::{ITEM_SITES, ITEM_TOTALS, ITEMS, ItemRow, ItemTotalsRow, MEMBERS};

/// Reads the items of a company that keeps its books in `home`.
///
/// # Errors
///
/// Returns an error when the file lacks a table or a column of it.
pub(crate) fn read(source: &dyn Source, home: CurrencyId) -> Result<List<Item>> {
    let mut damage = Vec::new();
    let mut parts = Parts::read(source, &mut damage)?;
    let links = parts.totals.values();
    let links = links.filter_map(|totals| Some((totals.id.number(), totals.next?)));
    let links: Vec<(i32, i32)> = links.collect();
    let mut items: Vec<Item> = ITEMS
        .open(source)?
        .scan(&mut damage)
        .filter(|(_, row)| !row.is_deleted)
        .map(|(origin, row)| item(origin, row, &mut parts, home))
        .collect();
    order::sort(&mut items, links, |item| item.id.number());
    Ok(List::new(items, damage))
}

/// What the file keeps apart from the items, by item.
struct Parts {
    totals: HashMap<ItemId, ItemTotalsRow>,
    members: HashMap<ItemId, Vec<Member>>,
    sites: HashMap<ItemId, Vec<ItemSite>>,
}

impl Parts {
    fn read(source: &dyn Source, damage: &mut Vec<Damage>) -> Result<Self> {
        let totals = ITEM_TOTALS.open(source)?;
        let totals = totals.scan(damage);
        Ok(Self {
            totals: totals.map(|(_, totals)| (totals.id, totals)).collect(),
            members: members(source, damage)?,
            sites: sites(source, damage)?,
        })
    }
}

fn members(source: &dyn Source, damage: &mut Vec<Damage>) -> Result<HashMap<ItemId, Vec<Member>>> {
    let mut members: HashMap<ItemId, Vec<Member>> = HashMap::new();
    let Some(table) = MEMBERS.find(source)? else {
        return Ok(members);
    };
    let mut rows: Vec<_> = table.scan(damage).collect();
    rows.sort_by_key(|(_, row)| (row.of, row.position));
    for (origin, row) in rows {
        // A blank row of the window is stored, too.
        if let Some(item) = row.item {
            members.entry(row.of).or_default().push(Member {
                item,
                quantity: row.quantity,
                origin,
            });
        }
    }
    Ok(members)
}

fn sites(source: &dyn Source, damage: &mut Vec<Damage>) -> Result<HashMap<ItemId, Vec<ItemSite>>> {
    let mut sites: HashMap<ItemId, Vec<ItemSite>> = HashMap::new();
    let Some(table) = ITEM_SITES.find(source)? else {
        return Ok(sites);
    };
    let mut rows: Vec<_> = table.scan(damage).collect();
    rows.retain(|(_, row)| !row.is_deleted);
    rows.sort_by_key(|(_, row)| (row.item, row.site));
    for (origin, row) in rows {
        if let Some(item) = row.item {
            sites.entry(item).or_default().push(ItemSite {
                site: row.site,
                on_hand: row.on_hand,
                reorder_point: row.reorder_point,
                origin,
            });
        }
    }
    Ok(sites)
}

fn item(origin: Origin, row: ItemRow, parts: &mut Parts, home: CurrencyId) -> Item {
    let totals = parts.totals.remove(&row.id);
    let money = |amount| Money::new(amount, home);
    let is_bought = row.kind.is_stocked() || row.is_pass_through;
    Item {
        id: row.id,
        parent: row.parent,
        kind: row.kind,
        special: totals.as_ref().and_then(|totals| totals.special),
        sales: Sales {
            account: row.sales_account,
            price: price(&row, home),
            description: row.sales_description,
        },
        purchase: is_bought.then_some(Purchase {
            account: row.purchase_account,
            cost: row.cost.or(row.exact_cost).map(money),
            vendor: row.vendor,
            description: row.purchase_description,
        }),
        stock: row.kind.is_stocked().then_some(StockSettings {
            asset_account: row.asset_account,
            reorder_point: row.reorder_point,
            build_point: row.build_point,
        }),
        tax: (row.kind == ItemKind::SalesTax).then_some(TaxSettings {
            agency: row.tax_agency,
        }),
        members: parts.members.remove(&row.id).unwrap_or_default(),
        prints_members: row.prints_members,
        name: row.name.unwrap_or_default(),
        part_number: row.part_number,
        tax_code: row.tax_code,
        category: row.category,
        class: row.class,
        is_active: !row.is_hidden,
        stored: totals.as_ref().and_then(|totals| stored(totals, home)),
        sites: parts.sites.remove(&row.id).unwrap_or_default(),
        owner_key: row.owner_key,
        list_id: totals.and_then(|totals| totals.list_id),
        created: row.created,
        modified: row.modified,
        origin,
    }
}

/// What the item is sold for, from the one of its three columns that has it.
fn price(row: &ItemRow, home: CurrencyId) -> Option<Price> {
    let each = row.price.or(row.exact_price);
    let each = each.map(|price| Price::Each(Money::new(price, home)));
    each.or_else(|| row.price_percent.map(Price::Percent))
}

/// The stock of an item. One that is not kept in stock has no quantity.
fn stored(totals: &ItemTotalsRow, home: CurrencyId) -> Option<StoredStock> {
    let money = |amount| Money::new(amount, home);
    Some(StoredStock {
        on_hand: totals.on_hand?,
        value: totals.value.map(money),
        average_cost: totals.average_cost.map(money),
        on_order: totals.on_order,
        on_sales_orders: totals.on_sales_order,
        to_be_built: totals.to_be_built,
        required_for_builds: totals.required_for_builds,
    })
}

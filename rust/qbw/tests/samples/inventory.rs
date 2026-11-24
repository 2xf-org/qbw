//! The stock against what QuickBooks keeps itself, and against a second
//! reading of the lines.

use std::collections::HashMap;

use qbw::chrono::NaiveDate;
use qbw::inventory::StockRow;
use qbw::sa17::{Columns, Decimal, Row};
use qbw::{Company, ItemId, LineId, Side};

use crate::agreement::Agreement;
use crate::{agreement, books, support};

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn all_of_the_stock_can_be_read() {
    for company in support::samples() {
        let valuation = company.valuation(day("2023-12-31")).unwrap();
        let stock = company.stock().unwrap();
        let movements = stock.rows.iter().map(|row| {
            let movements = company.movements(row.item).unwrap().unwrap();
            assert!(movements.left_out.is_nothing());
            movements.rows.len()
        });
        let movements: usize = movements.sum();
        println!("{} items, {movements} movements", stock.rows.len());
        assert!(stock.left_out.is_nothing() && valuation.left_out.is_nothing());
    }
}

fn day(text: &str) -> NaiveDate {
    text.parse().unwrap()
}

/// Counts how often `kept` of a row is what QuickBooks keeps.
fn rows_that_agree(company: &Company, agrees: impl Fn(&StockRow) -> Option<bool>) -> Agreement {
    let mut rows = Agreement::default();
    for row in company.stock().unwrap().rows {
        if let Some(agrees) = agrees(&row) {
            rows.add(agrees);
        }
    }
    rows
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn the_history_ends_with_what_quickbooks_keeps() {
    for company in books::books() {
        let on_hand = |row: &StockRow| Some(row.on_hand == row.stored?.on_hand);
        rows_that_agree(company, on_hand).check("quantities of the history");
        let value = |row: &StockRow| Some(row.value == row.stored?.value?);
        rows_that_agree(company, value).check("values of the history");
        // An item that nothing happened to has a cost that somebody typed.
        let cost = |row: &StockRow| Some(row.average_cost? == row.stored?.average_cost?);
        rows_that_agree(company, cost).check("average costs of the history");
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn the_lines_add_up_to_what_quickbooks_keeps() {
    for company in books::books() {
        let counted = |row: &StockRow| Some(row.counted == row.stored?.on_hand);
        rows_that_agree(company, counted).check("quantities of the lines");
    }
}

/// Counts what differs, and checks that no more does in all books than did.
fn check_differing(what: &str, differing: usize, known: usize) {
    println!("{differing} {what} in all books");
    assert!(differing <= known, "{what} are more than they were");
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn orders_and_builds_are_what_quickbooks_keeps() {
    let mut differing = [0; 4];
    for company in books::books() {
        for row in company.stock().unwrap().rows {
            let stored = row.stored.unwrap();
            let differs = [
                stored.on_order != Some(row.on_order),
                stored.on_sales_orders != Some(row.on_sales_orders),
                stored.to_be_built != Some(row.to_be_built),
                stored.required_for_builds != Some(row.required_for_builds),
            ];
            for (count, differs) in differing.iter_mut().zip(differs) {
                *count += usize::from(differs);
            }
        }
    }
    let [on_order, on_sales_orders, to_be_built, required] = differing;
    check_differing("quantities on order that differ", on_order, 0);
    // Every line of the sales order that is kept as open is done.
    check_differing("quantities on sales orders that differ", on_sales_orders, 1);
    check_differing("quantities to be built that differ", to_be_built, 0);
    check_differing("quantities needed for builds that differ", required, 0);
}

/// A line of an item, read a second time.
struct ItemLine {
    id: LineId,
    item: ItemId,
    date: NaiveDate,
    /// What it put into the stock, unless it does not count.
    moved: Decimal,
    /// Whether it is of a transfer between sites, which leaves the stock as it
    /// was.
    is_transfer: bool,
}

/// The codes of the kinds whose lines take out of the stock.
const OUT: [i32; 3] = [4, 11, 12];

/// The code of a transfer between sites.
const SITE_TRANSFER: i32 = 36;

/// The lines of items in stock that count, from the tables themselves.
fn item_lines(company: &Company) -> Vec<ItemLine> {
    let columns = [
        "target_id",
        "item_id",
        "transaction_date",
        "transaction_view_type",
        "quantity_qnty",
        "is_no_post_bool",
        "is_memorized_transaction_bool",
    ];
    let stock = company.stock().unwrap();
    let mut lines = Vec::new();
    for table in company.source().tables() {
        let name = table.name();
        let is_of_lines = name.starts_with("abmc_")
            && name.ends_with("_lineitem")
            && !name.starts_with("abmc_audit_")
            && table.definition().position("quantity_qnty").is_some()
            && table.definition().position("item_id").is_some();
        if !is_of_lines {
            continue;
        }
        let columns = Columns::new(table.definition(), columns).unwrap();
        for row in table.scan(&columns) {
            let row = row.unwrap();
            let item = ItemId::new(row.read::<Option<i32>>(1).unwrap().unwrap_or_default());
            let is_out_of_the_books: [bool; 2] = [5, 6].map(|column| row.read(column).unwrap());
            if stock.get(item).is_none() || is_out_of_the_books.contains(&true) {
                continue;
            }
            let kind: i32 = row.read(3).unwrap();
            let quantity: Option<Decimal> = row.read(4).unwrap();
            let one = if OUT.contains(&kind) { -1 } else { 1 };
            lines.push(ItemLine {
                id: LineId::new(row.read(0).unwrap()),
                item,
                date: row.read(2).unwrap(),
                moved: quantity.unwrap_or_else(|| Decimal::from(one)),
                is_transfer: kind == SITE_TRANSFER,
            });
        }
    }
    lines
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn every_movement_is_of_a_line_and_every_line_is_a_movement() {
    for company in books::books() {
        let mut lines: HashMap<LineId, ItemLine> = item_lines(company)
            .into_iter()
            .filter(|line| !line.is_transfer && !line.moved.is_zero())
            .map(|line| (line.id, line))
            .collect();
        let mut movements = Agreement::default();
        for row in company.stock().unwrap().rows {
            for movement in company.movements(row.item).unwrap().unwrap().rows {
                let line = lines.remove(&movement.line);
                let agrees = line.map_or(movement.quantity.is_zero(), |line| {
                    line.item == row.item
                        && line.date == movement.date
                        && line.moved == movement.quantity
                });
                movements.add(agrees && movement.kind.is_some());
            }
        }
        movements.check("movements with the quantity of their line");
        assert_eq!(lines.len(), 0, "lines that are no movement");
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn on_a_day_there_was_what_the_lines_up_to_it_add_up_to() {
    let days = [
        "2014-12-31",
        "2018-06-30",
        "2021-12-31",
        "2023-02-28",
        "2025-06-30",
    ];
    for company in books::books() {
        let lines = item_lines(company);
        let mut quantities = Agreement::default();
        for day in days.map(day) {
            let mut counted: HashMap<ItemId, Decimal> = HashMap::new();
            for line in lines.iter().filter(|line| line.date <= day) {
                let counted = counted.entry(line.item).or_default();
                *counted = counted.checked_add(line.moved).unwrap();
            }
            for row in company.valuation(day).unwrap().rows {
                let counted = counted.get(&row.item).copied().unwrap_or_default();
                quantities.add(counted == row.on_hand);
            }
        }
        quantities.check("quantities of items on five days");
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn sites_and_bins_have_what_quickbooks_keeps() {
    let mut looked_at = 0;
    let mut differing = 0;
    for company in books::books() {
        for row in company.stock().unwrap().rows {
            for at in &row.sites {
                looked_at += 1;
                differing += usize::from(at.stored.unwrap_or_default() != at.on_hand);
            }
        }
    }
    println!("{looked_at} quantities at sites and in bins");
    check_differing("quantities at sites and in bins that differ", differing, 12);
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn open_orders_add_up_to_what_is_on_order() {
    for company in books::books() {
        let stock = company.stock().unwrap();
        let mut items = Agreement::default();
        let mut progress = Agreement::default();
        for side in [Side::Receivable, Side::Payable] {
            let has_open = company.has_open_orders(side).unwrap();
            let orders = company.open_orders(side).unwrap();
            assert!(orders.left_out.is_nothing() && has_open != orders.lines.is_empty());
            let mut open: HashMap<ItemId, Decimal> = HashMap::new();
            for line in &orders.lines {
                // The form has what the line has, where it has a quantity.
                let of_line = line.line.order.unwrap();
                progress.add(!of_line.is_done && of_line.open.is_none_or(|open| open == line.open));
                let open = open.entry(line.line.item.unwrap()).or_default();
                *open = open.checked_add(line.open).unwrap();
            }
            for row in &stock.rows {
                let on_order = match side {
                    Side::Receivable => row.on_sales_orders,
                    _ => row.on_order,
                };
                items.add(open.get(&row.item).copied().unwrap_or_default() == on_order);
            }
            let done = orders.lines.iter().filter(|line| line.is_order_done);
            println!(
                "{side:?}: {} open lines, {} of them in orders that are done",
                orders.lines.len(),
                done.count()
            );
        }
        items.check("items whose open lines are what is on order");
        progress.check("open lines as their form has them");
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn the_history_has_of_an_order_what_is_left_of_it_now() {
    let columns = [
        "item_id",
        "sequence_number",
        "target_id",
        "quantity_on_order",
        "quantity_on_sales_order",
    ];
    let mut looked_at = 0;
    let mut differing = 0;
    for company in books::books() {
        let mut open: HashMap<i32, Decimal> = HashMap::new();
        for side in [Side::Receivable, Side::Payable] {
            let lines = company.open_orders(side).unwrap().lines;
            open.extend(lines.iter().map(|line| (line.line.id.number(), line.open)));
        }
        let mut rows = agreement::rows(company, "abmc_item_history", &columns);
        let key = |row: &Row| -> [i32; 2] { [0, 1].map(|column| row.read(column).unwrap()) };
        rows.sort_by_key(key);
        for pair in rows.windows(2) {
            let [before, row] = pair else {
                continue;
            };
            if key(before)[0] != key(row)[0] {
                continue;
            }
            // On the row of an order, one of the two changes by what is left.
            let added = [3, 4].map(|column| {
                let figure = |row: &Row| row.read::<Decimal>(column).unwrap();
                figure(row).checked_sub(figure(before)).unwrap()
            });
            let added = added[0].checked_add(added[1]).unwrap();
            let line: i32 = row.read(2).unwrap();
            looked_at += 1;
            differing += usize::from(added != open.get(&line).copied().unwrap_or_default());
        }
    }
    println!("{looked_at} rows of the history");
    check_differing("rows that have another figure of an order", differing, 1);
}

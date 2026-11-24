//! A small company that the command's documentation and tests share.

use crate::cell::{Cell, LongStorage};
use crate::scramble::Sign;
use crate::synthetic_database::SyntheticDatabase;
use crate::synthetic_table::{SyntheticColumn, SyntheticRow, SyntheticTable};

/// The user that owns a company's tables.
const OWNER: &str = "QBAdminGroup";

/// 1 January 2024, in minutes since 29 February 1600.
const NEW_YEAR_2024: u32 = 222_917_760;
const MINUTES_PER_DAY: u32 = 24 * 60;

/// A company with three customers, four invoices and one view.
///
/// The `invoices` table had its `memo` column added in schema version 1, so
/// the first two invoices are older than the column. One customer's row
/// moved to an extension page, one was deleted, and one has notes too long
/// for its row.
#[must_use]
pub fn sample() -> SyntheticDatabase {
    let customers = SyntheticTable::new("customers")
        .column(SyntheticColumn::new("id", 2))
        .column(SyntheticColumn::new("name", 9).width(41))
        .column(SyntheticColumn::new("balance", 3).width(15).scale(2))
        .column(SyntheticColumn::new("since", 6))
        .column(SyntheticColumn::new("is_active", 24).width(1))
        .column(SyntheticColumn::new("notes", 10).width(32767).nullable())
        .primary_key(&["id"])
        .row(customer(
            1,
            "First Customer",
            Cell::numeric(Sign::Positive, -1, &[50, 34, 12]),
            0,
            Cell::bit(true),
            Cell::Null,
        ))
        .row(
            customer(
                2,
                "Deleted Customer",
                Cell::numeric(Sign::Positive, 0, &[]),
                1,
                Cell::bit(false),
                Cell::Null,
            )
            .deleted(),
        )
        .row(
            customer(
                3,
                "Second Customer",
                Cell::numeric(Sign::Negative, -1, &[99, 99]),
                31,
                Cell::bit(true),
                Cell::Null,
            )
            .forwarded(),
        )
        .row(customer(
            4,
            "Third Customer",
            Cell::numeric(Sign::Positive, 0, &[]),
            45,
            Cell::bit(false),
            Cell::long(
                "Prefers invoices by post. ".repeat(40).as_bytes(),
                LongStorage::Compressed,
            ),
        ));

    let invoices = SyntheticTable::new("invoices")
        .column(SyntheticColumn::new("id", 2))
        .column(SyntheticColumn::new("customer_id", 2))
        .column(SyntheticColumn::new("total", 3).width(15).scale(2))
        .column(SyntheticColumn::new("issued", 13).width(8))
        .column(
            SyntheticColumn::new("memo", 9)
                .width(60)
                .nullable()
                .added_in(1, None),
        )
        .primary_key(&["id"])
        .foreign_key("customer", &["customer_id"], "customers")
        .row(invoice(100, 1, &[0, 50, 2], 10, None))
        .row(invoice(101, 1, &[50, 84, 9], 12, None))
        .row(invoice(
            102,
            3,
            &[99, 99],
            40,
            Some(Cell::text("Paid in cash")),
        ))
        .row(invoice(103, 4, &[0, 20, 1], 50, Some(Cell::Null)));

    let customer_names = SyntheticTable::new("customer_names")
        .view()
        .column(SyntheticColumn::new("name", 9).width(41));

    SyntheticDatabase::new()
        .owner(OWNER)
        .table(customers)
        .table(invoices)
        .table(customer_names)
}

fn customer(
    id: i32,
    name: &str,
    balance: Cell,
    day: u32,
    active: Cell,
    notes: Cell,
) -> SyntheticRow {
    SyntheticRow::new([
        Cell::integer(id),
        Cell::text(name),
        balance,
        Cell::date(NEW_YEAR_2024 + day * MINUTES_PER_DAY),
        active,
        notes,
    ])
}

/// An invoice written before the `memo` column existed when `memo` is `None`.
fn invoice(id: i32, customer: i32, total: &[u8], day: u32, memo: Option<Cell>) -> SyntheticRow {
    let issued = NEW_YEAR_2024 + day * MINUTES_PER_DAY + 9 * 60 + 30;
    let mut cells = vec![
        Cell::integer(id),
        Cell::integer(customer),
        Cell::numeric(Sign::Positive, -1, total),
        Cell::timestamp(issued, 0),
    ];
    match memo {
        Some(memo) => {
            cells.push(memo);
            SyntheticRow::new(cells).schema(1)
        }
        None => SyntheticRow::new(cells),
    }
}

//! Small companies that keep stock, for what the books do not have.

use qbw::Company;
use qbw_synthetic::{Cell, SyntheticDatabase, SyntheticRow, SyntheticTable};

use crate::small_lists::{Kind, State};
use crate::{small_lists, support};

/// A flag on a row that no reader knows, which makes the row unreadable.
const UNKNOWN_FLAG: u8 = 0x10;

/// The codes of the kinds that the lines below are of.
pub(crate) const INVOICE: i32 = 4;
pub(crate) const BILL: i32 = 9;
pub(crate) const VENDOR_CREDIT: i32 = 12;
pub(crate) const PURCHASE_ORDER: i32 = 17;
pub(crate) const SALES_ORDER: i32 = 27;
pub(crate) const BUILD: i32 = 29;
pub(crate) const SITE_TRANSFER: i32 = 36;

/// A part in stock, the assembly made of it, and a service.
pub(crate) const PART: i32 = 1;
pub(crate) const ASSEMBLY: i32 = 2;
pub(crate) const SERVICE: i32 = 3;

/// The site and its two bins, and another site.
pub(crate) const YARD: i32 = 11;
pub(crate) const SHELF: i32 = 12;
pub(crate) const RACK: i32 = 13;
pub(crate) const SHOP: i32 = 14;

/// Whether the transaction of a line counts.
#[derive(Clone, Copy)]
pub(crate) enum Counts {
    Yes,
    Pending,
    Memorized,
}

/// Whether a line is the transaction's own side.
#[derive(Clone, Copy)]
pub(crate) enum Own {
    Side,
    Line,
}

/// A line of an item.
#[derive(Clone, Copy)]
pub(crate) struct Line {
    pub(crate) kind: i32,
    pub(crate) item: i32,
    pub(crate) quantity: Option<&'static str>,
    pub(crate) open: Option<&'static str>,
    pub(crate) own: Own,
    pub(crate) counts: Counts,
    pub(crate) site: Option<i32>,
    pub(crate) bin: Option<i32>,
}

impl Line {
    /// A line of a transaction that counts.
    pub(crate) fn new(kind: i32, item: i32, quantity: Option<&'static str>) -> Self {
        Self {
            kind,
            item,
            quantity,
            open: None,
            own: Own::Line,
            counts: Counts::Yes,
            site: None,
            bin: None,
        }
    }

    /// The same line at a site and in a bin of it.
    pub(crate) fn at(self, site: i32, bin: Option<i32>) -> Self {
        Self {
            site: Some(site),
            bin,
            ..self
        }
    }

    fn row(&self, id: i32) -> [Cell; 11] {
        let whole = |number: Option<i32>| number.map_or(Cell::Null, Cell::integer);
        let number = |text: Option<&str>| text.map_or(Cell::Null, Cell::decimal);
        [
            Cell::integer(id),
            Cell::integer(id + 1000),
            Cell::integer(self.kind),
            Cell::bit(matches!(self.own, Own::Side)),
            Cell::integer(self.item),
            number(self.quantity),
            number(self.open),
            Cell::bit(matches!(self.counts, Counts::Pending)),
            Cell::bit(matches!(self.counts, Counts::Memorized)),
            whole(self.site),
            whole(self.bin),
        ]
    }
}

/// The table of `lines`.
pub(crate) fn lines(lines: &[Line]) -> SyntheticTable {
    let columns = [
        ("target_id", Kind::Whole),
        ("transaction_id", Kind::Whole),
        ("transaction_view_type", Kind::Whole),
        ("is_source_bool", Kind::Flag),
        ("item_id", Kind::Whole),
        ("quantity_qnty", Kind::Number),
        ("open_qnty", Kind::Number),
        ("is_no_post_bool", Kind::Flag),
        ("is_memorized_transaction_bool", Kind::Flag),
        ("site_id", Kind::Whole),
        ("rsb_id", Kind::Whole),
    ];
    let rows = (1..).zip(lines).map(|(id, line)| line.row(id));
    small_lists::table("abmc_bill_lineitem", columns, rows)
}

/// What there was of an item after a line of it: the item, the place of the
/// row, the date, the quantity, the value and the average cost. The first row
/// of an item has no line.
pub(crate) type History = (
    i32,
    i32,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
);

/// The table of `rows`.
pub(crate) fn history(rows: &[History]) -> SyntheticTable {
    let rows = rows.iter().map(history_row);
    small_lists::table("abmc_item_history", HISTORY, rows)
}

/// The table of `rows`, of which the last cannot be read.
pub(crate) fn damaged_history(rows: &[History]) -> SyntheticTable {
    let lost = rows.last().map(history_row);
    let table = history(rows.get(..rows.len() - 1).unwrap_or_default());
    let lost = lost.map(|row| SyntheticRow::new(row).flags(UNKNOWN_FLAG));
    lost.into_iter().fold(table, SyntheticTable::row)
}

const HISTORY: [(&str, Kind); 8] = [
    ("item_id", Kind::Whole),
    ("sequence_number", Kind::Whole),
    ("transaction_date", Kind::Day),
    ("transaction_id", Kind::Whole),
    ("target_id", Kind::Whole),
    ("quantity_on_hand", Kind::Number),
    ("value_on_hand", Kind::Number),
    ("average_cost", Kind::Number),
];

fn history_row(&(item, sequence, date, on_hand, value, average_cost): &History) -> [Cell; 8] {
    let line = Cell::integer(sequence);
    let transaction = if sequence == 0 {
        Cell::integer(0)
    } else {
        Cell::integer(sequence + 1000)
    };
    [
        Cell::integer(item),
        Cell::integer(sequence),
        Cell::day(date),
        transaction,
        line,
        Cell::decimal(on_hand),
        Cell::decimal(value),
        Cell::decimal(average_cost),
    ]
}

/// A company with a part in stock that is to be ordered at `reorder_point`, the
/// assembly and the service, a yard with two bins and a shop, `tables`, and
/// the preferences `values`.
pub(crate) fn company(
    values: &[(&str, &str)],
    reorder_point: Option<&str>,
    tables: Vec<SyntheticTable>,
) -> Company {
    let database = support::preferences(values).table(support::no_accounts());
    let totals = small_lists::table("abmc_item_internal", [("item_id", Kind::Whole)], []);
    let database = database
        .table(items(reorder_point))
        .table(totals)
        .table(sites());
    let database = tables.into_iter().fold(database, SyntheticDatabase::table);
    support::company(database)
}

fn items(reorder_point: Option<&str>) -> SyntheticTable {
    let row = |id, name, kind, reorder_point: Option<&str>| {
        let [is_hidden, is_deleted] = State::Shown.flags();
        [
            Cell::integer(id),
            Cell::text(name),
            Cell::Null,
            Cell::integer(kind),
            is_hidden,
            is_deleted,
            reorder_point.map_or(Cell::Null, Cell::decimal),
        ]
    };
    let columns = [
        ("item_id", Kind::Whole),
        ("name", Kind::Text),
        ("parent_id", Kind::Whole),
        ("item_type", Kind::Whole),
        ("is_hidden_bool", Kind::Flag),
        ("is_deleted_bool", Kind::Flag),
        ("reorder_point_qnty", Kind::Number),
    ];
    let rows = [
        row(PART, "Hinge", 1, reorder_point),
        row(ASSEMBLY, "Gate", 2, None),
        row(SERVICE, "Fitting", 0, None),
    ];
    small_lists::table("abmc_item_user", columns, rows)
}

/// The sites in their order, each with the one after it.
fn sites() -> SyntheticTable {
    let row = |id, name, parent: Option<i32>, next| {
        [
            Cell::integer(id),
            Cell::text(name),
            parent.map_or(Cell::Null, Cell::integer),
            Cell::integer(next),
        ]
    };
    let columns = [
        ("site_id", Kind::Whole),
        ("name", Kind::Text),
        ("parent_id", Kind::Whole),
        ("display_index_next", Kind::Whole),
    ];
    let rows = [
        row(YARD, "Yard", None, -SHELF),
        row(SHELF, "Shelf", Some(YARD), RACK),
        row(RACK, "Rack", Some(YARD), SHOP),
        row(SHOP, "Shop", None, YARD),
    ];
    small_lists::table("abmc_site", columns, rows)
}

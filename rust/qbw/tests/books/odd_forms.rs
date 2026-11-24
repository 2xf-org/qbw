//! Small companies with transactions that the books do not have.

use qbw::Company;
use qbw_synthetic::{Cell, SyntheticColumn, SyntheticRow, SyntheticTable};

use crate::small_lists::Kind;
use crate::{small_lists, support};

/// A flag on a row that no reader knows, which makes the row unreadable.
const UNKNOWN_FLAG: u8 = 0x10;

/// The columns that every table of headers has.
const HEADER: &[&str] = &[
    "transaction_id",
    "target_id",
    "transaction_date",
    "doc_num",
    "memo",
    "currency_id",
    "exchange_rate",
    "last_auditable_action",
    "last_edit_by",
    "trans_num",
    "abm_id",
    "timestamp_tms",
    "modtimestamp_tms",
];

/// The columns that every table of lines has.
const LINE: &[&str] = &[
    "target_id",
    "transaction_id",
    "is_source_bool",
    "next_target_id",
    "account_id",
    "sibling_account_id",
    "memo",
    "doc_num",
    "currency_id",
    "amount_amt",
    "foreign_amount_amt",
    "is_gain_loss_target_bool",
    "abm_id",
];

/// The accounts with their types: a bank, a receivable, a payable, a liability
/// and income.
const ACCOUNTS: &[(i32, i32)] = &[(1, 0), (3, 1), (7, 5), (9, 7), (12, 10)];

/// The items with their kinds: a service, two taxes and a group of them, and a
/// discount.
const ITEMS: &[(i32, i32)] = &[(501, 0), (503, 10), (504, 10), (505, 11), (506, 8)];

/// A row: what its columns hold, written as a text. The others hold nothing.
#[derive(Clone)]
pub(crate) struct Row {
    values: Vec<(&'static str, &'static str)>,
    is_unreadable: bool,
}

impl Row {
    pub(crate) fn new(values: &[(&'static str, &'static str)]) -> Self {
        Self {
            values: values.to_vec(),
            is_unreadable: false,
        }
    }

    /// The same row, damaged.
    pub(crate) fn unreadable(self) -> Self {
        Self {
            is_unreadable: true,
            ..self
        }
    }

    fn write(&self, columns: &[&str]) -> SyntheticRow {
        let cells = columns.iter().map(|column| {
            let mut values = self.values.iter();
            let value = values.find(|(name, _)| name == column);
            match value {
                Some((_, value)) => cell(column, value),
                None => default(column),
            }
        });
        let row = SyntheticRow::new(cells);
        if self.is_unreadable {
            row.flags(UNKNOWN_FLAG)
        } else {
            row
        }
    }
}

/// The transactions of one form.
pub(crate) struct Form {
    /// What the names of its tables start with.
    pub(crate) tables: &'static str,
    pub(crate) headers: Vec<Row>,
    /// Without any, the form has no table of lines.
    pub(crate) lines: Vec<Row>,
}

/// A company with `forms`, and a few accounts and items.
pub(crate) fn company(forms: &[Form]) -> Company {
    let mut database = support::preferences(&[]);
    for table in lists() {
        database = database.table(table);
    }
    for form in forms {
        let headers = format!("{}_header", form.tables);
        database = database.table(written(&headers, HEADER, &form.headers));
        if !form.lines.is_empty() {
            let lines = format!("{}_lineitem", form.tables);
            database = database.table(written(&lines, LINE, &form.lines));
        }
    }
    support::company(database)
}

/// A company with one transaction.
pub(crate) fn with_one(tables: &'static str, header: Row, lines: Vec<Row>) -> Company {
    let headers = vec![header];
    company(&[Form {
        tables,
        headers,
        lines,
    }])
}

fn lists() -> Vec<SyntheticTable> {
    let off = || Cell::bit(false);
    let accounts = ACCOUNTS.iter().map(|&(id, kind)| {
        let (id, kind) = (Cell::integer(id), Cell::integer(kind));
        [id, Cell::text("Account"), Cell::Null, kind, off(), off()]
    });
    let items = ITEMS.iter().map(|&(id, kind)| {
        let (id, kind) = (Cell::integer(id), Cell::integer(kind));
        [id, Cell::text("Item"), Cell::Null, kind, off(), off()]
    });
    let columns = |key, kind| {
        [
            (key, Kind::Whole),
            ("name", Kind::Text),
            ("parent_id", Kind::Whole),
            (kind, Kind::Whole),
            ("is_hidden_bool", Kind::Flag),
            ("is_deleted_bool", Kind::Flag),
        ]
    };
    let account_totals = [
        ("account_id", Kind::Whole),
        ("special_type", Kind::Whole),
        ("display_index_next", Kind::Whole),
    ];
    let item_totals = [("item_id", Kind::Whole)];
    vec![
        small_lists::table("abmc_account_user", columns("account_id", "type"), accounts),
        small_lists::table("abmc_account_internal", account_totals, []),
        small_lists::table("abmc_item_user", columns("item_id", "item_type"), items),
        small_lists::table("abmc_item_internal", item_totals, []),
    ]
}

/// The table called `name` with `rows`, and with the columns `shared` and
/// those that a row has.
fn written(name: &str, shared: &[&'static str], rows: &[Row]) -> SyntheticTable {
    let mut columns = shared.to_vec();
    let own = rows.iter().flat_map(|row| &row.values);
    for (column, _) in own {
        if !columns.contains(column) {
            columns.push(column);
        }
    }
    let table = SyntheticTable::new(name);
    let table = columns
        .iter()
        .fold(table, |table, name| table.column(column(name)));
    let rows = rows.iter().map(|row| row.write(&columns));
    rows.fold(table, SyntheticTable::row)
}

/// What a column holds, which its name says.
#[derive(Clone, Copy)]
enum Holds {
    Flag,
    Number,
    Float,
    Day,
    Code,
    Text,
    Whole,
}

fn holds(column: &str) -> Holds {
    let ends_with = |endings: &[&str]| endings.iter().any(|ending| column.ends_with(ending));
    if ends_with(&["_bool"]) {
        Holds::Flag
    } else if ends_with(&["_amt", "_qnty", "_pct"]) {
        Holds::Number
    } else if column == "exchange_rate" {
        Holds::Float
    } else if ends_with(&["_date"]) {
        Holds::Day
    } else if ends_with(&["_type", "_action"]) {
        Holds::Code
    } else if ends_with(&["doc_num", "memo", "abm_id", "part_num", "other1"]) {
        Holds::Text
    } else {
        Holds::Whole
    }
}

fn column(name: &str) -> SyntheticColumn {
    let column = match holds(name) {
        Holds::Flag => SyntheticColumn::new(name, 24).width(1),
        // More digits than the file has, so that one can be too many.
        Holds::Number => SyntheticColumn::new(name, 3).width(20).scale(6),
        Holds::Float => SyntheticColumn::new(name, 4),
        Holds::Day => SyntheticColumn::new(name, 6),
        Holds::Code => SyntheticColumn::new(name, 1).width(2),
        Holds::Text => SyntheticColumn::new(name, 9).width(64),
        Holds::Whole => SyntheticColumn::new(name, 2),
    };
    column.nullable()
}

fn cell(column: &str, value: &str) -> Cell {
    match holds(column) {
        Holds::Flag => Cell::bit(value == "yes"),
        Holds::Number => Cell::decimal(value),
        Holds::Float => Cell::float(value.parse().unwrap()),
        Holds::Day => Cell::day(value),
        Holds::Code => Cell::small_int(value.parse().unwrap()),
        Holds::Text => Cell::text(value),
        Holds::Whole => Cell::integer(value.parse().unwrap()),
    }
}

fn default(column: &str) -> Cell {
    match (column, holds(column)) {
        ("exchange_rate", _) => Cell::float(1.0),
        ("last_auditable_action", _) => Cell::small_int(1),
        (_, Holds::Flag) => Cell::bit(false),
        _ => Cell::Null,
    }
}

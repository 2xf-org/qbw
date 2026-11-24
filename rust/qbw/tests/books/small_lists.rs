//! Small companies with a few rows in their lists, for what the books do not
//! have.

use qbw::Company;
use qbw_synthetic::{Cell, SyntheticColumn, SyntheticDatabase, SyntheticRow, SyntheticTable};

use crate::support;

/// What became of an entry.
#[derive(Clone, Copy)]
pub(crate) enum State {
    Shown,
    Hidden,
    Deleted,
}

impl State {
    /// The flags that a row has: whether it is hidden, and whether it is deleted.
    pub(crate) fn flags(self) -> [Cell; 2] {
        [
            Cell::bit(matches!(self, Self::Hidden)),
            Cell::bit(matches!(self, Self::Deleted)),
        ]
    }
}

/// The type of a column, which may hold nothing.
pub(crate) enum Kind {
    Whole,
    Number,
    Text,
    Flag,
    Day,
}

/// A table with `columns` and `rows`.
pub(crate) fn table<const COLUMNS: usize>(
    name: &str,
    columns: [(&str, Kind); COLUMNS],
    rows: impl IntoIterator<Item = [Cell; COLUMNS]>,
) -> SyntheticTable {
    let table = columns
        .into_iter()
        .map(|(name, kind)| column(name, kind))
        .fold(SyntheticTable::new(name), SyntheticTable::column);
    let rows = rows.into_iter().map(SyntheticRow::new);
    rows.fold(table, SyntheticTable::row)
}

fn column(name: &str, kind: Kind) -> SyntheticColumn {
    let column = match kind {
        Kind::Whole => SyntheticColumn::new(name, 2),
        Kind::Number => SyntheticColumn::new(name, 3).width(20).scale(5),
        Kind::Text => SyntheticColumn::new(name, 9).width(64),
        Kind::Flag => SyntheticColumn::new(name, 24).width(1),
        Kind::Day => SyntheticColumn::new(name, 6),
    };
    column.nullable()
}

/// A company with `tables`, and without accounts.
pub(crate) fn company(tables: impl IntoIterator<Item = SyntheticTable>) -> Company {
    let database = support::preferences(&[]).table(support::no_accounts());
    let database = tables.into_iter().fold(database, SyntheticDatabase::table);
    support::company(database)
}

/// A customer: its id, unless that cannot be read, its name and its state.
pub(crate) type Customer = (Option<i32>, &'static str, State);

/// A company with `customers`. The tables of its names have the columns that
/// every file has, and no others.
pub(crate) fn with_names(customers: &[Customer]) -> Company {
    company(name_tables(customers))
}

/// The tables of [`with_names`].
pub(crate) fn name_tables(customers: &[Customer]) -> Vec<SyntheticTable> {
    let rows = customers.iter().map(|&(id, name, state)| {
        let id = id.map_or(Cell::Null, Cell::integer);
        let [is_hidden, is_deleted] = state.flags();
        [id, Cell::text(name), Cell::Null, is_hidden, is_deleted]
    });
    let customers = table(
        "abmc_customer_user",
        [
            ("customer_id", Kind::Whole),
            ("name", Kind::Text),
            ("parent_id", Kind::Whole),
            ("is_hidden_bool", Kind::Flag),
            ("is_deleted_bool", Kind::Flag),
        ],
        rows,
    );
    vec![
        customers,
        totals("abmc_customer_internal", "customer_id"),
        no_names("abmc_vendor_user", "vendor_id"),
        totals("abmc_vendor_internal", "vendor_id"),
        no_names("abmc_employee", "employee_id"),
        no_names("abmc_other_user", "other_id"),
        totals("abmc_other_internal", "other_id"),
    ]
}

fn no_names(name: &str, key: &str) -> SyntheticTable {
    let columns = [
        (key, Kind::Whole),
        ("name", Kind::Text),
        ("is_hidden_bool", Kind::Flag),
        ("is_deleted_bool", Kind::Flag),
    ];
    table(name, columns, [])
}

fn totals(name: &str, key: &str) -> SyntheticTable {
    table(name, [(key, Kind::Whole)], [])
}

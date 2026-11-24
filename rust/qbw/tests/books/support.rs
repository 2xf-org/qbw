//! The companies that the tests read.

use qbw::Company;
use qbw::sa17::{Database, Source};
use qbw_synthetic::{Cell, SyntheticColumn, SyntheticDatabase, SyntheticRow, SyntheticTable};

/// A flag on a row that no reader knows, which makes the row unreadable.
const UNKNOWN_FLAG: u8 = 0x10;

/// The company of `qbw_synthetic::company::books`.
pub(crate) fn books() -> Company {
    Company::from_bytes(qbw_synthetic::company::books().build()).unwrap()
}

/// The same company, damaged.
pub(crate) fn damaged_books() -> Company {
    Company::from_bytes(qbw_synthetic::company::damaged_books().build()).unwrap()
}

/// The company in `database`, with an empty table for each table that every file
/// has of transactions, of their links and of who they are with.
pub(crate) fn company(database: SyntheticDatabase) -> Company {
    let found = Database::from_bytes(database.build()).unwrap();
    let all = Database::from_bytes(qbw_synthetic::company::books().build()).unwrap();
    let is_missing = |name: &str| is_in_every_file(name) && found.definition(name).is_err();
    let missing = all.definitions().iter();
    let missing = missing.filter(|table| is_missing(table.name()));
    let database = missing.fold(database, |database, table| {
        let columns = table.columns().iter();
        let columns = columns.map(|column| SyntheticColumn::new(column.name(), 2));
        database.table(columns.fold(SyntheticTable::new(table.name()), SyntheticTable::column))
    });
    Company::from_bytes(database.build()).unwrap()
}

fn is_in_every_file(table: &str) -> bool {
    table.ends_with("_header")
        || table.ends_with("_lineitem")
        || [
            "abmc_transaction_link",
            "abmc_customer_user",
            "abmc_vendor_user",
        ]
        .contains(&table)
}

/// A company with the accounts 1 and 2, of which the second cannot be read.
pub(crate) fn with_a_damaged_account() -> Company {
    let account = |id, kind| {
        let off = || Cell::bit(false);
        let fields = [
            Cell::integer(id),
            Cell::text("Till"),
            Cell::Null,
            kind,
            off(),
            off(),
        ];
        SyntheticRow::new(fields)
    };
    let accounts = SyntheticTable::new("abmc_account_user")
        .column(SyntheticColumn::new("account_id", 2))
        .column(SyntheticColumn::new("name", 9).width(32))
        .column(SyntheticColumn::new("parent_id", 2).nullable())
        .column(SyntheticColumn::new("type", 1).width(2).nullable())
        .column(SyntheticColumn::new("is_hidden_bool", 24).width(1))
        .column(SyntheticColumn::new("is_deleted_bool", 24).width(1))
        .row(account(1, Cell::small_int(0)))
        .row(account(2, Cell::Null));
    let totals = SyntheticTable::new("abmc_account_internal")
        .column(SyntheticColumn::new("account_id", 2))
        .column(SyntheticColumn::new("special_type", 1).width(2))
        .column(SyntheticColumn::new("display_index_next", 2));
    company(preferences(&[]).table(accounts).table(totals))
}

/// A company without accounts, with the preferences `values`.
pub(crate) fn with_preferences(values: &[(&str, &str)]) -> Company {
    company(preferences(values).table(no_accounts()))
}

/// The same, with one more preference, which cannot be read.
pub(crate) fn with_a_damaged_preference(values: &[(&str, &str)]) -> Company {
    let [names, stored] = preference_tables(values);
    let lost = [Cell::integer(1), Cell::Null, Cell::text("lost")];
    let database = SyntheticDatabase::new()
        .table(no_accounts())
        .table(names)
        .table(stored.row(SyntheticRow::new(lost).flags(UNKNOWN_FLAG)));
    company(database)
}

/// A database with the preferences `values`.
pub(crate) fn preferences(values: &[(&str, &str)]) -> SyntheticDatabase {
    let tables = preference_tables(values).into_iter();
    tables.fold(SyntheticDatabase::new(), SyntheticDatabase::table)
}

/// The table of accounts, with no row and one column.
pub(crate) fn no_accounts() -> SyntheticTable {
    SyntheticTable::new("abmc_account_user").column(SyntheticColumn::new("account_id", 2))
}

fn preference_tables(values: &[(&str, &str)]) -> [SyntheticTable; 2] {
    let mut names = SyntheticTable::new("cf_pref_attribute")
        .column(SyntheticColumn::new("preference_id", 2))
        .column(SyntheticColumn::new("preference_name", 9).width(2048));
    let mut stored = SyntheticTable::new("cf_pref_attribute_value")
        .column(SyntheticColumn::new("preference_id", 2))
        .column(SyntheticColumn::new("user_name", 9).width(128).nullable())
        .column(
            SyntheticColumn::new("preference_value", 9)
                .width(32000)
                .nullable(),
        );
    for (id, (name, value)) in (1..).zip(values) {
        let id = Cell::integer(id);
        names = names.row(SyntheticRow::new([id.clone(), Cell::text(name)]));
        stored = stored.row(SyntheticRow::new([id, Cell::Null, Cell::text(value)]));
    }
    [names, stored]
}

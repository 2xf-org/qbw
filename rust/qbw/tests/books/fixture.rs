//! The synthetic companies themselves: their tables hold together as a file's do.

use std::collections::HashMap;

use qbw::sa17::{Columns, Database, Row, Table, Value};
use qbw_synthetic::company;

use crate::support;

/// The company of the books, and the one of one currency.
fn databases() -> [Database; 2] {
    [company::books(), company::one_currency_books()]
        .map(|company| Database::from_bytes(company.build()).unwrap())
}

fn rows(table: Table<'_>) -> Vec<Row> {
    table.rows().collect::<Result<_, _>>().unwrap()
}

/// The rows of a table, cut down to `columns` and read as whole numbers.
fn numbers(database: &Database, table: &str, columns: &[&str]) -> Vec<Vec<Option<i64>>> {
    let table = database.table(table).unwrap();
    let columns = Columns::new(table.definition(), columns).unwrap();
    let rows = table.scan(&columns).map(|row| {
        let row = row.unwrap();
        row.values().iter().map(Value::as_i64).collect()
    });
    rows.collect()
}

fn values(row: &Row, columns: &[usize]) -> Vec<Value> {
    columns
        .iter()
        .map(|&column| row.value(column).unwrap().clone())
        .collect()
}

#[test]
fn every_row_has_a_primary_key_of_its_own() {
    let databases = databases();
    let tables = databases.iter().flat_map(Database::tables);
    let tables = tables.filter(|table| table.definition().stores_rows());
    for table in tables.filter(|table| !table.definition().is_system()) {
        let key = table.definition().primary_key().unwrap();
        let mut keys: Vec<Vec<Value>> = Vec::new();
        for row in rows(table) {
            let key = values(&row, key);
            assert!(!keys.contains(&key), "{}: {key:?}", table.name());
            keys.push(key);
        }
    }
}

#[test]
fn every_foreign_key_leads_to_a_row() {
    let mut followed = 0;
    for table in databases().iter().flat_map(Database::tables) {
        for key in table.definition().foreign_keys() {
            let target = table.target(key).unwrap();
            let there: Vec<Vec<Value>> = rows(target)
                .iter()
                .map(|row| values(row, key.target_positions()))
                .collect();
            for row in rows(table) {
                let value = values(&row, key.positions());
                if value != [Value::Null] {
                    assert!(there.contains(&value), "{}: {value:?}", table.name());
                    followed += 1;
                }
            }
        }
    }
    assert!(followed > 350, "only {followed} keys were followed");
}

/// Columns that hold a key and declare none: the column of any table, then the
/// table and the column it leads to.
const UNDECLARED: &[(&str, &str, &str)] = &[
    ("contact_id", "abmc_contacts", "contacts_abm_id"),
    ("ship_to_id", "abmc_ship_to", "ship_to_abm_id"),
    ("ship_to_abm_id", "abmc_ship_to", "ship_to_abm_id"),
    ("ship_via_id", "abmc_shipping", "shipping_id"),
    ("terms_id", "abmc_terms", "terms_id"),
    ("sibling_account_id", "abmc_account_user", "account_id"),
    ("preferred_vendor_id", "abmc_vendor_user", "vendor_id"),
    ("tax_agency_id", "abmc_vendor_user", "vendor_id"),
    ("sub_item_id", "abmc_item_user", "item_id"),
];

#[test]
fn every_key_that_is_not_declared_leads_to_a_row() {
    let databases = databases();
    let columns = UNDECLARED.iter();
    let columns =
        columns.flat_map(|column| databases.iter().map(move |database| (database, column)));
    for (database, (column, target, target_column)) in columns {
        let read = |table: Table<'_>, column: &str| -> Vec<Value> {
            let position = table.definition().position(column).unwrap();
            let rows = rows(table).into_iter();
            rows.map(|row| row.into_values().swap_remove(position))
                .collect()
        };
        let mut there = read(database.table(target).unwrap(), target_column);
        there.extend([Value::Null, Value::Integer(0)]);
        let having = database.tables().filter(|table| {
            let definition = table.definition();
            definition.stores_rows() && definition.position(column).is_some()
        });
        for table in having {
            for value in read(table, column) {
                assert!(there.contains(&value), "{}: {value:?}", table.name());
            }
        }
    }
}

#[test]
fn links_come_in_pairs_between_lines_of_their_transactions() {
    for (database, rows) in databases().iter().zip([28, 4]) {
        check_links(database, rows);
    }
}

/// Checks the links of a database, of which the table has so many rows.
fn check_links(database: &Database, rows: usize) {
    let mut transaction_of: HashMap<i64, i64> = HashMap::new();
    for table in database
        .tables()
        .filter(|table| table.name().ends_with("_lineitem"))
    {
        if table.definition().position("is_source_bool").is_none() {
            continue;
        }
        let lines = numbers(database, table.name(), &["target_id", "transaction_id"]);
        transaction_of.extend(
            lines
                .iter()
                .map(|line| (line[0].unwrap(), line[1].unwrap())),
        );
    }
    let links = numbers(
        database,
        "abmc_transaction_link",
        &[
            "transaction_id",
            "target_1_id",
            "transaction_2_id",
            "target_2_id",
        ],
    );
    assert_eq!(links.len(), rows);
    for pair in links.chunks(2) {
        let [link, mirror] = pair else {
            panic!("a link has a mirror");
        };
        assert_eq!((link[0], link[1]), (mirror[2], mirror[3]));
        assert_eq!((link[2], link[3]), (mirror[0], mirror[1]));
        for link in pair {
            assert_eq!(transaction_of.get(&link[1].unwrap()).copied(), link[0]);
        }
    }
}

#[test]
fn the_history_of_an_item_ends_with_what_is_on_hand() {
    let company = support::books();
    let database = company.source();
    let read = |table: &str, columns: &[&str]| -> Vec<Vec<Value>> {
        let table = database.table(table).unwrap();
        let columns = Columns::new(table.definition(), columns).unwrap();
        let rows = table.scan(&columns);
        rows.map(|row| row.unwrap().into_values()).collect()
    };
    let history = ["item_id", "quantity_on_hand", "value_on_hand"];
    let last: HashMap<i64, Vec<Value>> = read("abmc_item_history", &history)
        .into_iter()
        .map(|row| (row[0].as_i64().unwrap(), row))
        .collect();
    let totals = ["item_id", "on_hand_qnty", "value_on_hand_amt"];
    let stocked = read("abmc_item_internal", &totals);
    let stocked: Vec<Vec<Value>> = stocked
        .into_iter()
        .filter(|row| !row[1].is_null())
        .collect();
    assert_eq!(stocked.len(), last.len());
    for row in stocked {
        assert_eq!(last.get(&row[0].as_i64().unwrap()), Some(&row));
    }
}

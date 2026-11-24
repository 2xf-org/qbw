//! Every declared table and column, looked up.

mod books;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::{env, fs};

use sa17::{Database, Source};

use crate::damage::Damage;
use crate::error::Result;
use crate::tables::{
    StoredRow, Table, accounts, contacts, custom_fields, headers, information, items, lines, links,
    lists, name_currencies, names, postings, settings, stock,
};

/// A declared table, whatever its rows read as.
pub(super) trait Declared {
    fn name(&self) -> &'static str;

    /// The columns that the table has to have.
    fn required(&self) -> &'static [&'static str];

    /// The columns that it may lack.
    fn optional(&self) -> &'static [&'static str];

    /// Reads every row. `None` when `source` lacks the table.
    fn read(&self, source: &dyn Source) -> Result<Option<RowsRead>>;
}

/// What came of reading every row of a table.
#[derive(Debug)]
pub(super) struct RowsRead {
    rows: usize,
    damage: Vec<Damage>,
}

impl<R: StoredRow> Declared for Table<R> {
    fn name(&self) -> &'static str {
        Table::name(self)
    }

    fn required(&self) -> &'static [&'static str] {
        R::REQUIRED
    }

    fn optional(&self) -> &'static [&'static str] {
        R::OPTIONAL
    }

    fn read(&self, source: &dyn Source) -> Result<Option<RowsRead>> {
        let Some(table) = self.find(source)? else {
            return Ok(None);
        };
        let mut damage = Vec::new();
        let rows = table.scan(&mut damage).count();
        Ok(Some(RowsRead { rows, damage }))
    }
}

/// Every declared table.
fn declared() -> impl Iterator<Item = Box<dyn Declared>> {
    let families = [
        accounts::declared(),
        contacts::declared(),
        custom_fields::declared(),
        headers::declared(),
        information::declared(),
        items::declared(),
        lines::declared(),
        links::declared(),
        lists::declared(),
        name_currencies::declared(),
        names::declared(),
        postings::declared(),
        settings::declared(),
        stock::declared(),
    ];
    families.into_iter().flatten()
}

/// What came of looking every declared table up in one file.
#[derive(Debug, Default)]
struct LookedUp {
    tables: usize,
    rows: usize,
    missing_tables: Vec<&'static str>,
    /// Optional columns that no table of their shape has.
    missing_columns: Vec<&'static str>,
    damage: Vec<Damage>,
}

impl LookedUp {
    fn in_source(source: &dyn Source) -> Self {
        let mut looked_up = Self::default();
        // Tables of one shape, such as the lines of every kind, share their
        // optional columns, and each has those it has a use for.
        let mut found: BTreeMap<_, Vec<&'static str>> = BTreeMap::new();
        for table in declared() {
            let shape = (table.required(), table.optional());
            let found = found.entry(shape).or_default();
            let Some(read) = table.read(source).unwrap() else {
                looked_up.missing_tables.push(table.name());
                continue;
            };
            looked_up.tables += 1;
            looked_up.rows += read.rows;
            looked_up.damage.extend(read.damage);
            let definition = source.definition(table.name()).unwrap();
            let columns = table.optional().iter().copied();
            found.extend(columns.filter(|column| definition.position(column).is_some()));
        }
        for ((_, optional), found) in found {
            let columns = optional.iter().copied();
            let missing = columns.filter(|column| !found.contains(column));
            looked_up.missing_columns.extend(missing);
        }
        looked_up
    }
}

/// How the tables start that the books lack: those of the forms that nothing in
/// them is of.
const FORMS_WITHOUT_TABLES: &[&str] = &[
    "abmc_bill_ccard_",
    "abmc_customer_refund_",
    "abmc_inventory_transfer_",
    "abmc_sale_receipt_",
    "abmc_sales_receipt_",
    "abmc_statement_charge_",
    "es_",
];

fn books() -> Database {
    Database::from_bytes(qbw_synthetic::company::books().build()).unwrap()
}

#[test]
fn the_books_have_every_declared_column() {
    let looked_up = LookedUp::in_source(&books());
    assert_eq!(looked_up.missing_columns, Vec::<&str>::new());
    assert_eq!(looked_up.damage, Vec::new());
    let of_a_form = |table: &&str| {
        FORMS_WITHOUT_TABLES
            .iter()
            .any(|form| table.starts_with(form))
    };
    let others = looked_up.missing_tables.into_iter();
    let others: Vec<&str> = others.filter(|table| !of_a_form(table)).collect();
    assert_eq!(others, Vec::<&str>::new());
}

#[test]
fn a_table_or_a_column_is_named_where_it_is_declared_only() {
    let sources = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let declarations = sources.join("tables");
    let names: Vec<String> = declared()
        .flat_map(|table| [&[table.name()], table.required(), table.optional()].concat())
        // A column such as `name` is an everyday word, too.
        .filter(|name| name.contains('_'))
        .map(|name| format!("\"{name}\""))
        .collect();
    for path in rust_files(&sources) {
        if path.starts_with(&declarations) {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        // An example in a comment may go down to the tables.
        let code = text
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"));
        for line in code {
            let named = names.iter().find(|name| line.contains(*name));
            assert_eq!(named, None, "in {}", path.display());
        }
    }
}

#[test]
#[ignore = "reads the company files in the folder $QBW_SAMPLES"]
fn real_files_have_every_declared_column() {
    let folder = env::var_os("QBW_SAMPLES").expect("QBW_SAMPLES names a folder");
    let mut files = 0;
    for path in files_of(Path::new(&folder), "qbw") {
        let looked_up = LookedUp::in_source(&Database::open(&path).unwrap());
        println!(
            "{} tables, {} rows, {} damaged, missing {:?} and {:?}",
            looked_up.tables,
            looked_up.rows,
            looked_up.damage.len(),
            looked_up.missing_tables,
            looked_up.missing_columns,
        );
        assert_eq!(looked_up.missing_tables, Vec::<&str>::new());
        assert_eq!(looked_up.missing_columns, Vec::<&str>::new());
        assert_eq!(looked_up.damage.len(), 0);
        files += 1;
    }
    assert!(files > 0, "the folder has no company file");
}

/// Every Rust file under `folder`.
fn rust_files(folder: &Path) -> Vec<PathBuf> {
    let folders = entries(folder).into_iter().filter(|path| path.is_dir());
    let mut files = files_of(folder, "rs");
    files.extend(folders.flat_map(|folder| rust_files(&folder)));
    files
}

/// The files in `folder` that end with `extension`.
fn files_of(folder: &Path, extension: &str) -> Vec<PathBuf> {
    let entries = entries(folder).into_iter();
    let files = entries.filter(|path| {
        path.extension()
            .is_some_and(|found| found.eq_ignore_ascii_case(extension))
    });
    files.collect()
}

fn entries(folder: &Path) -> Vec<PathBuf> {
    let entries = fs::read_dir(folder).unwrap();
    entries.map(|entry| entry.unwrap().path()).collect()
}

//! A database that counts how many rows are looked up in it, and how many tables
//! are gone through.

use std::sync::atomic::{AtomicUsize, Ordering};

use qbw::Company;
use qbw::sa17::{Columns, Database, Result, Row, RowId, Source, TableDefinition, TextEncoding};
use qbw_synthetic::company;

/// A database that counts how many rows are looked up in it, and how many tables
/// are gone through and asked for by name. The rows that come with a whole table do not count.
pub(crate) struct Counted {
    database: Database,
    fetched: AtomicUsize,
    scanned: AtomicUsize,
    named: AtomicUsize,
}

/// The company of `qbw_synthetic::company::books`.
pub(crate) fn books() -> Company<Counted> {
    let database = Database::from_bytes(company::books().build()).unwrap();
    let source = Counted {
        database,
        fetched: AtomicUsize::new(0),
        scanned: AtomicUsize::new(0),
        named: AtomicUsize::new(0),
    };
    Company::try_new(source).unwrap()
}

impl Counted {
    /// How many rows were looked up so far.
    pub(crate) fn fetched(&self) -> usize {
        self.fetched.load(Ordering::Relaxed)
    }

    /// How many tables were gone through so far.
    pub(crate) fn scanned(&self) -> usize {
        self.scanned.load(Ordering::Relaxed)
    }

    /// How many tables were asked for by their names so far.
    pub(crate) fn named(&self) -> usize {
        self.named.load(Ordering::Relaxed)
    }
}

impl Source for Counted {
    fn definitions(&self) -> &[TableDefinition] {
        self.database.definitions()
    }

    fn scan<'source>(
        &'source self,
        table: &'source TableDefinition,
        columns: &'source Columns,
    ) -> Box<dyn Iterator<Item = Result<Row>> + Send + 'source> {
        self.scanned.fetch_add(1, Ordering::Relaxed);
        self.database.scan(table, columns)
    }

    fn fetch(&self, table: &TableDefinition, columns: &Columns, row: RowId) -> Result<Option<Row>> {
        self.fetched.fetch_add(1, Ordering::Relaxed);
        self.database.fetch(table, columns, row)
    }

    fn text_encoding(&self) -> TextEncoding {
        self.database.text_encoding()
    }

    fn definition(&self, name: &str) -> Result<&TableDefinition> {
        self.named.fetch_add(1, Ordering::Relaxed);
        self.database.definition(name)
    }
}

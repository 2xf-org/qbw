//! The company of `qbw_synthetic::company::books` with other rows.

use std::collections::{HashMap, HashSet};

use qbw::Company;
use qbw::sa17::{
    Columns, DataType, Database, Error, Result, Row, RowId, RowProblem, Source, TableDefinition,
    TextEncoding, Value,
};
use qbw_synthetic::company;

/// A row: its table, the column of its key, and the key.
pub(crate) type Key = (&'static str, &'static str, i64);

/// A cell: its table, the column of the row's key, the key, and its own column.
pub(crate) type Cell = (&'static str, &'static str, i64, &'static str);

/// What a column holds.
pub(crate) type Set = (&'static str, Value);

/// A database with other rows.
pub(crate) struct Changed {
    database: Database,
    changes: HashMap<(String, RowId), Vec<Change>>,
    /// The rows that are added to each table, with every column.
    added: HashMap<&'static str, Vec<(u32, Vec<Value>)>>,
    /// The tables that are not there.
    missing: HashSet<&'static str>,
}

/// What becomes of a row.
enum Change {
    Set(&'static str, Value),
    Deleted,
    Unreadable,
}

/// The company with `cells` changed.
pub(crate) fn books(cells: Vec<(Cell, Value)>) -> Company<Changed> {
    let source = Changed::books();
    let source = cells.into_iter().fold(
        source,
        |source, ((table, key_column, key, column), value)| {
            source.set((table, key_column, key), column, value)
        },
    );
    source.company()
}

/// A number.
pub(crate) fn number(text: &str) -> Value {
    Value::Decimal(text.parse().unwrap())
}

/// A whole number.
pub(crate) fn whole(number: i32) -> Value {
    Value::Integer(number.into())
}

/// A text.
pub(crate) fn text(text: &str) -> Value {
    Value::Text(text.to_owned())
}

/// A day.
pub(crate) fn date(text: &str) -> Value {
    Value::Date(text.parse().unwrap())
}

impl Changed {
    pub(crate) fn books() -> Self {
        Self {
            database: Database::from_bytes(company::books().build()).unwrap(),
            changes: HashMap::new(),
            added: HashMap::new(),
            missing: HashSet::new(),
        }
    }

    pub(crate) fn company(self) -> Company<Self> {
        Company::try_new(self).unwrap()
    }

    /// Puts another value into a column of a row.
    pub(crate) fn set(self, row: Key, column: &'static str, value: Value) -> Self {
        let old = self.find(row);
        assert!(old.get(column).is_some_and(|old| *old != value), "{column}");
        self.change(row, Change::Set(column, value))
    }

    /// Takes a row out.
    pub(crate) fn deleted(self, row: Key) -> Self {
        self.change(row, Change::Deleted)
    }

    /// Damages a row.
    pub(crate) fn unreadable(self, row: Key) -> Self {
        self.change(row, Change::Unreadable)
    }

    /// Adds a row that has what the row `like` has, but for `values`.
    pub(crate) fn copied(self, like: Key, values: &[Set]) -> Self {
        let (table, ..) = like;
        let row = self.find(like).into_values();
        self.with_row(table, row, values)
    }

    /// Adds a row that has `values`, and nothing else.
    pub(crate) fn added(self, table: &'static str, values: &[Set]) -> Self {
        let columns = self.definition(table).unwrap().columns().iter();
        let row = columns.map(|column| match column.data_type() {
            _ if column.is_nullable() => Value::Null,
            DataType::Bit => Value::Boolean(false),
            DataType::Float => Value::Float(1.0),
            DataType::VarChar | DataType::Char => Value::Text(String::new()),
            _ => Value::Integer(0),
        });
        let row = row.collect();
        self.with_row(table, row, values)
    }

    /// Takes a table out.
    pub(crate) fn without(mut self, table: &'static str) -> Self {
        assert!(self.definition(table).is_ok(), "{table}");
        self.missing.insert(table);
        self
    }

    fn with_row(mut self, table: &'static str, mut row: Vec<Value>, values: &[Set]) -> Self {
        let definition = self.definition(table).unwrap();
        for (column, value) in values {
            row[definition.position(column).unwrap()] = value.clone();
        }
        let rows = self.added.values().map(Vec::len).sum::<usize>();
        let id = u32::try_from(rows).unwrap();
        self.added.entry(table).or_default().push((id, row));
        self
    }

    fn change(mut self, row: Key, change: Change) -> Self {
        let (table, ..) = row;
        let place = (table.to_owned(), self.find(row).id());
        self.changes.entry(place).or_default().push(change);
        self
    }

    fn find(&self, (table, key_column, key): Key) -> Row {
        let mut rows = self.database.table(table).unwrap().rows();
        let row = rows.find_map(|row| {
            let row = row.unwrap();
            (row.get(key_column).and_then(Value::as_i64) == Some(key)).then_some(row)
        });
        row.expect("there is such a row")
    }

    /// The row as it became, unless it was deleted.
    fn changed(&self, table: &TableDefinition, mut row: Row) -> Result<Option<Row>> {
        let place = (table.name().to_owned(), row.id());
        for change in self.changes.get(&place).into_iter().flatten() {
            match change {
                Change::Set(column, value) => {
                    if let Some(position) = row.columns().position(column) {
                        row.values_mut()[position] = value.clone();
                    }
                }
                Change::Deleted => return Ok(None),
                Change::Unreadable => {
                    let address = row.id().address().unwrap();
                    return Err(Error::Row {
                        table: table.name().to_owned(),
                        address,
                        problem: RowProblem::Columns,
                    });
                }
            }
        }
        Ok(Some(row))
    }

    /// The rows that are added to a table, cut down to `columns`.
    fn added_to(&self, table: &TableDefinition, columns: &Columns) -> impl Iterator<Item = Row> {
        let added = self.added.get(table.name()).into_iter().flatten();
        added.map(|(id, values)| {
            let values = columns.names().map(|name| {
                let position = table.position(name);
                position.map_or(Value::Null, |position| values[position].clone())
            });
            Row::new(RowId::Added(*id), columns, values.collect()).unwrap()
        })
    }
}

impl Source for Changed {
    fn definition(&self, name: &str) -> Result<&TableDefinition> {
        if self.missing.contains(name) {
            return Err(Error::UnknownTable {
                name: name.to_owned(),
            });
        }
        self.database.definition(name)
    }

    fn definitions(&self) -> &[TableDefinition] {
        self.database.definitions()
    }

    fn scan<'source>(
        &'source self,
        table: &'source TableDefinition,
        columns: &'source Columns,
    ) -> Box<dyn Iterator<Item = Result<Row>> + Send + 'source> {
        let rows = self.database.scan(table, columns);
        let rows = rows.filter_map(|row| match row {
            Ok(row) => self.changed(table, row).transpose(),
            Err(error) => Some(Err(error)),
        });
        let added: Vec<_> = self.added_to(table, columns).map(Ok).collect();
        Box::new(rows.chain(added))
    }

    fn fetch(&self, table: &TableDefinition, columns: &Columns, row: RowId) -> Result<Option<Row>> {
        if matches!(row, RowId::Added(_)) {
            let mut added = self.added_to(table, columns);
            return Ok(added.find(|added| added.id() == row));
        }
        match self.database.fetch(table, columns, row)? {
            Some(row) => self.changed(table, row),
            None => Ok(None),
        }
    }

    fn text_encoding(&self) -> TextEncoding {
        self.database.text_encoding()
    }
}

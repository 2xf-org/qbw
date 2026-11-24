//! Reading a declared table.

use std::marker::PhantomData;
use std::result;

use sa17::{Columns, Error as DatabaseError, Row, RowId, Source, TableDefinition};

use crate::damage::Damage;
use crate::error::{Error, Result};
use crate::origin::Origin;
use crate::tables::{Fields, StoredRow};

/// The table of `source` called `name`, or `None` when it has none. No two
/// tables of a company file share a name.
///
/// # Errors
///
/// Returns [`Error::Database`] when more than one table is called that.
pub(crate) fn definition<'source>(
    source: &'source dyn Source,
    name: &str,
) -> Result<Option<&'source TableDefinition>> {
    match source.definition(name) {
        Ok(definition) => Ok(Some(definition)),
        Err(DatabaseError::UnknownTable { .. }) => Ok(None),
        Err(error) => Err(Error::Database(error)),
    }
}

/// A table of the file whose rows read as `R`.
pub(crate) struct Table<R> {
    name: &'static str,
    rows: PhantomData<fn() -> R>,
}

impl<R: StoredRow> Table<R> {
    pub(crate) const fn new(name: &'static str) -> Self {
        Self {
            name,
            rows: PhantomData,
        }
    }

    pub(crate) const fn name(&self) -> &'static str {
        self.name
    }

    /// The table of `source`, with the columns that `R` reads found.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoTable`] when the source has no such table, and
    /// [`Error::Schema`] when it lacks a column that `R` requires.
    pub(crate) fn open<'source>(
        &self,
        source: &'source dyn Source,
    ) -> Result<OpenTable<'source, R>> {
        let table = self.name;
        self.find(source)?.ok_or(Error::NoTable { table })
    }

    /// The table of `source`, or `None` when it has no such table, as a file
    /// of an older release may not.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Schema`] when the table lacks a column that `R`
    /// requires, and [`Error::Database`] when more than one table has its name.
    pub(crate) fn find<'source>(
        &self,
        source: &'source dyn Source,
    ) -> Result<Option<OpenTable<'source, R>>> {
        let Some(definition) = definition(source, self.name)? else {
            return Ok(None);
        };
        let columns = self.columns(definition)?;
        Ok(Some(self.opened(source, definition, columns)))
    }

    /// The table of `source` with a `definition` and with columns that were found
    /// before.
    pub(super) fn opened<'source>(
        &self,
        source: &'source dyn Source,
        definition: &'source TableDefinition,
        columns: Columns,
    ) -> OpenTable<'source, R> {
        OpenTable {
            name: self.name,
            source,
            definition,
            columns,
            rows: PhantomData,
        }
    }

    fn columns(&self, definition: &TableDefinition) -> Result<Columns> {
        let columns = Columns::with_optional(definition, R::REQUIRED, R::OPTIONAL);
        columns.map_err(|error| {
            let lacks = |column: &&&str| definition.position(column).is_none();
            match R::REQUIRED.iter().find(lacks) {
                Some(column) => Error::Schema {
                    table: self.name,
                    column,
                },
                None => Error::Database(error),
            }
        })
    }
}

/// A declared table of one source, to read rows from.
pub(crate) struct OpenTable<'source, R> {
    name: &'static str,
    source: &'source dyn Source,
    definition: &'source TableDefinition,
    columns: Columns,
    rows: PhantomData<fn() -> R>,
}

impl<R: StoredRow> OpenTable<'_, R> {
    pub(super) fn id(&self) -> u32 {
        self.definition.id()
    }

    pub(super) fn columns(&self) -> Columns {
        self.columns.clone()
    }

    /// Every row that can be read, with where it came from. What cannot be
    /// read is added to `damage`.
    pub(crate) fn scan<'scan>(
        &'scan self,
        damage: &'scan mut Vec<Damage>,
    ) -> impl Iterator<Item = (Origin, R)> + 'scan {
        let rows = self.source.scan(self.definition, &self.columns);
        rows.filter_map(|row| match self.read(row) {
            Ok(found) => Some(found),
            Err(found) => {
                damage.push(found);
                None
            }
        })
    }

    /// The row `row`, or `None` when the source has no such row, or no longer.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Damaged`] when the row cannot be read.
    pub(crate) fn fetch(&self, row: RowId) -> Result<Option<R>> {
        let row = self.source.fetch(self.definition, &self.columns, row);
        let Some(row) = row.transpose() else {
            return Ok(None);
        };
        let (_, read) = self.read(row).map_err(Error::Damaged)?;
        Ok(Some(read))
    }

    fn read(&self, row: result::Result<Row, DatabaseError>) -> result::Result<(Origin, R), Damage> {
        let row = row.map_err(|error| Damage::of_scan(self.name, &error))?;
        match R::read(&mut Fields::new(&row)) {
            Ok(read) => Ok((Origin::new(self.name, row.id()), read)),
            Err(problem) => Err(Damage::new(self.name, Some(row.id()), problem)),
        }
    }
}

#[cfg(test)]
mod tests {
    use qbw_synthetic::{Cell, SyntheticColumn, SyntheticDatabase, SyntheticRow, SyntheticTable};
    use sa17::{Database, ValueProblem};

    use super::*;
    use crate::damage::Problem;
    use crate::ids::ItemId;
    use crate::tables::stored_row;

    stored_row! {
        /// A plant, with a column that only newer gardens have.
        pub(crate) struct PlantRow {
            required {
                id: ItemId = "plant_id",
                name: Option<String> = "plant_name",
            }
            optional {
                color: Option<String> = "plant_color",
            }
        }
    }

    const PLANTS: Table<PlantRow> = Table::new("plants");

    fn database(plants: SyntheticTable) -> Database {
        Database::from_bytes(SyntheticDatabase::new().table(plants).build()).unwrap()
    }

    /// A fern, a row of no plant, a row with a flag that no reader knows, and
    /// a moss.
    fn garden() -> Database {
        let plants = SyntheticTable::new("plants")
            .column(SyntheticColumn::new("plant_name", 9).width(20).nullable())
            .column(SyntheticColumn::new("plant_id", 2).nullable())
            .row(SyntheticRow::new([Cell::text("Fern"), Cell::integer(1)]))
            .row(SyntheticRow::new([Cell::text("Nothing"), Cell::Null]))
            .row(SyntheticRow::new([Cell::text("Lost"), Cell::integer(3)]).flags(0x10))
            .row(SyntheticRow::new([Cell::text(""), Cell::integer(4)]));
        database(plants)
    }

    #[test]
    fn a_scan_reads_the_rows_and_collects_the_damage() {
        let garden = garden();
        let plants = PLANTS.open(&garden).unwrap();
        let mut damage = Vec::new();
        let (origins, read): (Vec<Origin>, Vec<PlantRow>) = plants.scan(&mut damage).unzip();
        let read: Vec<_> = read
            .into_iter()
            .map(|row| (row.id, row.name, row.color))
            .collect();
        let fern = Some("Fern".to_owned());
        assert_eq!(
            read,
            [(ItemId::new(1), fern, None), (ItemId::new(4), None, None)]
        );

        let [nothing, lost] = damage.as_slice() else {
            panic!("two rows are damaged: {damage:?}");
        };
        let expected = ValueProblem::Null { expected: "i64" };
        assert_eq!(
            nothing.problem,
            Problem::Column {
                column: "plant_id",
                found: expected
            }
        );
        assert!(matches!(lost.problem, Problem::Row { .. }));
        assert!(origins.iter().all(|origin| Some(origin.row) != lost.row));
    }

    #[test]
    fn a_row_is_read_by_where_it_is() {
        let garden = garden();
        let plants = PLANTS.open(&garden).unwrap();
        let mut damage = Vec::new();
        let origins: Vec<Origin> = plants.scan(&mut damage).map(|(origin, _)| origin).collect();
        let moss = plants.fetch(origins[1].row).unwrap();
        assert_eq!(moss.map(|row| row.id), Some(ItemId::new(4)));
        assert!(plants.fetch(RowId::Added(1)).unwrap().is_none());
        let nothing = damage[0].row.unwrap();
        let error = plants.fetch(nothing).unwrap_err();
        assert!(matches!(error, Error::Damaged(found) if found == damage[0]));
    }

    #[test]
    fn a_table_lacks_a_required_column_or_is_not_there() {
        let nameless = SyntheticTable::new("plants").column(SyntheticColumn::new("plant_id", 2));
        let Err(Error::Schema { table, column }) = PLANTS.open(&database(nameless)).map(|_| ())
        else {
            panic!("the table has no column for the name");
        };
        assert_eq!((table, column), ("plants", "plant_name"));

        let stones = database(SyntheticTable::new("stones").column(SyntheticColumn::new("id", 2)));
        assert!(PLANTS.find(&stones).unwrap().is_none());
        let error = PLANTS.open(&stones).map(|_| ()).unwrap_err();
        assert!(matches!(error, Error::NoTable { table: "plants" }));
    }
}

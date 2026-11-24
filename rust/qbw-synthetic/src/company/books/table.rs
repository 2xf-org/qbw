//! Tables to write, from a description of their columns.

use crate::cell::Cell;
use crate::company::books::literal::{Literal, Values};
use crate::synthetic_table::{SyntheticColumn, SyntheticRow, SyntheticTable};

/// A flag on a row that no reader knows, which makes the row unreadable.
const UNKNOWN_FLAG: u8 = 0x10;

/// A table of the company's, with its rows.
pub(crate) struct Table {
    name: String,
    columns: Vec<Column>,
    rows: Vec<Row>,
}

struct Row {
    values: Vec<Literal>,
    is_unreadable: bool,
}

struct Column {
    name: &'static str,
    kind: Kind,
    is_nullable: bool,
    is_key: bool,
    target: Option<String>,
    default: Literal,
}

/// The type of a column. A number with a fraction has 20 digits, five of them
/// after the point, in every table of a company.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    SmallInt,
    Integer,
    Numeric,
    Float,
    Date,
    Bit,
    VarChar(u32),
    Char(u32),
}

impl Table {
    /// A table without rows, with the columns that `descriptions` describe one
    /// after the other. Each describes one column per line:
    ///
    /// ```text
    /// account_id integer key
    /// parent_id integer null
    /// currency_id integer null -> abmc_currency = 1
    /// ```
    ///
    /// `key` is a column of the primary key, `null` one that may hold nothing,
    /// `->` names the table whose row it holds the key of, and `=` gives the
    /// whole number that a row has unless it says otherwise. Without it, that
    /// is `NULL`, or where the column is not nullable 0, off or an empty text.
    pub(crate) fn new(name: &str, descriptions: &[&'static str]) -> Self {
        let lines = descriptions.iter().flat_map(|columns| columns.lines());
        let lines = lines.map(str::trim).filter(|line| !line.is_empty());
        Self {
            name: name.to_owned(),
            columns: lines.map(Column::parse).collect(),
            rows: Vec::new(),
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// Declares that `column` holds the key of a row of `target`.
    pub(crate) fn referring(mut self, column: &str, target: &str) -> Self {
        let place = self.place(column);
        self.columns[place].target = Some(target.to_owned());
        self
    }

    /// Adds a row.
    pub(crate) fn row(self, values: Values) -> Self {
        self.rows(Vec::new(), [values])
    }

    /// Adds rows that have the values `shared` and, over them, their own.
    pub(crate) fn rows(mut self, shared: Values, rows: impl IntoIterator<Item = Values>) -> Self {
        for own in rows {
            let defaults = self.columns.iter().map(|column| column.default);
            let mut values: Vec<Literal> = defaults.collect();
            for (column, value) in shared.iter().chain(&own) {
                values[self.place(column)] = *value;
            }
            self.rows.push(Row {
                values,
                is_unreadable: false,
            });
        }
        self
    }

    /// The same table without its rows.
    pub(crate) fn emptied(mut self) -> Self {
        self.rows.clear();
        self
    }

    /// The same table, whose rows say no currency unless they say which.
    pub(crate) fn without_currency(mut self) -> Self {
        let mut columns = self.columns.iter_mut();
        if let Some(currency) = columns.find(|column| column.name == "currency_id") {
            currency.default = Literal::Null;
        }
        self
    }

    /// Damages the row whose `key` column holds `number`.
    pub(crate) fn make_unreadable(&mut self, key: &str, number: i32) {
        self.found(key, number).is_unreadable = true;
    }

    /// Changes the value of `column` in the row whose `key` column holds `number`.
    pub(crate) fn set(&mut self, key: &str, number: i32, column: &str, value: impl Into<Literal>) {
        let place = self.place(column);
        self.found(key, number).values[place] = value.into();
    }

    /// The table as the writer takes it.
    pub(crate) fn write(&self) -> SyntheticTable {
        let mut table = SyntheticTable::new(&self.name);
        for column in &self.columns {
            table = table.column(column.write());
        }
        let key = self.columns.iter().filter(|column| column.is_key);
        let key: Vec<&str> = key.map(|column| column.name).collect();
        table = table.primary_key(&key);
        for column in &self.columns {
            if let Some(target) = &column.target {
                table = table.foreign_key(column.name, &[column.name], target);
            }
        }
        for row in &self.rows {
            let cells = self.columns.iter().zip(&row.values);
            let cells = cells.map(|(column, value)| column.cell(*value, &self.name));
            let written = SyntheticRow::new(cells);
            table = table.row(if row.is_unreadable {
                written.flags(UNKNOWN_FLAG)
            } else {
                written
            });
        }
        table
    }

    fn place(&self, column: &str) -> usize {
        let mut columns = self.columns.iter();
        let place = columns.position(|known| known.name == column);
        place.unwrap_or_else(|| panic!("the table {} has no column {column}", self.name))
    }

    fn found(&mut self, key: &str, number: i32) -> &mut Row {
        let place = self.place(key);
        let holds = |row: &&mut Row| matches!(row.values[place], Literal::Whole(n) if n == number);
        let found = self.rows.iter_mut().find(holds);
        found.unwrap_or_else(|| panic!("the table {} has no row {number}", self.name))
    }
}

impl Column {
    fn parse(line: &'static str) -> Self {
        let mut words = line.split_whitespace();
        let (Some(name), Some(kind)) = (words.next(), words.next()) else {
            panic!("a column has a name and a type: {line}");
        };
        let mut column = Self {
            name,
            kind: Kind::parse(kind),
            is_nullable: false,
            is_key: false,
            target: None,
            default: Literal::Null,
        };
        let mut default = None;
        while let Some(word) = words.next() {
            match word {
                "key" => column.is_key = true,
                "null" => column.is_nullable = true,
                "->" => column.target = words.next().map(str::to_owned),
                "=" => default = words.next(),
                _ => panic!("a column has no {word}: {line}"),
            }
        }
        column.default = match default {
            Some(number) if column.kind == Kind::Float => Literal::Text(number),
            Some(number) => Literal::Whole(number.parse().expect("a default is a whole number")),
            None if column.is_nullable => Literal::Null,
            None => column.kind.nothing(),
        };
        column
    }

    fn write(&self) -> SyntheticColumn {
        let column = match self.kind {
            Kind::SmallInt => SyntheticColumn::new(self.name, 1).width(2),
            Kind::Integer => SyntheticColumn::new(self.name, 2),
            Kind::Numeric => SyntheticColumn::new(self.name, 3).width(20).scale(5),
            Kind::Float => SyntheticColumn::new(self.name, 4),
            Kind::Date => SyntheticColumn::new(self.name, 6),
            Kind::Bit => SyntheticColumn::new(self.name, 24).width(1),
            Kind::VarChar(width) => SyntheticColumn::new(self.name, 9).width(width),
            Kind::Char(width) => SyntheticColumn::new(self.name, 8).width(width),
        };
        if self.is_nullable {
            column.nullable()
        } else {
            column
        }
    }

    /// The field that a row of `table` stores for `value`.
    fn cell(&self, value: Literal, table: &str) -> Cell {
        match (self.kind, value) {
            (_, Literal::Null) => Cell::Null,
            (Kind::SmallInt, Literal::Whole(number)) => {
                Cell::small_int(i16::try_from(number).expect("the number fits a smallint"))
            }
            (Kind::Integer, Literal::Whole(number)) => Cell::integer(number),
            (Kind::Bit, Literal::Flag(flag)) => Cell::bit(flag),
            (Kind::Numeric, Literal::Text(number)) => Cell::decimal(number),
            (Kind::Float, Literal::Text(number)) => {
                Cell::float(number.parse().expect("a float is a number"))
            }
            (Kind::Date, Literal::Text(day)) => Cell::day(day),
            (Kind::VarChar(_) | Kind::Char(_), Literal::Text(text)) => Cell::text(text),
            (Kind::VarChar(_) | Kind::Char(_), Literal::Bytes(bytes)) => Cell::bytes(bytes),
            (kind, value) => panic!("{table}.{} is {kind:?}, not {value:?}", self.name),
        }
    }
}

impl Kind {
    fn parse(text: &str) -> Self {
        let (name, width) = match text.split_once('(') {
            Some((name, width)) => (name, width.trim_end_matches(')')),
            None => (text, ""),
        };
        let width = || width.parse().expect("the type has a width");
        match name {
            "smallint" => Self::SmallInt,
            "integer" => Self::Integer,
            "numeric" => Self::Numeric,
            "float" => Self::Float,
            "date" => Self::Date,
            "bit" => Self::Bit,
            "varchar" => Self::VarChar(width()),
            "char" => Self::Char(width()),
            _ => panic!("there is no type {text}"),
        }
    }

    /// What a column that is not nullable holds when nothing was said.
    fn nothing(self) -> Literal {
        match self {
            Self::SmallInt | Self::Integer => Literal::Whole(0),
            Self::Numeric | Self::Float => Literal::Text("0"),
            Self::Date => Literal::Text("1970-01-01"),
            Self::Bit => Literal::Flag(false),
            Self::VarChar(_) | Self::Char(_) => Literal::Text(""),
        }
    }
}

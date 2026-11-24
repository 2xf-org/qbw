//! The tables of a company file, and what this crate reads of them.
//!
//! The modules below, one for each family of tables, are the only place in
//! the crate where a table or a column of the file is named. The rest of the
//! crate sees the structs that their rows read as. A family declares
//!
//! - its rows, with `stored_row!`: a struct whose fields name the columns they
//!   are read from. A table that lacks a required column is an
//!   [`Error::Schema`](crate::Error::Schema), and an optional column that it
//!   lacks reads as nothing. A column is required when every file that was
//!   looked at has it and no record makes sense without it;
//! - its tables, as constants of [`Table`]. [`Table::open`] is for a table
//!   that every company file has, and [`Table::find`] for the others;
//! - for the tests, `declared`, which lists the tables. The tests look every
//!   column up in the synthetic company and in real files.
//!
//! To add a family, write a module beside the others, as in `accounts.rs`,
//! give it a `mod` line here, and put its `declared` in the list in
//! `tests.rs`. Until it is there, `declared` is dead code.

pub(crate) mod accounts;
pub(crate) mod contacts;
pub(crate) mod custom_fields;
pub(crate) mod headers;
pub(crate) mod information;
pub(crate) mod items;
pub(crate) mod lines;
pub(crate) mod links;
pub(crate) mod lists;
pub(crate) mod name_currencies;
pub(crate) mod names;
pub(crate) mod postings;
pub(crate) mod settings;
pub(crate) mod stock;
pub(crate) mod transactions;

mod field;
mod form;
mod opened;
mod stored_row;
pub(crate) mod table;

pub(crate) use field::Field;
pub(crate) use form::{FORMS, Form};
pub(crate) use opened::OpenedTables;
pub(crate) use stored_row::{Fields, StoredRow, stored_row};
pub(crate) use table::{OpenTable, Table};

#[cfg(test)]
mod tests;

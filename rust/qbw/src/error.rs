//! The error type of every fallible operation in the crate.

use sa17::Error as DatabaseError;

use crate::damage::Damage;

/// A `Result` whose error is [`enum@Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Why a company file, or one part of it, could not be read.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The database could not be opened or read.
    #[error(transparent)]
    Database(#[from] DatabaseError),

    /// The database lacks a table that makes one a company's.
    #[error("the database is not a QuickBooks company file: it has no table {table}")]
    NotACompany {
        /// The table's name.
        table: &'static str,
    },

    /// A company file lacks a table that the answer is worked out from, and that
    /// every file read so far has.
    #[error("the company file has no table {table}")]
    NoTable {
        /// The table's name.
        table: &'static str,
    },

    /// A table lacks a column that it has in every release read so far.
    #[error("the table {table} has no column {column}")]
    Schema {
        /// The table's name.
        table: &'static str,
        /// The column's name.
        column: &'static str,
    },

    /// The record that was asked for is damaged.
    #[error("{0}")]
    Damaged(Damage),
}

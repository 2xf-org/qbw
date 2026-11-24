//! Writes small scrambled SQL Anywhere database files, for tests.
//!
//! The writer implements the SQL Anywhere 17 file format directly and shares no
//! code with the reader. A test that reads what it wrote finds a slip in either
//! of the two, and nothing that both have wrong.
//!
//! ```
//! use qbw_synthetic::{
//!     Cell, SyntheticColumn, SyntheticDatabase, SyntheticRow, SyntheticTable,
//! };
//!
//! let plants = SyntheticTable::new("plants")
//!     .column(SyntheticColumn::new("id", 2))
//!     .column(SyntheticColumn::new("name", 9).width(40).nullable())
//!     .primary_key(&["id"])
//!     .row(SyntheticRow::new([Cell::integer(1), Cell::text("Fern")]));
//! let file: Vec<u8> = SyntheticDatabase::new().table(plants).build();
//! assert_eq!(file.len() % 4096, 0);
//! ```
//!
//! [`company`] holds companies shaped like the files that qbw is for.

mod calendar;
mod cell;
pub mod company;
mod keys;
mod pages;
mod scramble;
mod synthetic_database;
mod synthetic_table;

pub use cell::{Cell, LongStorage};
pub use scramble::Sign;
pub use synthetic_database::SyntheticDatabase;
pub use synthetic_table::{IndexOrder, SyntheticColumn, SyntheticRow, SyntheticTable};

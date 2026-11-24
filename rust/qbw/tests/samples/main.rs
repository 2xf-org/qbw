//! Reads real company files, which the repository never holds.
//!
//! Point `QBW_SAMPLES` at a folder of them and run the ignored tests:
//!
//! ```console
//! QBW_SAMPLES=~/Books cargo test --release -p qbw -- --ignored
//! ```
//!
//! A test prints counts and shares, never a name or a value.

// Every function here runs inside a test, where a failed unwrap is a failed test.
#![allow(clippy::unwrap_used)]

mod agreement;
mod books;
mod chain;
mod check;
mod inventory;
mod items;
mod ledger;
mod links;
mod lists;
mod names;
mod open_items;
mod order;
mod queries;
mod references;
mod registers;
mod share;
mod small_lists;
mod statements;
mod support;
mod transactions;

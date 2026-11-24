//! Registers: what is posted to an account, with its running balance.
//!
//! A register has a row for each posting to the account, in the order of the
//! dates and within a day of the lines. QuickBooks' own order within a day is
//! not known, so a running balance is certain at the end of each day and not
//! between.
//!
//! What the column for the other account shows is what the file keeps for it,
//! and nothing is worked out: one account, or a split where the own side of a
//! transaction has several lines against it.

mod lines;
mod read;
mod register;
mod row;
mod texts;

pub use register::Register;
pub use row::RegisterRow;
pub use texts::{PostingText, PostingTexts};

pub(crate) use read::read;
pub(crate) use texts::texts;

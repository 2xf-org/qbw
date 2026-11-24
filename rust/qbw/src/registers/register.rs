//! A register.

use crate::ids::AccountId;
use crate::ledger::Balance;
use crate::left_out::LeftOut;
use crate::period::Period;
use crate::registers::RegisterRow;

/// What is posted to an account in a period, or one page of it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Register {
    /// The account.
    pub account: AccountId,
    /// The period.
    pub period: Period,
    /// The balance before the first row of the period, as the general ledger has
    /// it. An account of profit and loss starts again with each fiscal year, so
    /// it opens with what was posted since the year of the first day started.
    /// Within the period no year is closed. Retained Earnings opens with the
    /// profit of the years before, too.
    pub opening: Balance,
    /// The rows of the page.
    pub rows: Vec<RegisterRow>,
    /// The balance after the last row of the period.
    pub closing: Balance,
    /// How many rows the period has, on every page.
    pub row_count: usize,
    /// How much could not be read.
    pub left_out: LeftOut,
}

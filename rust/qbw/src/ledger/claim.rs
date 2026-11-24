//! What the ledger keeps of a line that somebody owes.

use chrono::NaiveDate;

use crate::codes::TransactionKind;
use crate::rest;
use crate::rest::Rest;
use crate::tables::postings::PostingLineRow;

/// What the file keeps of a line on an account for what customers owe or what
/// vendors are owed, besides what it posts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Claim {
    pub(crate) due_date: Option<NaiveDate>,
    pub(crate) rest: Rest,
    /// What one of the line's currency is in the company's.
    pub(crate) rate: f32,
}

impl Claim {
    /// A claim that is paid, in the company's currency and never due. The
    /// ledger keeps those that are not so.
    pub(super) const SETTLED: Self = Self {
        due_date: None,
        rest: Rest::Nothing,
        rate: 1.0,
    };

    /// The claim of a line of a transaction of `kind`.
    pub(super) fn of(line: &PostingLineRow, kind: TransactionKind) -> Self {
        Self {
            due_date: rest::due_date(kind, line.due_date),
            rest: Rest::of(line.is_paid, line.open_balance),
            rate: line.exchange_rate.unwrap_or(1.0),
        }
    }
}

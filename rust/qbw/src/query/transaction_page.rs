//! A page of transactions.

use crate::codes::TransactionKind;
use crate::left_out::LeftOut;
use crate::query::TransactionSummary;

/// A page of the transactions that a query found.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct TransactionPage {
    /// The rows of the page.
    pub rows: Vec<TransactionSummary>,
    /// How many transactions the query found, on every page.
    pub row_count: usize,
    /// How many of them are of each kind, in the order of the kinds' codes. Kinds
    /// that none is of are left out.
    pub kinds: Vec<KindCount>,
    /// How much could not be read that may have been found, too.
    pub left_out: LeftOut,
}

/// How many of the transactions that a query found are of one kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct KindCount {
    /// The kind.
    pub kind: TransactionKind,
    /// How many are of it.
    pub count: usize,
}

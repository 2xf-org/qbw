//! A row of a register.

use chrono::NaiveDate;

use crate::codes::{Cleared, TransactionKind};
use crate::ids::{LineId, NameId, TransactionId};
use crate::ledger::PostingSource;
use crate::money::Money;
use crate::origin::Origin;
use crate::other_side::OtherSide;

/// A row of a register: one posting to its account.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct RegisterRow {
    /// The date of the transaction.
    pub date: NaiveDate,
    /// The kind of the transaction.
    pub kind: TransactionKind,
    /// The number of the transaction, such as a check's. Nothing for a check
    /// that waits to be printed.
    pub number: Option<String>,
    /// Who the line is with: the line's own name, or else the transaction's.
    pub name: Option<NameId>,
    /// The account on the other side.
    pub other_side: OtherSide,
    /// What the line is about.
    pub memo: Option<String>,
    /// Whether the bank has confirmed the line.
    pub cleared: Cleared,
    /// The date of the statement that the line was reconciled with, which the
    /// file has for some of the reconciled lines.
    pub reconciled_on: Option<NaiveDate>,
    /// The amount in the account's currency. A debit is positive.
    pub amount: Money,
    /// The account's balance after the row, in its currency.
    pub balance: Money,
    /// What the posting comes from.
    pub source: PostingSource,
    /// The transaction.
    pub transaction: TransactionId,
    /// The line of the transaction.
    pub line: LineId,
    /// The stored row.
    pub origin: Origin,
}

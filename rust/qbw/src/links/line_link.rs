//! A link between two lines.

use sa17::Decimal;

use crate::codes::LinkCode;
use crate::ids::{LineId, TransactionId};
use crate::left_out::LeftOut;
use crate::money::Money;

/// The links of a transaction's lines to lines of others.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct LineLinks {
    /// The transaction.
    pub transaction: TransactionId,
    /// The links, by the other transaction.
    pub links: Vec<LineLink>,
    /// How much could not be read.
    pub left_out: LeftOut,
}

/// A link of a line of a transaction to a line of another, as the file keeps it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct LineLink {
    /// The line of the transaction.
    pub line: Option<LineId>,
    /// The other transaction.
    pub other: TransactionId,
    /// The line of the other transaction.
    pub other_line: Option<LineId>,
    /// What ties the two.
    pub code: LinkCode,
    /// Whether the transaction is the one that pays, credits or was made from
    /// the other.
    pub is_source: bool,
    /// The amount, above zero for what is paid, credited or taken off.
    pub amount: Option<Money>,
    /// The quantity received or invoiced.
    pub quantity: Option<Decimal>,
}

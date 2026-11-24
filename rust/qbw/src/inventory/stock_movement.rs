//! A line that changed the stock.

use chrono::NaiveDate;
use sa17::Decimal;

use crate::codes::TransactionKind;
use crate::ids::{LineId, NameId, TransactionId};
use crate::money::Money;
use crate::origin::Origin;

/// A line of a transaction that changed how much there is of an item, or what
/// that is worth.
///
/// Values and costs are in the company's currency.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct StockMovement {
    /// The date of the transaction.
    pub date: NaiveDate,
    /// The transaction.
    pub transaction: TransactionId,
    /// The line.
    pub line: LineId,
    /// The kind of the transaction. `None` when its header cannot be read.
    pub kind: Option<TransactionKind>,
    /// The number of the transaction.
    pub number: Option<String>,
    /// Who the transaction is with.
    pub name: Option<NameId>,
    /// How much the line put into the stock, below zero for what it took out.
    /// Zero where only a value or a cost changed.
    pub quantity: Decimal,
    /// By how much it changed what the stock is worth.
    pub value: Money,
    /// How much there was after it.
    pub on_hand: Decimal,
    /// What that was worth.
    pub value_on_hand: Money,
    /// What one was worth on average.
    pub average_cost: Money,
    /// The row of the history.
    pub origin: Origin,
}

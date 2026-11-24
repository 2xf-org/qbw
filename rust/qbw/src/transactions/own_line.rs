//! The own side of a transaction.

use chrono::NaiveDate;

use crate::codes::{Cleared, TransactionKind};
use crate::ids::{AccountId, ClassId, LineId};
use crate::money::Money;
use crate::origin::Origin;
use crate::other_side::OtherSide;
use crate::transactions::OrderProgress;
use crate::transactions::own_side::OwnSide;
use crate::transactions::stored_line::StoredLine;

/// The own side of a transaction: the line of the bank account, the card, the
/// receivable or the payable that the form is of.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct OwnLine {
    /// What the line is known by.
    pub id: LineId,
    /// The account. `None` in a transfer between sites, and in some journal
    /// entries.
    pub account: Option<AccountId>,
    /// The amount in the transaction's currency. A debit is positive.
    pub amount: Option<Money>,
    /// The amount in the company's currency. A debit is positive.
    pub home_amount: Option<Money>,
    /// The class. Unchecked.
    pub class: Option<ClassId>,
    /// Whether the line was found on the statement of its account.
    pub cleared: Cleared,
    /// The date of that statement, on some of the lines that were.
    pub reconciled_on: Option<NaiveDate>,
    /// The account of the other lines, when it is one.
    pub other_side: OtherSide,
    /// How far the whole order is filled, on an order.
    pub order: Option<OrderProgress>,
    /// The stored row.
    pub origin: Origin,
}

impl OwnLine {
    /// The own line `stored` of a transaction of `kind`, which is `side` to it.
    pub(super) fn new(stored: &StoredLine, kind: TransactionKind, side: OwnSide<'_>) -> Self {
        let row = &stored.row;
        Self {
            id: row.id,
            account: row.account,
            amount: side.amount(),
            home_amount: side.home_amount(),
            class: row.class,
            cleared: row.cleared.unwrap_or_default(),
            reconciled_on: row.reconciled_on,
            other_side: OtherSide::of_own_line(row.is_split, row.other_account),
            order: OrderProgress::of_whole(&row.into(), kind),
            origin: stored.origin,
        }
    }
}

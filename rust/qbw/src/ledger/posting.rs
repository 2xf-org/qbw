//! An amount on an account.

use chrono::NaiveDate;

use crate::codes::{Cleared, TransactionKind};
use crate::ids::{AccountId, ItemId, LineId, NameId, TransactionId};
use crate::ledger::PostingSource;
use crate::money::Money;
use crate::origin::Origin;
use crate::other_side::OtherSide;

/// An amount on an account, by a line of a transaction. A debit is positive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Posting {
    /// The account.
    pub account: AccountId,
    /// The date of the transaction.
    pub date: NaiveDate,
    /// The transaction.
    pub transaction: TransactionId,
    /// The kind of the transaction.
    pub kind: TransactionKind,
    /// The line of the transaction. For what a line of an item in stock posts
    /// besides, that line.
    pub line: LineId,
    /// What the posting comes from.
    pub source: PostingSource,
    /// The amount in the account's currency.
    pub amount: Money,
    /// The amount in the company's currency.
    pub home_amount: Money,
    /// Who the line is with: the line's own name, or else the transaction's.
    pub name: Option<NameId>,
    /// The customer or job.
    pub customer: Option<NameId>,
    /// The vendor. For sales tax, the agency it is owed to.
    pub vendor: Option<NameId>,
    /// The item.
    pub item: Option<ItemId>,
    /// The account on the other side.
    pub other_side: OtherSide,
    /// Whether the bank has confirmed it.
    pub cleared: Cleared,
    /// The stored row.
    pub origin: Origin,
}

impl Posting {
    /// The amount in the company's currency, when it is a debit.
    #[must_use]
    pub fn debit(&self) -> Option<Money> {
        (!self.home_amount.is_negative()).then_some(self.home_amount)
    }

    /// The amount in the company's currency without its sign, when it is a credit.
    #[must_use]
    pub fn credit(&self) -> Option<Money> {
        let credit = self.home_amount.abs();
        self.home_amount.is_negative().then_some(credit)
    }
}

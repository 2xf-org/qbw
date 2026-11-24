//! A posting as the ledger keeps it.

use std::ops::Range;

use chrono::NaiveDate;
use sa17::RowId;

use crate::amount::Amount;
use crate::codes::{Cleared, TransactionKind};
use crate::currencies::Currencies;
use crate::ids::{AccountId, CurrencyId, ItemId, LineId, NameId, TransactionId};
use crate::ledger::{Posting, PostingSource};
use crate::origin::Origin;
use crate::other_side::OtherSide;
use crate::period::Period;
use crate::side::Side;

/// The place of a line that its transaction's chain does not reach.
pub(super) const UNREACHED: u16 = u16::MAX;

/// A posting, packed, since a ledger has hundreds of thousands of them. An id
/// is the number that the file stores, with 0 where there is none.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Entry {
    /// The amount in the company's currency.
    pub(crate) home: Amount,
    /// The amount in the other currency of the line, where it has one.
    foreign: Amount,
    pub(crate) date: NaiveDate,
    pub(crate) account: AccountId,
    pub(crate) transaction: TransactionId,
    pub(crate) line: LineId,
    pub(crate) row: RowId,
    name: i32,
    customer: i32,
    vendor: i32,
    item: i32,
    other_side: i32,
    kind: i16,
    cleared: i16,
    /// The line's place in its transaction.
    pub(super) place: u16,
    /// Which of the ledger's tables the row is in.
    pub(super) table: u8,
    pub(crate) source: PostingSource,
}

/// What an entry has besides its amounts, unpacked.
pub(super) struct Details {
    pub(super) date: NaiveDate,
    pub(super) account: AccountId,
    pub(super) transaction: TransactionId,
    pub(super) line: LineId,
    pub(super) row: RowId,
    pub(super) table: u8,
    pub(super) source: PostingSource,
    pub(super) kind: TransactionKind,
    pub(super) name: Option<NameId>,
    pub(super) customer: Option<NameId>,
    pub(super) vendor: Option<NameId>,
    pub(super) item: Option<ItemId>,
    pub(super) other_side: OtherSide,
    pub(super) cleared: Cleared,
}

impl Entry {
    pub(super) fn new(home: Amount, foreign: Amount, details: &Details) -> Self {
        let name = |name: Option<NameId>| name.map_or(0, NameId::number);
        Self {
            home,
            foreign,
            date: details.date,
            account: details.account,
            transaction: details.transaction,
            line: details.line,
            row: details.row,
            name: name(details.name),
            customer: name(details.customer),
            vendor: name(details.vendor),
            item: details.item.map_or(0, ItemId::number),
            other_side: details.other_side.packed(),
            kind: packed(details.kind.code()),
            cleared: packed(details.cleared.code()),
            place: UNREACHED,
            table: details.table,
            source: details.source,
        }
    }

    /// The places of the entries of `period` among the entries of an account,
    /// which are ordered by date.
    pub(crate) fn within(entries: &[Self], period: Period) -> Range<usize> {
        let before = |first| entries.partition_point(|entry| entry.date < first);
        let up_to = |last| entries.partition_point(|entry| entry.date <= last);
        let start = period.first().map_or(0, before);
        let end = period.last().map_or(entries.len(), up_to);
        start..end.max(start)
    }

    /// The amount in `currency`, the one of the account or of the name, of a
    /// company that keeps its books in `home`.
    pub(crate) fn own(&self, currency: CurrencyId, home: CurrencyId) -> Amount {
        Currencies::new(currency, home).own_of(self.home, self.foreign)
    }

    pub(crate) fn kind(&self) -> TransactionKind {
        TransactionKind::from_code(self.kind.into())
    }

    pub(crate) fn name(&self) -> Option<NameId> {
        name(self.name)
    }

    pub(crate) fn customer(&self) -> Option<NameId> {
        name(self.customer)
    }

    pub(crate) fn vendor(&self) -> Option<NameId> {
        name(self.vendor)
    }

    pub(crate) fn item(&self) -> Option<ItemId> {
        (self.item != 0).then(|| ItemId::new(self.item))
    }

    /// The customer of something owed to the company, or the vendor of something
    /// that it owes.
    pub(crate) fn name_on(&self, side: Side) -> Option<NameId> {
        match side {
            Side::Receivable => self.customer(),
            Side::Payable => self.vendor(),
        }
    }

    pub(crate) fn other_side(&self) -> OtherSide {
        OtherSide::unpacked(self.other_side)
    }

    pub(crate) fn cleared(&self) -> Cleared {
        Cleared::from_code(self.cleared.into())
    }

    /// Whether the entry is a line itself, and not what one posts besides.
    pub(crate) fn is_line(&self) -> bool {
        matches!(self.source, PostingSource::OwnLine | PostingSource::Line)
    }

    /// The posting, of an account kept in `own` by a company that keeps its
    /// books in `home`.
    pub(super) fn posting(&self, home: CurrencyId, own: CurrencyId, origin: Origin) -> Posting {
        Posting {
            account: self.account,
            date: self.date,
            transaction: self.transaction,
            kind: self.kind(),
            line: self.line,
            source: self.source,
            home_amount: self.home.money(home),
            amount: self.own(own, home).money(own),
            name: self.name(),
            customer: self.customer(),
            vendor: self.vendor(),
            item: self.item(),
            other_side: self.other_side(),
            cleared: self.cleared(),
            origin,
        }
    }
}

fn name(number: i32) -> Option<NameId> {
    (number != 0).then(|| NameId::new(number))
}

/// A code in the width that the file stores it with.
fn packed(code: i64) -> i16 {
    i16::try_from(code).unwrap_or(i16::MAX)
}

#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;

    #[test]
    fn an_entry_is_small() {
        assert!(mem::size_of::<Entry>() <= 72);
    }
}

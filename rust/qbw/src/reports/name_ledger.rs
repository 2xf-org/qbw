//! What a name owes or is owed, posting by posting.

use crate::as_of::AsOf;
use crate::ids::NameId;
use crate::ledger::{Entry, Ledger, Posting};
use crate::left_out::LeftOut;
use crate::period::Period;
use crate::reports::OwedBalance;
use crate::side::Side;

/// What a customer owes or a vendor is owed, posting by posting. A customer's
/// jobs are not part of it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct NameLedger {
    /// The customer, the job or the vendor.
    pub name: NameId,
    /// Receivable or payable.
    pub side: Side,
    /// The period.
    pub period: Period,
    /// The balance before the period.
    pub opening: OwedBalance,
    /// The postings of the period, by date and by line.
    pub rows: Vec<NameLedgerRow>,
    /// The balance after the period.
    pub closing: OwedBalance,
    /// How much could not be read.
    pub left_out: LeftOut,
}

/// A row of a [`NameLedger`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct NameLedgerRow {
    /// The posting, in whose amounts a debit is positive.
    pub posting: Posting,
    /// What the posting adds to what is owed.
    pub change: OwedBalance,
    /// What is owed after it.
    pub balance: OwedBalance,
}

pub(crate) fn name_ledger(ledger: &Ledger, side: Side, name: NameId, period: Period) -> NameLedger {
    let up_to = period.last().map_or(AsOf::Latest, AsOf::Date);
    let entries = ledger.name_entries(side, name, up_to);
    let is_before = |entry: &&Entry| period.first().is_some_and(|first| entry.date < first);
    let (before, within) = entries.split_at(entries.partition_point(is_before));

    let owed = |sums| OwedBalance::new(sums, side, ledger.home());
    let mut sums = ledger.sums(name);
    for entry in before {
        sums.add(entry);
    }
    let opening = owed(sums);
    let mut rows = Vec::with_capacity(within.len());
    for entry in within {
        let mut amount = ledger.sums(name);
        amount.add(entry);
        sums.add(entry);
        rows.push(NameLedgerRow {
            posting: ledger.posting(entry),
            change: owed(amount),
            balance: owed(sums),
        });
    }
    NameLedger {
        name,
        side,
        period,
        opening,
        rows,
        closing: owed(sums),
        left_out: ledger.left_out(),
    }
}

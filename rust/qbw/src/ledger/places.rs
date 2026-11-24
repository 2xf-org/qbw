//! The places of lines in their transactions.

use std::ops::Range;

use crate::chain::{Chain, Linked};
use crate::ids::{LineId, TransactionId};
use crate::ledger::Entry;
use crate::ledger::entry::UNREACHED;

/// A line, with where the entries are that it posts.
pub(super) struct PlacedLine {
    pub(super) transaction: TransactionId,
    pub(super) id: LineId,
    pub(super) next: Option<LineId>,
    pub(super) is_own: bool,
    pub(super) entries: Range<usize>,
}

impl Linked for &PlacedLine {
    fn id(&self) -> LineId {
        self.id
    }

    fn next(&self) -> Option<LineId> {
        self.next
    }
}

/// Gives every entry of `lines` the place of its line in its transaction, as far
/// as the transaction's chain reaches. Gives back the entries of lines that come
/// more than once, but for the first.
pub(super) fn place(mut lines: Vec<PlacedLine>, entries: &mut [Entry]) -> Vec<Range<usize>> {
    lines.sort_unstable_by_key(|line| (line.transaction, line.id, line.entries.start));
    let mut repeated = Vec::new();
    lines.dedup_by(|line, before| {
        let is_repeated = line.id == before.id && line.transaction == before.transaction;
        if is_repeated {
            repeated.push(line.entries.clone());
        }
        is_repeated
    });
    for lines in lines.chunk_by(|left, right| left.transaction == right.transaction) {
        let Some(own) = lines.iter().find(|line| line.is_own) else {
            continue;
        };
        let chain = Chain::new(lines.iter().collect(), own.id);
        for (place, line) in chain.lines.iter().take(chain.reached).enumerate() {
            let place = u16::try_from(place).unwrap_or(UNREACHED);
            for entry in entries.get_mut(line.entries.clone()).unwrap_or_default() {
                entry.place = place;
            }
        }
    }
    repeated
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use sa17::{RowAddress, RowId};

    use super::*;
    use crate::amount::Amount;
    use crate::codes::{Cleared, TransactionKind};
    use crate::ids::AccountId;
    use crate::ledger::PostingSource;
    use crate::ledger::entry::Details;
    use crate::other_side::OtherSide;

    fn entry(line: i32) -> Entry {
        let details = Details {
            date: NaiveDate::MIN,
            account: AccountId::new(1),
            transaction: TransactionId::new(1),
            line: LineId::new(line),
            row: RowId::Stored(RowAddress::new(1, 1)),
            table: 0,
            source: PostingSource::Line,
            kind: TransactionKind::JournalEntry,
            name: None,
            customer: None,
            vendor: None,
            item: None,
            other_side: OtherSide::None,
            cleared: Cleared::NotCleared,
        };
        Entry::new(Amount::ONE, Amount::ONE, &details)
    }

    /// The places of the lines 1 to `count` of one transaction, each of which
    /// leads to the next and has one entry.
    fn places(count: i32) -> Vec<u16> {
        let mut entries: Vec<Entry> = (1..=count).map(entry).collect();
        let lines = (0..).zip(1..=count).map(|(start, id)| PlacedLine {
            transaction: TransactionId::new(1),
            id: LineId::new(id),
            next: Some(LineId::new(id + 1)).filter(|next| next.number() <= count),
            is_own: id == 1,
            entries: start..start + 1,
        });
        assert!(place(lines.collect(), &mut entries).is_empty());
        entries.iter().map(|entry| entry.place).collect()
    }

    #[test]
    fn a_line_has_its_place_in_the_chain() {
        assert_eq!(places(3), [0, 1, 2]);
    }

    #[test]
    fn lines_past_the_last_place_have_none() {
        let last = i32::from(u16::MAX);
        let places = places(last + 2);
        let end = &places[places.len() - 3..];
        assert_eq!(end, [u16::MAX - 1, UNREACHED, UNREACHED]);
    }

    #[test]
    fn a_line_that_comes_twice_is_placed_once() {
        let line = |entries| PlacedLine {
            transaction: TransactionId::new(1),
            id: LineId::new(1),
            next: None,
            is_own: true,
            entries,
        };
        let mut entries = vec![entry(1), entry(1)];
        let (first, again) = (0..1, 1..2);
        let repeated = place(vec![line(again.clone()), line(first)], &mut entries);
        assert_eq!(repeated, vec![again]);
        let places: Vec<u16> = entries.iter().map(|entry| entry.place).collect();
        assert_eq!(places, [0, UNREACHED]);
    }
}

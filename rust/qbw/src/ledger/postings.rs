//! Walking over postings.

use std::iter::FusedIterator;
use std::slice;

use crate::ledger::{Entry, Ledger, Posting};

/// Postings of a ledger, by account, by date and by line.
#[derive(Clone, Debug)]
pub struct Postings<'ledger> {
    ledger: &'ledger Ledger,
    entries: slice::Iter<'ledger, Entry>,
}

impl<'ledger> Postings<'ledger> {
    pub(super) fn new(ledger: &'ledger Ledger, entries: &'ledger [Entry]) -> Self {
        Self {
            ledger,
            entries: entries.iter(),
        }
    }
}

impl Iterator for Postings<'_> {
    type Item = Posting;

    fn next(&mut self) -> Option<Posting> {
        self.entries.next().map(|entry| self.ledger.posting(entry))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.entries.size_hint()
    }
}

impl DoubleEndedIterator for Postings<'_> {
    fn next_back(&mut self) -> Option<Posting> {
        let entry = self.entries.next_back();
        entry.map(|entry| self.ledger.posting(entry))
    }
}

impl ExactSizeIterator for Postings<'_> {}

impl FusedIterator for Postings<'_> {}

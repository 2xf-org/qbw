//! The journal.

use chrono::NaiveDate;

use crate::amount::Total;
use crate::codes::TransactionKind;
use crate::ids::TransactionId;
use crate::ledger::{Entry, Ledger, Posting};
use crate::left_out::LeftOut;
use crate::money::Money;
use crate::page::Page;
use crate::period::Period;

/// Every transaction of a period with what it posts.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct Journal<'company> {
    /// The period.
    pub period: Period,
    /// How much could not be read.
    pub left_out: LeftOut,
    ledger: &'company Ledger,
}

/// A transaction of a [`Journal`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct JournalEntry {
    /// The transaction.
    pub transaction: TransactionId,
    /// Its date.
    pub date: NaiveDate,
    /// Its kind.
    pub kind: TransactionKind,
    /// What it posts: its own side first, and then line by line as on its
    /// form.
    pub postings: Vec<Posting>,
    /// What its debits come to, in the company's currency.
    pub debits: Money,
    /// What its credits come to. The same as the debits, unless something could
    /// not be read.
    pub credits: Money,
}

impl<'company> Journal<'company> {
    pub(crate) fn new(ledger: &'company Ledger, period: Period) -> Self {
        Self {
            period,
            left_out: ledger.left_out(),
            ledger,
        }
    }

    /// How many transactions the period has.
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.firsts().count()
    }

    /// A page of the transactions in the order they were entered in. Only those
    /// of the page are put together.
    pub fn page(&self, page: Page) -> impl Iterator<Item = JournalEntry> {
        let firsts = self.firsts().skip(page.start()).take(page.size());
        firsts.map(|first| self.entry(first))
    }

    /// The transactions in the order they were entered in. Each is put together
    /// when the iterator comes to it.
    pub fn entries(&self) -> impl Iterator<Item = JournalEntry> {
        self.page(Page::all())
    }

    /// The first entry of each transaction of the period.
    fn firsts(&self) -> impl Iterator<Item = &'company Entry> {
        let period = self.period;
        let firsts = self.ledger.first_entries();
        firsts.filter(move |entry| period.contains(entry.date))
    }

    /// The transaction that starts with `first`.
    fn entry(&self, first: &Entry) -> JournalEntry {
        let entries = self.ledger.transaction_entries(first.transaction);
        let entries: Vec<&Entry> = entries.collect();
        let amounts = entries.iter().map(|entry| Total::from(entry.home));
        let (credits, debits): (Vec<Total>, Vec<Total>) =
            amounts.partition(|amount| amount.is_negative());
        let home = self.ledger.home();
        JournalEntry {
            transaction: first.transaction,
            date: first.date,
            kind: first.kind(),
            postings: entries
                .iter()
                .map(|entry| self.ledger.posting(entry))
                .collect(),
            debits: debits.into_iter().sum::<Total>().money(home),
            credits: (-credits.into_iter().sum::<Total>()).money(home),
        }
    }
}

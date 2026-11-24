//! The ledger.

use std::collections::HashMap;
use std::fmt;

use chrono::NaiveDate;

use crate::accounts::Account;
use crate::amount::Total;
use crate::as_of::AsOf;
use crate::damage::Damage;
use crate::ids::{AccountId, CurrencyId, ItemId, LineId, NameId, TransactionId};
use crate::ledger::account_postings::AccountPostings;
use crate::ledger::sums::Sums;
use crate::ledger::{Balance, Claim, Entry, Posting, Postings};
use crate::left_out::LeftOut;
use crate::list::List;
use crate::origin::Origin;
use crate::period::Period;

/// Every posting of a company's books.
pub struct Ledger {
    home: CurrencyId,
    /// By account, by date and by line.
    entries: Box<[Entry]>,
    /// By account.
    pub(super) accounts: Vec<AccountPostings>,
    /// What is kept of lines that somebody owes.
    claims: HashMap<LineId, Claim>,
    /// The currencies of the names that trade in another one than the company's.
    pub(super) name_currencies: HashMap<NameId, CurrencyId>,
    /// The entries by transaction and by the line's place in it.
    by_transaction: Box<[u32]>,
    /// Where each transaction starts among those.
    transactions: Box<[u32]>,
    /// The tables that the entries' rows are in.
    tables: Vec<&'static str>,
    damage: Vec<Damage>,
    left_out: LeftOut,
}

/// What a ledger is made of.
pub(super) struct Parts<'company> {
    pub(super) home: CurrencyId,
    pub(super) accounts: &'company List<Account>,
    pub(super) entries: Vec<Entry>,
    pub(super) claims: HashMap<LineId, Claim>,
    pub(super) name_currencies: HashMap<NameId, CurrencyId>,
    pub(super) tables: Vec<&'static str>,
    pub(super) damage: Vec<Damage>,
    pub(super) unread_tables: u32,
}

impl Ledger {
    pub(super) fn new(parts: Parts<'_>) -> Self {
        let mut entries = parts.entries.into_boxed_slice();
        entries.sort_unstable_by_key(|entry| (entry.account, entry.date, entry.line, entry.source));
        let by_transaction = by_transaction(&entries);
        Self {
            home: parts.home,
            accounts: AccountPostings::all(parts.accounts, &entries, parts.home),
            transactions: starts(&by_transaction, &entries),
            by_transaction,
            entries,
            claims: parts.claims,
            name_currencies: parts.name_currencies,
            tables: parts.tables,
            // What cannot be read of the accounts decides whose balances a
            // posting is in, and in which currency.
            left_out: LeftOut::of(&parts.damage)
                .with(parts.accounts.left_out())
                .with(LeftOut::of_unread_tables(parts.unread_tables)),
            damage: parts.damage,
        }
    }

    /// Every posting.
    #[must_use]
    pub fn postings(&self) -> Postings<'_> {
        Postings::new(self, &self.entries)
    }

    /// The postings to an account, without those to the accounts below it.
    #[must_use]
    pub fn of_account(&self, account: AccountId) -> Postings<'_> {
        Postings::new(self, self.entries(account))
    }

    /// The postings to an account in a period.
    #[must_use]
    pub fn during(&self, account: AccountId, period: impl Into<Period>) -> Postings<'_> {
        Postings::new(self, self.entries_during(account, period.into()))
    }

    /// The postings of a transaction in the order of its lines, its own side
    /// first. None for a transaction that does not post.
    pub fn of_transaction(&self, transaction: TransactionId) -> impl Iterator<Item = Posting> {
        let entries = self.transaction_entries(transaction);
        entries.map(|entry| self.posting(entry))
    }

    /// The balance of an account, without the accounts below it: the sum of
    /// everything posted to it up to then, for an account of profit and loss,
    /// too. The one of [`AsOf::Latest`] is what QuickBooks keeps as
    /// [`StoredBalance`](crate::accounts::StoredBalance). `None` when there is no
    /// such account and nothing is posted to one.
    #[must_use]
    pub fn balance(&self, account: AccountId, as_of: impl Into<AsOf>) -> Option<Balance> {
        self.account(account)?;
        let period = as_of.into().period();
        Some(self.sum(account, self.entries_during(account, period)))
    }

    /// How much could not be read.
    #[must_use]
    pub fn left_out(&self) -> LeftOut {
        self.left_out
    }

    /// What could not be read.
    #[must_use]
    pub fn damage(&self) -> &[Damage] {
        &self.damage
    }

    /// The currency the books are kept in.
    pub(crate) fn home(&self) -> CurrencyId {
        self.home
    }

    /// The currency an account is kept in.
    pub(crate) fn currency(&self, account: AccountId) -> CurrencyId {
        let account = self.account(account);
        account.map_or(self.home, |account| account.currency)
    }

    /// Whether an account is listed or posted to.
    pub(crate) fn knows(&self, account: AccountId) -> bool {
        self.account(account).is_some()
    }

    /// The accounts that something is posted to, by id.
    pub(crate) fn posted_accounts(&self) -> impl Iterator<Item = AccountId> {
        let posted = self
            .accounts
            .iter()
            .filter(|account| !account.entries.is_empty());
        posted.map(|account| account.account)
    }

    /// The entries of an account.
    pub(crate) fn entries(&self, account: AccountId) -> &[Entry] {
        let account = self.account(account);
        let entries = account.and_then(|account| self.entries.get(account.entries.clone()));
        entries.unwrap_or_default()
    }

    /// The entries of an account in a period.
    pub(crate) fn entries_during(&self, account: AccountId, period: Period) -> &[Entry] {
        let entries = self.entries(account);
        let dated = Entry::within(entries, period);
        entries.get(dated).unwrap_or_default()
    }

    /// The entries of a transaction in the order of its lines.
    pub(crate) fn transaction_entries(
        &self,
        transaction: TransactionId,
    ) -> impl Iterator<Item = &Entry> {
        let of = |index: &u32| self.entry(*index).map(|entry| entry.transaction);
        let order = &self.by_transaction;
        let start = order.partition_point(|index| of(index) < Some(transaction));
        let entries = order
            .iter()
            .skip(start)
            .filter_map(|index| self.entry(*index));
        entries.take_while(move |entry| entry.transaction == transaction)
    }

    /// The entries of the lines with an item, by account and by date, without what
    /// they post besides.
    pub(crate) fn item_entries(&self, item: ItemId) -> impl Iterator<Item = &Entry> {
        let entries = self.entries.iter();
        entries.filter(move |entry| entry.is_line() && entry.item() == Some(item))
    }

    /// Every transaction that posts, by id, with the date that it posts on.
    pub(crate) fn dates(&self) -> impl Iterator<Item = (TransactionId, NaiveDate)> {
        let entries = self.first_entries();
        entries.map(|entry| (entry.transaction, entry.date))
    }

    /// The last day that something is posted on.
    pub(crate) fn last_day(&self) -> Option<NaiveDate> {
        let accounts = self.accounts.iter();
        let last = accounts.filter_map(|account| self.entries(account.account).last());
        last.map(|entry| entry.date).max()
    }

    /// Every entry, by transaction and by the line's place in it.
    pub(crate) fn entries_by_transaction(&self) -> impl Iterator<Item = &Entry> {
        let order = self.by_transaction.iter();
        order.filter_map(|index| self.entry(*index))
    }

    /// The first entry of every transaction, in the order they were entered in.
    pub(crate) fn first_entries(&self) -> impl Iterator<Item = &Entry> {
        let starts = self.transactions.iter();
        let first = starts.filter_map(|&start| {
            let start = usize::try_from(start).ok()?;
            self.by_transaction.get(start)
        });
        first.filter_map(|&index| self.entry(index))
    }

    /// The sum of an account's postings in a period, in the company's currency.
    pub(crate) fn total(&self, account: AccountId, period: Period) -> Total {
        let entries = self.entries_during(account, period).iter();
        entries.map(|entry| entry.home).sum()
    }

    /// What `entries` of an account add up to.
    pub(crate) fn sum(&self, account: AccountId, entries: &[Entry]) -> Balance {
        let mut sums = Sums::new(self.home, self.currency(account));
        sums.add_all(entries);
        sums.balance(self.left_out)
    }

    /// What is kept of a line that somebody owes.
    pub(crate) fn claim(&self, line: LineId) -> Claim {
        self.claims.get(&line).copied().unwrap_or(Claim::SETTLED)
    }

    /// The posting of an entry.
    pub(crate) fn posting(&self, entry: &Entry) -> Posting {
        entry.posting(self.home, self.currency(entry.account), self.origin(entry))
    }

    /// The row that an entry came from.
    pub(crate) fn origin(&self, entry: &Entry) -> Origin {
        Origin::new(self.table(entry), entry.row)
    }

    /// The table that the entry's row is in.
    pub(crate) fn table(&self, entry: &Entry) -> &'static str {
        let table = self.tables.get(usize::from(entry.table));
        table.copied().unwrap_or_default()
    }

    fn entry(&self, index: u32) -> Option<&Entry> {
        self.entries.get(usize::try_from(index).ok()?)
    }

    fn account(&self, account: AccountId) -> Option<&AccountPostings> {
        let found = self
            .accounts
            .binary_search_by_key(&account, |known| known.account);
        self.accounts.get(found.ok()?)
    }
}

impl fmt::Debug for Ledger {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Ledger")
            .field("postings", &self.entries.len())
            .field("left_out", &self.left_out)
            .finish_non_exhaustive()
    }
}

/// The places in `by_transaction`, an order of `entries`, where a transaction
/// starts.
fn starts(by_transaction: &[u32], entries: &[Entry]) -> Box<[u32]> {
    let of = |index: u32| {
        let entry = entries.get(usize::try_from(index).ok()?);
        entry.map(|entry| entry.transaction)
    };
    let mut before = None;
    let places = (0..).zip(by_transaction).filter_map(|(place, &index)| {
        let transaction = of(index);
        let is_start = transaction != before;
        before = transaction;
        is_start.then_some(place)
    });
    places.collect()
}

/// The places of `entries` in the order of their transactions and lines.
fn by_transaction(entries: &[Entry]) -> Box<[u32]> {
    let keyed = (0..).zip(entries).map(|(place, entry): (u32, &Entry)| {
        let line = (entry.place, entry.line, entry.source);
        (entry.transaction, line, place)
    });
    let mut order: Vec<_> = keyed.collect();
    order.sort_unstable();
    order.into_iter().map(|(.., place)| place).collect()
}

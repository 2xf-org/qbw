//! What a transaction has on one account with one name.

use chrono::NaiveDate;

use crate::accounts::Account;
use crate::amount::Total;
use crate::ids::NameId;
use crate::ledger::{Entry, Ledger};
use crate::links::{LinkDates, LinkIndex, StoredLink};
use crate::open_items::OpenItem;
use crate::open_items::open_item::OpenSums;
use crate::side::Side;
use crate::transactions::{Header, ShownHeaders};

/// What a transaction has on one account with one name, as a debit or as a
/// credit. A transaction has one part, but for a journal entry of several names,
/// or one that debits and credits the same.
pub(super) struct Part<'company> {
    first: &'company Entry,
    entries: Vec<&'company Entry>,
    account: &'company Account,
    side: Side,
    /// In the account's currency. A debit is positive, here and below.
    own: Total,
    home: Total,
}

/// What settled the parts of transactions up to the end of a day.
#[derive(Clone, Copy)]
pub(crate) struct Settling<'company> {
    pub(crate) day: NaiveDate,
    pub(crate) links: &'company LinkIndex,
    pub(crate) dates: &'company LinkDates,
}

/// The amounts of a part are too large.
pub(super) struct TooLarge;

impl<'company> Part<'company> {
    /// The parts of the transaction with `lines`, each given with its account.
    /// They are ordered by account, by name and by whether they are below zero.
    pub(super) fn of_transaction(
        lines: &[(&'company Entry, &'company Account)],
        side: Side,
        ledger: &Ledger,
    ) -> Vec<Self> {
        let parts = lines.chunk_by(|left, right| {
            let [left, right] = [left, right].map(|(entry, account)| {
                let own = entry.own(account.currency, ledger.home());
                (entry.account, entry.name_on(side), own.is_negative())
            });
            left == right
        });
        let parts = parts.filter_map(|lines| {
            let &(first, account) = lines.first()?;
            let entries: Vec<&Entry> = lines.iter().map(|(entry, _)| *entry).collect();
            let own = |entry: &&Entry| entry.own(account.currency, ledger.home());
            Some(Self {
                first,
                account,
                side,
                own: entries.iter().map(own).sum(),
                home: entries.iter().map(|entry| entry.home).sum(),
                entries,
            })
        });
        parts.collect()
    }

    /// The customer or the vendor.
    pub(super) fn name(&self) -> Option<NameId> {
        self.first.name_on(self.side)
    }

    /// What is left of the part, as the file keeps it.
    pub(super) fn rest(&self, ledger: &Ledger) -> Total {
        let rests = self.entries.iter().map(|entry| {
            let whole = entry.own(self.account.currency, ledger.home());
            ledger.claim(entry.line).rest.of_line(whole)
        });
        rests.sum()
    }

    /// What was left of the part at the end of a day, by the links. It is the
    /// part at `place` of `parts`, the parts of one transaction.
    ///
    /// The date of what settled it is the one it posts on, or else that of its
    /// header. Where neither can be read, it counts on the last day there is.
    pub(super) fn unsettled(
        &self,
        place: usize,
        parts: &[Self],
        settling: Settling<'_>,
        headers: &mut ShownHeaders<'_>,
    ) -> Total {
        let of_day = settling
            .links
            .of(self.first.transaction)
            .iter()
            .filter(|link| self.side.is_settled_by(link.code()))
            .filter(|link| {
                let date = settling.dates.get(link.other).or_else(|| {
                    let header = headers.of_any_kind(link.other);
                    header.map(|header| header.date)
                });
                date.unwrap_or(NaiveDate::MAX) <= settling.day
            });
        // A link to no line of a part belongs to the first.
        let settled = of_day.filter(|link| {
            let of_line = parts.iter().position(|part| part.has_line(link));
            of_line.unwrap_or(0) == place
        });
        let settled: Total = settled.map(StoredLink::amount).sum();
        if self.own.is_negative() {
            self.own + settled
        } else {
            self.own - settled
        }
    }

    fn has_line(&self, link: &StoredLink) -> bool {
        self.entries
            .iter()
            .any(|entry| Some(entry.line) == link.line())
    }

    /// The item of the part, of which `rest` is left. `None` when nothing is.
    ///
    /// # Errors
    ///
    /// Returns an error for amounts that are too large.
    pub(super) fn item(
        &self,
        rest: Total,
        ledger: &Ledger,
        headers: &mut ShownHeaders<'_>,
    ) -> Result<Option<OpenItem>, TooLarge> {
        if rest.is_zero() {
            return Ok(None);
        }
        let home_rest = self.home_rest(rest, ledger)?;
        let first = self.first;
        let header = headers.of_kind(first.transaction, first.kind());
        let currency = self.account.currency;
        let sums = OpenSums {
            open: self.side.owed(rest),
            home_open: self.side.owed(home_rest),
        };
        let mut due_dates = self
            .entries
            .iter()
            .map(|entry| ledger.claim(entry.line).due_date);
        Ok(Some(OpenItem {
            transaction: first.transaction,
            kind: first.kind(),
            date: first.date,
            number: header.and_then(Header::number),
            name: self.name(),
            account: self.account.id,
            due_date: due_dates.find_map(|due_date| due_date),
            total: self.side.owed(self.own).money(currency),
            open: sums.open.money(currency),
            home_open: sums.home_open.money(ledger.home()),
            origin: ledger.origin(first),
            sums,
        }))
    }

    /// What is left in the company's currency: the line's own amount, less what
    /// settled a part of it at the rate of the line. A line whose own amount is
    /// not at that rate has what is left at it.
    fn home_rest(&self, rest: Total, ledger: &Ledger) -> Result<Total, TooLarge> {
        if self.account.currency == ledger.home() {
            return Ok(rest);
        }
        let settled = self.own - rest;
        if settled.is_zero() {
            return Ok(self.home);
        }
        let claim = ledger.claim(self.first.line);
        let at_rate = |own: Total| {
            let home = own.amount().and_then(|own| own.at_rate(claim.rate));
            home.map(Total::from).ok_or(TooLarge)
        };
        if at_rate(self.own)? == self.home {
            Ok(self.home - at_rate(settled)?)
        } else {
            at_rate(rest)
        }
    }
}

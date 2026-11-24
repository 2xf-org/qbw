//! Reading a register.

use sa17::Source;

use crate::amount::{Amount, Total};
use crate::error::Result;
use crate::ids::{AccountId, CurrencyId};
use crate::ledger::{Entry, Ledger, Sums};
use crate::left_out::LeftOut;
use crate::page::Page;
use crate::period::Period;
use crate::registers::lines::Lines;
use crate::registers::{Register, RegisterRow};
use crate::reports::Books;
use crate::transactions::{Header, Headers, ShownHeaders};

/// Reads a page of the register of an account in a period. Only the rows of the
/// page are read from `source`; the rest is sums over the ledger.
///
/// # Errors
///
/// Returns an error when the table of a row lacks a column.
pub(crate) fn read(
    source: &dyn Source,
    books: Books<'_>,
    headers: &Headers,
    account: AccountId,
    (period, page): (Period, Page),
) -> Result<Register> {
    let ledger = books.ledger;
    let currency = ledger.currency(account);
    let entries = ledger.entries_during(account, period);
    let page = page.of(entries.len());
    let skipped = entries.get(..page.start).unwrap_or_default();
    let shown = entries.get(page).unwrap_or_default();

    let before = books.before(account, period);
    let before = before.map(|before| ledger.entries_during(account, before));
    let brought_forward = books.brought_forward(account, period);
    let mut sums = Sums::new(ledger.home(), currency).with(brought_forward);
    sums.add_all(before.unwrap_or_default());
    let opening = sums;
    sums.add_all(entries);

    let mut running = Running {
        ledger,
        headers: ShownHeaders::new(headers),
        currency,
        lines: Lines::new(source, ledger),
        balance: opening.own,
    };
    running.skip(skipped);
    let rows = shown.iter().map(|entry| running.row(entry));
    let rows: Result<Vec<RegisterRow>> = rows.collect();
    Ok(Register {
        account,
        period,
        opening: opening.balance(ledger.left_out()),
        rows: rows?,
        closing: sums.balance(ledger.left_out()),
        row_count: entries.len(),
        left_out: ledger.left_out().with(running.left_out()),
    })
}

/// The rows of a register, one after the other.
struct Running<'company> {
    ledger: &'company Ledger,
    headers: ShownHeaders<'company>,
    currency: CurrencyId,
    lines: Lines<'company>,
    /// The balance after the last row.
    balance: Total,
}

impl Running<'_> {
    /// Goes on behind the rows of `entries`.
    fn skip(&mut self, entries: &[Entry]) {
        self.balance += entries.iter().map(|entry| self.amount(entry)).sum();
    }

    fn row(&mut self, entry: &Entry) -> Result<RegisterRow> {
        let origin = self.ledger.origin(entry);
        let line = self.lines.read(entry.transaction, entry.line, origin)?;
        let header = self.headers.of_kind(entry.transaction, entry.kind());
        let amount = self.amount(entry);
        self.balance += amount.into();
        Ok(RegisterRow {
            date: entry.date,
            kind: entry.kind(),
            number: header.and_then(Header::number),
            name: entry.name(),
            other_side: entry.other_side(),
            reconciled_on: line.as_ref().and_then(|line| line.reconciled_on),
            memo: line.and_then(|line| line.memo),
            cleared: entry.cleared(),
            amount: amount.money(self.currency),
            balance: self.balance.money(self.currency),
            source: entry.source,
            transaction: entry.transaction,
            line: entry.line,
            origin,
        })
    }

    /// How much could not be read of the rows so far, besides what the ledger
    /// lacks.
    fn left_out(&self) -> LeftOut {
        self.lines.left_out().with(self.headers.left_out())
    }

    /// What an entry adds to the balance.
    fn amount(&self, entry: &Entry) -> Amount {
        entry.own(self.currency, self.ledger.home())
    }
}

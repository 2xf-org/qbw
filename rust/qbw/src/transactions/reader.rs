//! Reading a transaction.

use std::collections::HashSet;

use sa17::{RowId, Source};

use crate::accounts::Account;
use crate::chain::Chain;
use crate::damage::{Damage, Problem};
use crate::error::{Error, Result};
use crate::ids::{CurrencyId, LineId};
use crate::items::Item;
use crate::list::{List, ListEntry};
use crate::lists::{SalesRep, Terms};
use crate::origin::Origin;
use crate::side::Side;
use crate::tables::headers::{AdjustmentRow, HeaderRow, ReceiptRow};
use crate::tables::lines::{LineRow, SummaryLineRow};
use crate::tables::{Form, OpenedTables, StoredRow};
use crate::transactions::family::Family;
use crate::transactions::forms::{
    Bill, BillPayment, Build, CheckForm, InventoryAdjustment, JournalForm, Payment, PurchaseOrder,
    Refund, Sale, Transfer,
};
use crate::transactions::left::Left;
use crate::transactions::own_side::OwnSide;
use crate::transactions::stored_line::StoredLine;
use crate::transactions::{Details, Header, Line, LineRole, OpenState, OwnLine, Transaction};

/// What reading transactions takes besides their rows.
pub(crate) struct Reader<'company> {
    pub(crate) source: &'company dyn Source,
    pub(crate) tables: &'company OpenedTables,
    /// The currency the books are kept in.
    pub(crate) home: CurrencyId,
    pub(crate) accounts: &'company List<Account>,
    pub(crate) items: &'company List<Item>,
    pub(crate) terms: &'company List<Terms>,
    pub(crate) sales_reps: &'company List<SalesRep>,
}

impl Reader<'_> {
    /// The transaction of `header`, whose lines are at `rows`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Damaged`] when the header cannot be read.
    pub(crate) fn read(
        &self,
        header: &Header,
        rows: impl Iterator<Item = RowId>,
    ) -> Result<Transaction> {
        let row: HeaderRow = self.header_row(header)?;
        let currency = row.currency.unwrap_or(self.home);
        let mut damage = Vec::new();
        let lines = self.stored_lines(header, rows, &mut damage)?;
        damage.extend(self.doubts(&lines, currency));

        let own = lines.iter().find(|line| line.row.is_own);
        let own_row = own.map(|own| SummaryLineRow::from(&own.row));
        let side = own_row.as_ref();
        let side = side.map(|row| OwnSide::new(header, row, self.home));
        let own_line = own.zip(side);
        let own_line = own_line.map(|(own, side)| OwnLine::new(own, header.kind, side));
        let left = lines.iter().filter_map(|line| {
            let account = self.accounts.get(line.row.account?)?;
            let side = Side::of_type(account.account_type)?;
            let row = SummaryLineRow::from(&line.row);
            Some(Left::of_line(&row, side, self.home))
        });
        let left = left.reduce(Left::with);
        let details = self.details(header, &lines)?;
        let terms = entry(self.terms, details.terms(), &mut damage);
        let sales_rep = entry(self.sales_reps, details.sales_rep(), &mut damage);
        Ok(Transaction {
            id: header.id,
            kind: header.kind,
            date: header.date,
            number: header.number(),
            name: header.name(),
            memo: row.memo,
            currency,
            exchange_rate: row.exchange_rate,
            total: side.and_then(OwnSide::total),
            home_total: side.and_then(OwnSide::home_total),
            due_date: side.and_then(OwnSide::due_date),
            open_state: left.map_or(OpenState::NotTracked, |left| left.state(header, currency)),
            own_line,
            posting: header.posting,
            last_action: row.last_action,
            billing: row.billing,
            terms_name: terms.map(|terms| terms.name.clone()),
            sales_rep_initials: sales_rep.map(|rep| rep.initials.clone()),
            lines: self.lines(header, &details, lines),
            details,
            created: row.created,
            modified: row.modified,
            edited_by: row.edited_by,
            sequence: row.sequence,
            owner_key: row.owner_key,
            damage,
            origin: header.origin(),
            party: header.party,
            owed: left.map(Left::owed).unwrap_or_default(),
        })
    }

    /// The line of the transaction of `header` that is stored in `row`. `None`
    /// when there is no such line.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Damaged`] when the line cannot be read.
    pub(crate) fn line(&self, header: &Header, row: RowId) -> Result<Option<Line>> {
        let lines = header.form.lines_among::<LineRow>(self.tables, self.source);
        let Some(table) = lines? else {
            return Ok(None);
        };
        let origin = Origin::new(header.form.lines::<LineRow>().name(), row);
        let stored = table.fetch(row)?.map(|row| StoredLine { row, origin });
        Ok(stored.map(|stored| self.shown(stored, header, None)))
    }

    /// The line `stored` as the form of `header` shows it, in `group`.
    fn shown(&self, stored: StoredLine, header: &Header, group: Option<LineId>) -> Line {
        let item = stored.row.item.and_then(|item| self.items.get(item));
        let item = item.map(|item| item.kind);
        Line::new(stored, header, self.home, item, group)
    }

    /// The row of a header, read as `R`.
    fn header_row<R: StoredRow + 'static>(&self, header: &Header) -> Result<R> {
        let headers = header.form.headers::<R>();
        let table = self.tables.open(&headers, self.source)?;
        let row = table.fetch(header.row)?;
        row.ok_or_else(|| {
            let Origin { table, row } = header.origin();
            Error::Damaged(Damage::new(table, Some(row), Problem::Header))
        })
    }

    /// The lines at `rows` in the order of their form. What is wrong with them
    /// is added to `damage`.
    fn stored_lines(
        &self,
        header: &Header,
        rows: impl Iterator<Item = RowId>,
        damage: &mut Vec<Damage>,
    ) -> Result<Vec<StoredLine>> {
        let lines = header.form.lines::<LineRow>();
        let mut stored = Vec::new();
        let mut seen = HashSet::new();
        let table = header
            .form
            .lines_among::<LineRow>(self.tables, self.source)?;
        if let Some(table) = table {
            for row in rows {
                let origin = Origin::new(lines.name(), row);
                match table.fetch(row) {
                    Ok(Some(row)) if !seen.insert(row.id) => {
                        damage.push(Damage::repeated(origin));
                    }
                    Ok(read) => stored.extend(read.map(|row| StoredLine { row, origin })),
                    Err(Error::Damaged(found)) => damage.push(found),
                    Err(error) => return Err(error),
                }
            }
        }
        let chain = Chain::new(stored, header.own_line);
        if damage.is_empty() {
            damage.extend(lost(header.form, &chain));
        }
        Ok(chain.lines)
    }

    /// What makes it doubtful how to read `lines` of a transaction in
    /// `currency`.
    fn doubts(&self, lines: &[StoredLine], currency: CurrencyId) -> Vec<Damage> {
        let mut doubts = Vec::new();
        let currency_of = |line: &StoredLine| line.row.currency.unwrap_or(self.home);
        let other = lines.iter().find(|line| currency_of(line) != currency);
        if let Some(line) = other {
            let Origin { table, row } = line.origin;
            doubts.push(Damage::new(table, Some(row), Problem::Currencies));
        }
        let mut accounts = lines.iter().filter_map(|line| line.row.account);
        if accounts.any(|account| self.accounts.get(account).is_none()) {
            doubts.extend_from_slice(self.accounts.damage());
        }
        let mut items = lines.iter().filter_map(|line| line.row.item);
        if items.any(|item| self.items.get(item).is_none()) {
            doubts.extend_from_slice(self.items.damage());
        }
        doubts
    }

    /// The lines of `stored`, without the own one unless the form shows it.
    fn lines(&self, header: &Header, details: &Details, stored: Vec<StoredLine>) -> Vec<Line> {
        let mut group = None;
        let mut lines = Vec::with_capacity(stored.len());
        for stored in stored {
            if stored.row.is_own && !header.shows_own_line() {
                continue;
            }
            let mut line = self.shown(stored, header, group);
            if let Details::Journal(journal) = details {
                line = line.of_journal(journal);
            }
            match line.role {
                LineRole::GroupStart => group = Some(line.id),
                LineRole::GroupEnd => group = None,
                _ => {}
            }
            lines.push(line);
        }
        lines
    }

    /// What the transaction of `header` has because of its kind.
    fn details(&self, header: &Header, lines: &[StoredLine]) -> Result<Details> {
        let line = |is_own| {
            let mut lines = lines.iter().map(|line| &line.row);
            lines.find(|row| row.is_own == is_own)
        };
        let (own, other) = (line(true), line(false));
        Ok(match Family::of(header.kind) {
            Family::Sale => Details::Sale(Sale::new(self.header_row(header)?, own)),
            Family::Payment => {
                let row: ReceiptRow = self.header_row(header)?;
                Details::Payment(Payment::new(&row, header.is_without_money))
            }
            Family::Refund => Details::Refund(Refund::new(&self.header_row(header)?)),
            Family::Bill => Details::Bill(Bill::new(&self.header_row(header)?)),
            Family::PurchaseOrder => {
                Details::PurchaseOrder(PurchaseOrder::new(self.header_row(header)?))
            }
            Family::BillPayment => {
                Details::BillPayment(BillPayment::new(&self.header_row(header)?))
            }
            Family::Check => Details::Check(CheckForm::new(&self.header_row(header)?)),
            Family::CardCharge => Details::CardCharge,
            Family::Deposit => Details::Deposit,
            Family::Transfer => Details::Transfer(Transfer::new(own, other)),
            Family::Journal => Details::Journal(JournalForm::new(&self.header_row(header)?, own)),
            Family::InventoryAdjustment => {
                let row: AdjustmentRow = self.header_row(header)?;
                Details::InventoryAdjustment(InventoryAdjustment::new(&row, own))
            }
            Family::InventoryTransfer => Details::InventoryTransfer,
            Family::Build => Details::Build(Build::new(&self.header_row(header)?, own)),
            Family::Other => Details::Other,
        })
    }
}

/// The entry of `list` with `id`. Where it is not there, what could not be read of
/// the list is added to `damage`.
fn entry<'list, T: ListEntry>(
    list: &'list List<T>,
    id: Option<T::Id>,
    damage: &mut Vec<Damage>,
) -> Option<&'list T> {
    let entry = list.get(id?);
    if entry.is_none() {
        damage.extend_from_slice(list.damage());
    }
    entry
}

/// What shows that a line of `chain` is lost, when all that were found can be
/// read.
fn lost(form: Form, chain: &Chain<StoredLine>) -> Option<Damage> {
    let table = form.lines::<LineRow>().name();
    let mut lines = chain.lines.iter();
    if !lines.any(|line| line.row.is_own) {
        return Some(Damage::new(table, None, Problem::OwnLine));
    }
    if chain.is_whole {
        return None;
    }
    let mut reached = chain.lines.iter().take(chain.reached);
    let last = reached.next_back().map(|line| line.origin.row);
    Some(Damage::new(table, last, Problem::LineChain))
}

//! Reading the ledger.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::mem;

use sa17::Source;

use crate::accounts::Account;
use crate::amount::Amount;
use crate::codes::TransactionKind;
use crate::damage::{Damage, Problem};
use crate::error::Result;
use crate::ids::{AccountId, CurrencyId, LineId, NameId};
use crate::ledger::book::Parts;
use crate::ledger::entry::Details;
use crate::ledger::places;
use crate::ledger::places::PlacedLine;
use crate::ledger::{Claim, Entry, Ledger, PostingSource};
use crate::list::List;
use crate::origin::Origin;
use crate::other_side::OtherSide;
use crate::side::Side;
use crate::tables::name_currencies::{CUSTOMER_CURRENCIES, VENDOR_CURRENCIES};
use crate::tables::postings::{InventoryLineRow, PostingLineRow};
use crate::tables::{Form, transactions};

/// Reads the ledger of a company that keeps its books in `home`.
///
/// # Errors
///
/// Returns an error when a table lacks a column that it has in every file.
pub(crate) fn build(
    source: &dyn Source,
    accounts: &List<Account>,
    home: CurrencyId,
) -> Result<Ledger> {
    let claimed = accounts
        .iter()
        .filter(|account| Side::of_type(account.account_type).is_some());
    let mut reader = Reader {
        source,
        home,
        claimed: claimed.map(|account| account.id).collect(),
        entries: Vec::new(),
        repeated: BTreeSet::new(),
        claims: HashMap::new(),
        tables: Vec::new(),
        damage: Vec::new(),
    };
    for form in Form::all() {
        reader.read(form)?;
    }
    let name_currencies = reader.name_currencies()?;
    Ok(Ledger::new(Parts {
        home,
        accounts,
        entries: reader.take_entries(),
        claims: reader.claims,
        name_currencies,
        tables: reader.tables,
        damage: reader.damage,
        unread_tables: transactions::unread(source),
    }))
}

/// Reads one form of transaction after the other.
struct Reader<'source> {
    source: &'source dyn Source,
    home: CurrencyId,
    /// The accounts for what customers owe and what vendors are owed.
    claimed: HashSet<AccountId>,
    entries: Vec<Entry>,
    /// The places of the entries of lines that come more than once.
    repeated: BTreeSet<usize>,
    claims: HashMap<LineId, Claim>,
    tables: Vec<&'static str>,
    damage: Vec<Damage>,
}

/// What lines of items in stock post besides, by line.
type Stock = HashMap<LineId, Vec<(Origin, InventoryLineRow)>>;

impl Reader<'_> {
    fn read(&mut self, form: Form) -> Result<()> {
        let Some(table) = form.lines_in::<PostingLineRow>(self.source)? else {
            return Ok(());
        };
        let mut stock = self.stock(form)?;
        let mut damage = Vec::new();
        let mut lines = Vec::new();
        for (origin, line) in table.scan(&mut damage) {
            let stock = stock.remove(&line.id).unwrap_or_default();
            let kind = form.kind(Some(line.kind));
            let first = self.entries.len();
            if line.flags(kind).is_posted() {
                self.post(origin, &line, kind, &stock);
            }
            lines.push(PlacedLine {
                transaction: line.transaction,
                id: line.id,
                next: line.next,
                is_own: line.is_own,
                entries: first..self.entries.len(),
            });
        }
        let repeated = places::place(lines, &mut self.entries);
        self.repeated.extend(repeated.into_iter().flatten());
        let lost = stock.into_values().flatten();
        damage.extend(
            lost.map(|(origin, _)| Damage::new(origin.table, Some(origin.row), Problem::NoLine)),
        );
        self.damage.append(&mut damage);
        Ok(())
    }

    fn stock(&mut self, form: Form) -> Result<Stock> {
        let mut stock = Stock::new();
        if let Some(table) = form.stock_lines_in(self.source)? {
            for (origin, row) in table.scan(&mut self.damage) {
                stock.entry(row.line).or_default().push((origin, row));
            }
        }
        Ok(stock)
    }

    /// The entries, those of repeated lines left out and counted as damage.
    fn take_entries(&mut self) -> Vec<Entry> {
        let mut entries = mem::take(&mut self.entries);
        if self.repeated.is_empty() {
            return entries;
        }
        let repeated = mem::take(&mut self.repeated);
        for entry in repeated.iter().filter_map(|&place| entries.get(place)) {
            let table = self.tables.get(usize::from(entry.table)).copied();
            let origin = table.map(|table| Origin::new(table, entry.row));
            self.damage.extend(origin.map(Damage::repeated));
        }
        let mut place = 0;
        entries.retain(|_| {
            place += 1;
            !repeated.contains(&(place - 1))
        });
        entries
    }

    /// Adds what a line posts, and what it posts besides.
    fn post(
        &mut self,
        origin: Origin,
        line: &PostingLineRow,
        kind: TransactionKind,
        stock: &[(Origin, InventoryLineRow)],
    ) {
        let source = if line.is_own {
            PostingSource::OwnLine
        } else {
            PostingSource::Line
        };
        let amounts = (line.home_amount, line.foreign_amount);
        self.push(origin, line, kind, source, line.account, amounts);
        if line
            .account
            .is_some_and(|account| self.claimed.contains(&account))
        {
            let claim = Claim::of(line, kind);
            if claim != Claim::SETTLED {
                self.claims.entry(line.id).or_insert(claim);
            }
        }
        for (origin, row) in stock {
            let amounts = (row.home_amount, row.foreign_amount);
            self.push(*origin, line, kind, row.side.into(), row.account, amounts);
        }
    }

    /// Adds an entry of `line`, unless it has no account. The amounts are the
    /// one in the company's currency and the other one.
    fn push(
        &mut self,
        origin: Origin,
        line: &PostingLineRow,
        kind: TransactionKind,
        source: PostingSource,
        account: Option<AccountId>,
        (home, foreign): (Option<Amount>, Option<Amount>),
    ) {
        let Some(account) = account else {
            return;
        };
        let details = Details {
            date: line.date,
            account,
            transaction: line.transaction,
            line: line.id,
            row: origin.row,
            table: self.table(origin.table),
            source,
            kind,
            name: line
                .names()
                .of_posting(line.name_column)
                .map(|party| party.name),
            customer: line.customer,
            vendor: line.vendor,
            item: line.item,
            other_side: other_side(line),
            cleared: line.cleared.unwrap_or_default(),
        };
        let (home, foreign) = (home.unwrap_or_default(), foreign.unwrap_or_default());
        self.entries.push(Entry::new(home, foreign, &details));
    }

    /// The place of a table among [`Self::tables`].
    fn table(&mut self, name: &'static str) -> u8 {
        let known = self.tables.iter().rposition(|&table| table == name);
        let place = known.unwrap_or_else(|| {
            self.tables.push(name);
            self.tables.len() - 1
        });
        // There are two tables for each of the 23 forms at most.
        u8::try_from(place).unwrap_or(u8::MAX)
    }

    /// The currencies of the names that trade in another one than the company's.
    fn name_currencies(&mut self) -> Result<HashMap<NameId, CurrencyId>> {
        let mut currencies = Vec::new();
        let customers = CUSTOMER_CURRENCIES.open(self.source)?;
        let rows = customers.scan(&mut self.damage);
        currencies.extend(rows.map(|(_, row)| (row.id, row.currency)));
        let vendors = VENDOR_CURRENCIES.open(self.source)?;
        let rows = vendors.scan(&mut self.damage);
        currencies.extend(rows.map(|(_, row)| (row.id, row.currency)));
        let currencies = currencies.into_iter();
        let foreign = currencies.filter_map(|(name, currency)| {
            let currency = currency.filter(|&currency| currency != self.home)?;
            Some((name, currency))
        });
        Ok(foreign.collect())
    }
}

/// The account on the other side of a line: on the own line of one of the
/// others, and on the others the own line's.
fn other_side(line: &PostingLineRow) -> OtherSide {
    if line.is_own {
        OtherSide::of_own_line(line.is_split, line.other_account)
    } else {
        line.other_account.into()
    }
}

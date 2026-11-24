//! What every customer owes, or every vendor is owed.

use std::collections::BTreeMap;

use crate::amount::Total;
use crate::as_of::AsOf;
use crate::ids::NameId;
use crate::ledger::{Ledger, Sums};
use crate::left_out::LeftOut;
use crate::money::Money;
use crate::names::Names;
use crate::reports::OwedBalance;
use crate::side::Side;

/// What every customer owes, or every vendor is owed: the sums of their postings
/// to the accounts for it. A vendor's has the sales tax that it is owed.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct NameBalances {
    /// Receivable or payable.
    pub side: Side,
    /// The moment of the balances.
    pub as_of: AsOf,
    /// The names with a balance, or with a job that has one, in the list's
    /// order. Those that the list does not have come last.
    pub rows: Vec<NameBalanceRow>,
    /// What the rows come to, in the company's currency.
    pub total: Money,
    /// How much could not be read.
    pub left_out: LeftOut,
    /// The rows of every name of the side, those without a balance among them.
    by_name: BTreeMap<NameId, NameBalanceRow>,
}

impl NameBalances {
    /// The balance of a name, also of one that [`Self::rows`] does not have since
    /// it comes to nothing: then it is nothing in the currency of the name, and
    /// nothing in the company's. `None` for a name that is no customer, or no
    /// vendor.
    ///
    /// ```
    /// use qbw::{AsOf, Company, CurrencyId, NameId, Side};
    ///
    /// # let file = qbw_synthetic::company::books().build();
    /// let company = Company::from_bytes(file)?;
    /// let balances = company.name_balances(Side::Receivable, AsOf::Latest)?;
    /// // Bell Works trades in euros, and owes nothing.
    /// let bell = NameId::new(102);
    /// assert!(balances.rows.iter().all(|row| row.name != bell));
    /// let row = balances.balance(bell).ok_or("no such customer")?;
    /// let own = row.balance.amount;
    /// assert_eq!((own.to_string(), own.currency()), ("0.00".to_owned(), CurrencyId::new(2)));
    /// assert_eq!(row.total.currency(), CurrencyId::new(1));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn balance(&self, name: NameId) -> Option<NameBalanceRow> {
        self.by_name.get(&name).copied()
    }
}

/// A customer, a job or a vendor of [`NameBalances`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct NameBalanceRow {
    /// The name.
    pub name: NameId,
    /// Its own balance.
    pub balance: OwedBalance,
    /// The balance with those of its jobs, in the company's currency. Unchecked
    /// for a customer with jobs.
    pub total: Money,
}

impl NameBalanceRow {
    /// Whether the name has a balance, or a job with one.
    fn is_shown(&self) -> bool {
        let own = [self.balance.amount, self.balance.home_amount, self.total];
        own.iter().any(|amount| !amount.is_zero())
    }
}

pub(crate) fn name_balances(
    ledger: &Ledger,
    names: &Names,
    side: Side,
    as_of: AsOf,
) -> NameBalances {
    let balances = Balances::new(ledger, names, side, as_of);
    let known = |name: &NameId| names.get(*name).is_some();
    let unlisted = balances.sums.keys().copied();
    let unlisted = unlisted.filter(|name| !known(name));
    let all = listed(names, side).into_iter().chain(unlisted);
    let all: Vec<NameBalanceRow> = all.map(|name| balances.row(name)).collect();
    let total: Total = balances.sums.values().map(|sums| sums.home).sum();
    NameBalances {
        side,
        as_of,
        rows: all.iter().filter(|row| row.is_shown()).copied().collect(),
        total: side.owed(total).money(ledger.home()),
        left_out: ledger.left_out().union(names.left_out_on(side)),
        by_name: all.into_iter().map(|row| (row.name, row)).collect(),
    }
}

/// The balance of one name, also when it is nothing.
pub(crate) fn name_balance(
    ledger: &Ledger,
    names: &Names,
    side: Side,
    name: NameId,
    as_of: AsOf,
) -> NameBalanceRow {
    Balances::new(ledger, names, side, as_of).row(name)
}

/// The sums that balances of names are made of.
struct Balances<'company> {
    ledger: &'company Ledger,
    side: Side,
    sums: BTreeMap<NameId, Sums>,
    /// The sums in the company's currency, each with those of the jobs below.
    totals: BTreeMap<NameId, Total>,
}

impl<'company> Balances<'company> {
    fn new(ledger: &'company Ledger, names: &Names, side: Side, as_of: AsOf) -> Self {
        let sums = ledger.name_sums(side, as_of);
        Self {
            ledger,
            side,
            totals: with_jobs(&sums, names),
            sums,
        }
    }

    /// The row of a name, also when it has no balance.
    fn row(&self, name: NameId) -> NameBalanceRow {
        let sums = self.sums.get(&name).copied();
        let sums = sums.unwrap_or_else(|| self.ledger.sums(name));
        let total = self.totals.get(&name).copied().unwrap_or_default();
        NameBalanceRow {
            name,
            balance: OwedBalance::new(sums, self.side, self.ledger.home()),
            total: self.side.owed(total).money(self.ledger.home()),
        }
    }
}

/// The names of a side, in the order of their list.
fn listed(names: &Names, side: Side) -> Vec<NameId> {
    match side {
        Side::Receivable => names.customers().iter().map(|name| name.id).collect(),
        Side::Payable => names.vendors().iter().map(|name| name.id).collect(),
    }
}

/// The balances in the company's currency, each with those of the jobs below.
fn with_jobs(sums: &BTreeMap<NameId, Sums>, names: &Names) -> BTreeMap<NameId, Total> {
    let mut totals: BTreeMap<NameId, Total> = BTreeMap::new();
    for (&name, sums) in sums {
        for name in names.customers().lineage(name) {
            *totals.entry(name).or_default() += sums.home;
        }
    }
    totals
}

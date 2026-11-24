//! An account, and the reading of the list of them.

use std::collections::HashMap;

use sa17::Source;

use crate::accounts::{Reconciling, StoredBalance};
use crate::codes::{AccountType, SpecialAccount};
use crate::error::Result;
use crate::ids::{AccountId, CurrencyId};
use crate::list::{List, ListEntry, order, sealed};
use crate::money::Money;
use crate::origin::Origin;
use crate::tables::accounts::{ACCOUNT_TOTALS, ACCOUNTS, AccountRow, AccountTotalsRow};
use crate::unix_time::UnixTime;

/// An account of the chart of accounts.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Account {
    /// What the account is known by.
    pub id: AccountId,
    /// Its own name, without those of the accounts above it.
    pub name: String,
    /// The account it is a sub-account of.
    pub parent: Option<AccountId>,
    /// Its type.
    pub account_type: AccountType,
    /// What QuickBooks uses it for, when it made the account itself.
    pub special: Option<SpecialAccount>,
    /// Its number.
    pub number: Option<String>,
    /// What it is for.
    pub description: Option<String>,
    /// The currency it is kept in.
    pub currency: CurrencyId,
    /// The line of the income tax form that it goes to, as the file numbers it.
    pub tax_line: Option<i32>,
    /// Whether it can be used in new transactions.
    pub is_active: bool,
    /// The balance that QuickBooks keeps with it.
    pub stored: Option<StoredBalance>,
    /// The reconciliation that was started and is not finished.
    pub reconciling: Option<Reconciling>,
    /// When it was made.
    pub created: Option<UnixTime>,
    /// When it was last changed.
    pub modified: Option<UnixTime>,
    /// The stored row.
    pub origin: Origin,
}

impl sealed::Sealed for Account {}

impl ListEntry for Account {
    type Id = AccountId;

    fn id(&self) -> AccountId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn parent(&self) -> Option<AccountId> {
        self.parent
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn origin(&self) -> Origin {
        self.origin
    }
}

/// Reads the chart of accounts of a company that keeps its books in `home`.
///
/// # Errors
///
/// Returns an error when the file lacks a table or a column of it.
pub(crate) fn read(source: &dyn Source, home: CurrencyId) -> Result<List<Account>> {
    let mut damage = Vec::new();
    let totals: HashMap<AccountId, AccountTotalsRow> = ACCOUNT_TOTALS
        .open(source)?
        .scan(&mut damage)
        .map(|(_, totals)| (totals.id, totals))
        .collect();
    let mut accounts: Vec<Account> = ACCOUNTS
        .open(source)?
        .scan(&mut damage)
        .filter(|(_, row)| !row.is_deleted)
        .map(|(origin, row)| {
            let totals = totals.get(&row.id);
            account(origin, row, totals, home)
        })
        .collect();
    let links = totals
        .values()
        .map(|totals| (totals.id.number(), totals.next));
    order::sort(&mut accounts, links, |account| account.id.number());
    Ok(List::new(accounts, damage))
}

fn account(
    origin: Origin,
    row: AccountRow,
    totals: Option<&AccountTotalsRow>,
    home: CurrencyId,
) -> Account {
    let currency = row.currency.unwrap_or(home);
    let own = |amount| Money::new(amount, currency);
    Account {
        id: row.id,
        name: row.name.unwrap_or_default(),
        parent: row.parent,
        account_type: row.account_type,
        special: totals.and_then(|totals| totals.special),
        number: row.number,
        description: row.description,
        currency,
        tax_line: row.tax_line.filter(|&line| line != 0),
        is_active: !row.is_hidden,
        stored: totals.and_then(|totals| {
            Some(StoredBalance {
                amount: own(totals.balance?),
                home_amount: totals.home_balance.map(|amount| Money::new(amount, home)),
            })
        }),
        reconciling: totals.and_then(|totals| {
            Some(Reconciling {
                statement_date: totals.statement_date,
                statement_balance: own(totals.statement_balance?),
            })
        }),
        created: row.created,
        modified: row.modified,
        origin,
    }
}

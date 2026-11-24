//! The overviews of a company and its entries.

use sa17::Source;

use crate::as_of::AsOf;
use crate::company::Company;
use crate::error::Result;
use crate::ids::{AccountId, ItemId, NameId};
use crate::names::Name;
use crate::overviews::{AccountOverview, CompanyOverview, ItemOverview, NameOverview};
use crate::page::Page;
use crate::query::TransactionQuery;
use crate::reports;
use crate::side::Side;

/// # What a person sees
impl<S: Source> Company<S> {
    /// What there is in the bank, what is owed and overdue, and the latest
    /// transactions: at most `rows` of those dated up to `as_of`.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::account_listing`], [`Self::open_items`] and
    /// [`Self::transactions`].
    pub fn overview(&self, as_of: impl Into<AsOf>, rows: usize) -> Result<CompanyOverview> {
        let as_of = as_of.into();
        let owed = [
            self.owed(Side::Receivable, None, as_of)?,
            self.owed(Side::Payable, None, as_of)?,
        ];
        let query = TransactionQuery::new().dated(as_of.period());
        let transactions = self.transactions(&query, Page::new(0, rows))?;
        let listing = self.account_listing(as_of)?;
        let accounts = self.accounts()?;
        Ok(CompanyOverview::new(accounts, listing, owed, transactions))
    }

    /// What there is about a name, with its latest transactions: at most `rows`
    /// of those dated up to `as_of`. `None` when there is no such name.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::names`], [`Self::owed_by`] and
    /// [`Self::transactions`].
    pub fn name_overview(
        &self,
        name: NameId,
        as_of: impl Into<AsOf>,
        rows: usize,
    ) -> Result<Option<NameOverview<'_>>> {
        let as_of = as_of.into();
        let names = self.names()?;
        let Some(record) = names.get(name) else {
            return Ok(None);
        };
        let owed = match record {
            Name::Customer(_) => self.owed_by(name, as_of)?,
            Name::Vendor(_) => self.owed_to(name, as_of)?,
            Name::Employee(_) | Name::Other(_) => None,
        };
        let ledger = self.ledger()?;
        let side = owed.as_ref().map(|owed| owed.side);
        let balance = side.map(|side| reports::name_balance(ledger, names, side, name, as_of));
        let query = TransactionQuery::new().name(name).with_jobs();
        let query = query.dated(as_of.period());
        let transactions = self.transactions(&query, Page::new(0, rows))?;
        let left_out = ledger.left_out().union(names.left_out());
        let rest = (transactions, left_out);
        Ok(Some(NameOverview::new(record, as_of, balance, owed, rest)))
    }

    /// What there is about an item now, with at most `rows` of the latest lines
    /// with it. `None` when there is no such item.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::stock`] and [`Self::ledger`].
    pub fn item_overview(&self, item: ItemId, rows: usize) -> Result<Option<ItemOverview<'_>>> {
        let Some(record) = self.items()?.get(item) else {
            return Ok(None);
        };
        let stock = record.kind.is_stocked().then(|| self.stock());
        let stock = stock.transpose()?;
        let books = (self.ledger()?, self.header_index()?);
        let reader = self.reader()?;
        ItemOverview::read(record, stock.as_ref(), books, &reader, rows).map(Some)
    }

    /// What there is about an account, with the latest rows of its register: at
    /// most `rows` of those dated up to `as_of`. `None` when there is no such
    /// account.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::account_listing`] and [`Self::register`].
    pub fn account_overview(
        &self,
        account: AccountId,
        as_of: impl Into<AsOf>,
        rows: usize,
    ) -> Result<Option<AccountOverview<'_>>> {
        let as_of = as_of.into();
        let Some(record) = self.accounts()?.get(account) else {
            return Ok(None);
        };
        let period = as_of.period();
        let row_count = self.ledger()?.during(account, period).len();
        let latest = Page::new(row_count.saturating_sub(rows), rows);
        let register = self.register(account, period, latest)?;
        let listing = self.account_listing(as_of)?;
        Ok(register.map(|register| AccountOverview::new(record, &listing, register)))
    }
}

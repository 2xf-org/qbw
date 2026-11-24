//! What customers owe and vendors are owed.

use std::collections::BTreeMap;

use crate::as_of::AsOf;
use crate::ids::NameId;
use crate::ledger::{Balance, Entry, Ledger, Sums};
use crate::side::Side;

impl Ledger {
    /// What a customer or job owes at a moment: the postings with the customer on
    /// accounts of type accounts receivable. A customer's jobs are not part of
    /// it. `None` when there are no such postings.
    #[must_use]
    pub fn customer_balance(&self, customer: NameId, as_of: impl Into<AsOf>) -> Option<Balance> {
        self.name_balance(Side::Receivable, customer, as_of.into())
    }

    /// What a vendor is owed at a moment: the postings with the vendor on
    /// accounts of type accounts payable and on the sales tax that QuickBooks
    /// keeps. `None` when there are no such postings.
    #[must_use]
    pub fn vendor_balance(&self, vendor: NameId, as_of: impl Into<AsOf>) -> Option<Balance> {
        self.name_balance(Side::Payable, vendor, as_of.into())
    }

    /// [`Self::customer_balance`] of every customer and job with a posting.
    #[must_use]
    pub fn customer_balances(&self, as_of: impl Into<AsOf>) -> BTreeMap<NameId, Balance> {
        self.name_balances(Side::Receivable, as_of.into())
    }

    /// [`Self::vendor_balance`] of every vendor with a posting.
    #[must_use]
    pub fn vendor_balances(&self, as_of: impl Into<AsOf>) -> BTreeMap<NameId, Balance> {
        self.name_balances(Side::Payable, as_of.into())
    }

    /// The sums of the names of a side.
    pub(crate) fn name_sums(&self, side: Side, as_of: AsOf) -> BTreeMap<NameId, Sums> {
        let mut sums: BTreeMap<NameId, Sums> = BTreeMap::new();
        for (name, entry) in self.named(side, as_of) {
            let sums = sums.entry(name).or_insert_with(|| self.sums(name));
            sums.add(entry);
        }
        sums
    }

    /// The entries of a name on a side, by date and by line.
    pub(crate) fn name_entries(&self, side: Side, name: NameId, as_of: AsOf) -> Vec<&Entry> {
        let named = self.named(side, as_of);
        let entries = named.filter(|(found, _)| *found == name);
        let mut entries: Vec<&Entry> = entries.map(|(_, entry)| entry).collect();
        entries.sort_by_key(|entry| (entry.date, entry.line, entry.source));
        entries
    }

    /// The balance of nothing, in the currency that a name trades in.
    pub(crate) fn sums(&self, name: NameId) -> Sums {
        let currency = self.name_currencies.get(&name).copied();
        Sums::new(self.home(), currency.unwrap_or(self.home()))
    }

    fn name_balance(&self, side: Side, name: NameId, as_of: AsOf) -> Option<Balance> {
        let named = self.named(side, as_of);
        let mut entries = named.filter(|(found, _)| *found == name).peekable();
        entries.peek()?;
        let mut sums = self.sums(name);
        for (_, entry) in entries {
            sums.add(entry);
        }
        Some(sums.balance(self.left_out()))
    }

    fn name_balances(&self, side: Side, as_of: AsOf) -> BTreeMap<NameId, Balance> {
        let sums = self.name_sums(side, as_of).into_iter();
        let balances = sums.map(|(name, sums)| (name, sums.balance(self.left_out())));
        balances.collect()
    }

    /// The entries of a side that have a name, each with the name.
    fn named(&self, side: Side, as_of: AsOf) -> impl Iterator<Item = (NameId, &Entry)> {
        let accounts = self.accounts.iter();
        let accounts = accounts.filter(move |account| account.side == Some(side));
        accounts.flat_map(move |account| {
            let entries = self.entries_during(account.account, as_of.period()).iter();
            entries.filter_map(move |entry| Some((entry.name_on(side)?, entry)))
        })
    }
}

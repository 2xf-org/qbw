//! Comparing what is open with the links and with the accounts.

use std::collections::{BTreeMap, BTreeSet};

use crate::accounts::Account;
use crate::amount::Total;
use crate::as_of::AsOf;
use crate::check::tally::Figures;
use crate::check::{Cause, CheckKind, Subject, Tally};
use crate::ids::{AccountId, NameId, TransactionId};
use crate::ledger::Ledger;
use crate::list::List;
use crate::money::Money;
use crate::open_items::{OpenItem, OpenItems};
use crate::side::Side;

/// What is open on one side, by two ways of working it out.
pub(crate) struct Open {
    /// What the file keeps as open.
    pub(crate) kept: OpenItems,
    /// What the links leave open.
    pub(crate) linked: OpenItems,
}

impl Open {
    /// Whether what is kept as open is what the links leave, and comes to the
    /// balances of the accounts.
    pub(crate) fn check(&self, tally: &mut Tally, ledger: &Ledger, accounts: &List<Account>) {
        let accounts = accounts.iter();
        let accounts = accounts
            .filter(|account| account.account_type == self.kept.side.account_type())
            .map(|account| account.id)
            .filter(|&id| !ledger.entries(id).is_empty());
        let accounts: Vec<AccountId> = accounts.collect();
        self.check_transactions(tally, ledger, &accounts);
        self.check_accounts(tally, ledger, &accounts);
    }

    fn check_transactions(&self, tally: &mut Tally, ledger: &Ledger, accounts: &[AccountId]) {
        let what = match self.kept.side {
            Side::Receivable => CheckKind::OpenReceivable,
            Side::Payable => CheckKind::OpenPayable,
        };
        let entries = accounts.iter().flat_map(|&id| ledger.entries(id));
        let transactions: BTreeSet<_> = entries.map(|entry| entry.transaction).collect();
        let kept = by_transaction(&self.kept);
        let linked = by_transaction(&self.linked);
        for transaction in transactions {
            let kept = kept.get(&transaction).map(Vec::as_slice);
            let linked = linked.get(&transaction).map(Vec::as_slice);
            let (kept, linked) = (kept.unwrap_or_default(), linked.unwrap_or_default());
            let figures = different(linked, kept, ledger);
            let subject = Subject::Transaction(transaction);
            tally.compare(what, subject, figures, || self.causes());
        }
    }

    fn check_accounts(&self, tally: &mut Tally, ledger: &Ledger, accounts: &[AccountId]) {
        let side = self.kept.side;
        let what = match side {
            Side::Receivable => CheckKind::ReceivableAccount,
            Side::Payable => CheckKind::PayableAccount,
        };
        let kept = by_account(&self.kept);
        let linked = by_account(&self.linked);
        for &account in accounts {
            let Some(balance) = ledger.balance(account, AsOf::Latest) else {
                continue;
            };
            let balance = side.owed(balance.amount);
            let currency = balance.currency();
            let kept = kept.get(&account).copied().unwrap_or_default();
            let linked = linked.get(&account).copied().unwrap_or_default();
            let against_links = (linked - kept).money(currency);
            let subject = Subject::Account(account);
            let figures = Figures::new(kept.money(currency), balance);
            tally.compare(what, subject, figures, || {
                let mut causes = self.causes();
                if !against_links.is_zero() {
                    causes.push(Cause::KeptAsOpen(against_links));
                }
                causes
            });
        }
    }

    pub(super) fn causes(&self) -> Vec<Cause> {
        Cause::of_missing(self.linked.amounts_left_out)
    }
}

/// The items of each transaction.
fn by_transaction(open: &OpenItems) -> BTreeMap<TransactionId, Vec<&OpenItem>> {
    let mut items: BTreeMap<TransactionId, Vec<&OpenItem>> = BTreeMap::new();
    for item in open {
        items.entry(item.transaction).or_default().push(item);
    }
    items
}

/// What the items of each account come to.
fn by_account(open: &OpenItems) -> BTreeMap<AccountId, Total> {
    let mut sums: BTreeMap<AccountId, Total> = BTreeMap::new();
    for item in open {
        *sums.entry(item.account).or_default() += item.sums().open;
    }
    sums
}

/// The first two amounts that the items of one transaction do not share, or twice
/// nothing when they share them all.
fn different(linked: &[&OpenItem], kept: &[&OpenItem], ledger: &Ledger) -> Figures {
    // A journal entry can have a debit and a credit of one name.
    let part = |item: &OpenItem| (item.account, item.name, item.total.is_negative());
    let parts = linked.iter().chain(kept);
    let parts: BTreeSet<_> = parts.map(|item| part(item)).collect();
    let open = |items: &[&OpenItem], of: (AccountId, Option<NameId>, bool)| {
        let (account, ..) = of;
        let mut items = items.iter();
        let item = items.find(|item| part(item) == of);
        let nothing = || Money::zero(ledger.currency(account));
        item.map_or_else(nothing, |item| item.open)
    };
    let mut pairs = parts
        .into_iter()
        .map(|part| (open(linked, part), open(kept, part)));
    let (linked, kept) = pairs.find(|(linked, kept)| linked != kept).unzip();
    Figures {
        computed: linked.map(Into::into),
        expected: kept.map(Into::into),
    }
}

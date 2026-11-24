//! Comparing the ledger with the balances that QuickBooks keeps.

use crate::accounts::{Account, StoredBalance};
use crate::amount::Total;
use crate::as_of::AsOf;
use crate::check::tally::Figures;
use crate::check::{Cause, CheckKind, Subject, Tally};
use crate::ids::CurrencyId;
use crate::ledger::{Balance, Ledger};
use crate::list::List;
use crate::money::Money;
use crate::names::Names;

/// Whether every transaction, and the whole ledger, comes to zero.
pub(crate) fn ledger(tally: &mut Tally, ledger: &Ledger) {
    let causes = || Cause::of_missing(ledger.left_out());
    let mut whole = Total::default();
    let entries: Vec<_> = ledger.entries_by_transaction().collect();
    for entries in entries.chunk_by(|left, right| left.transaction == right.transaction) {
        let total: Total = entries.iter().map(|entry| entry.home).sum();
        whole += total;
        if let Some(first) = entries.first() {
            let subject = Subject::Transaction(first.transaction);
            let figures = with_zero(total, ledger.home());
            tally.compare(CheckKind::TransactionBalance, subject, figures, causes);
        }
    }
    let figures = with_zero(whole, ledger.home());
    tally.compare(CheckKind::LedgerBalance, Subject::Books, figures, causes);
}

/// Whether the postings come to the balances that QuickBooks keeps.
pub(crate) fn accounts(tally: &mut Tally, ledger: &Ledger, accounts: &List<Account>) {
    let mut kept = Kept::new(tally, ledger);
    for account in accounts {
        if let Some(stored) = account.stored {
            let balance = ledger.balance(account.id, AsOf::Latest);
            let of = (Subject::Account(account.id), account.currency);
            kept.compare(CheckKind::AccountBalance, of, balance, stored);
        }
    }
}

/// Whether the postings of the names come to the balances that QuickBooks keeps.
pub(crate) fn names(tally: &mut Tally, ledger: &Ledger, names: &Names) {
    let mut kept = Kept::new(tally, ledger);
    let balances = ledger.customer_balances(AsOf::Latest);
    for customer in names.customers() {
        if let Some(stored) = customer.stored {
            let balance = balances.get(&customer.id).copied();
            let of = (Subject::Name(customer.id), customer.currency);
            kept.compare(CheckKind::CustomerBalance, of, balance, stored);
        }
    }
    let balances = ledger.vendor_balances(AsOf::Latest);
    for vendor in names.vendors() {
        if let Some(stored) = vendor.stored {
            let balance = balances.get(&vendor.id).copied();
            let of = (Subject::Name(vendor.id), vendor.currency);
            kept.compare(CheckKind::VendorBalance, of, balance, stored);
        }
    }
}

/// Compares balances with those that QuickBooks keeps.
struct Kept<'check> {
    tally: &'check mut Tally,
    ledger: &'check Ledger,
    causes: Vec<Cause>,
}

impl<'check> Kept<'check> {
    fn new(tally: &'check mut Tally, ledger: &'check Ledger) -> Self {
        Self {
            causes: causes(tally, ledger),
            tally,
            ledger,
        }
    }

    /// Compares a balance in both currencies. A finding has the one that differs:
    /// the own one, or else the company's. Without postings it is nothing, in the
    /// currency given with the subject.
    fn compare(
        &mut self,
        what: CheckKind,
        (subject, currency): (Subject, CurrencyId),
        balance: Option<Balance>,
        stored: StoredBalance,
    ) {
        let nothing = [Money::zero(currency), Money::zero(self.ledger.home())];
        let posted = balance.map(|balance| [balance.amount, balance.home_amount]);
        let [amount, home_amount] = posted.unwrap_or(nothing);
        let home = stored.home_amount.filter(|_| amount == stored.amount);
        let figures = match home {
            Some(home) => Figures::new(home_amount, home),
            None => Figures::new(amount, stored.amount),
        };
        let causes = &self.causes;
        self.tally
            .compare(what, subject, figures, || causes.clone());
    }
}

/// Why a balance that QuickBooks keeps is another one than the postings come to.
fn causes(tally: &Tally, ledger: &Ledger) -> Vec<Cause> {
    if tally.all_agree(CheckKind::TransactionBalance) {
        Cause::of_kept(ledger.left_out())
    } else {
        Cause::of_missing(ledger.left_out())
    }
}

fn with_zero(total: Total, home: CurrencyId) -> Figures {
    Figures::new(total.money(home), Money::zero(home))
}

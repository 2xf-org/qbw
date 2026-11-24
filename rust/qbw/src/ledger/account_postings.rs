//! Where the postings of an account are.

use std::collections::BTreeSet;
use std::ops::Range;

use crate::accounts::Account;
use crate::codes::{AccountType, SpecialAccount};
use crate::ids::{AccountId, CurrencyId};
use crate::ledger::Entry;
use crate::list::List;
use crate::side::Side;

/// Where the entries of an account are among the ledger's, and what the ledger
/// needs to know of the account.
#[derive(Debug)]
pub(super) struct AccountPostings {
    pub(super) account: AccountId,
    /// The currency the account is kept in.
    pub(super) currency: CurrencyId,
    /// The side whose names' balances its postings add up to.
    pub(super) side: Option<Side>,
    pub(super) entries: Range<usize>,
}

impl AccountPostings {
    /// Every account that is listed or posted to, ordered by its id, with its
    /// place in `entries`, which are ordered by their accounts. One that is not
    /// listed counts as kept in the currency `home`.
    pub(super) fn all(accounts: &List<Account>, entries: &[Entry], home: CurrencyId) -> Vec<Self> {
        let listed = accounts.iter().map(|account| account.id);
        let posted = entries.chunk_by(|left, right| left.account == right.account);
        let posted = posted.filter_map(|entries| entries.first());
        let posted = posted.map(|entry| entry.account);
        let ids: BTreeSet<AccountId> = listed.chain(posted).collect();
        let found = ids.into_iter().map(|id| {
            let account = accounts.get(id);
            Self {
                account: id,
                currency: account.map_or(home, |account| account.currency),
                side: account.and_then(side),
                entries: entries.partition_point(|entry| entry.account < id)
                    ..entries.partition_point(|entry| entry.account <= id),
            }
        });
        found.collect()
    }
}

/// The side whose names' balances the postings of an account add up to. On the
/// side of the vendors that is sales tax, too.
///
/// The accounts are found by their type, since not every company's have the code
/// of QuickBooks' own. Sales tax has no type of its own.
fn side(account: &Account) -> Option<Side> {
    match (account.account_type, account.special) {
        (AccountType::AccountsReceivable, _) => Some(Side::Receivable),
        (AccountType::AccountsPayable, _) | (_, Some(SpecialAccount::SalesTaxPayable)) => {
            Some(Side::Payable)
        }
        _ => None,
    }
}

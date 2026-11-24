//! Nesting the accounts of a section.

use std::collections::HashMap;

use crate::accounts::Account;
use crate::amount::Total;
use crate::codes::AccountType;
use crate::ids::AccountId;
use crate::list::DEEPEST;
use crate::reports::{Books, Section, SectionAccount};

/// The amount of one account in a section.
#[derive(Clone, Copy)]
pub(super) struct AccountAmount {
    /// A debit where it is positive.
    pub(super) total: Total,
    /// Whether the account is worth a line, even where its amount is zero.
    pub(super) is_shown: bool,
}

/// A section, with its total as a number to go on with.
pub(super) struct Summed {
    pub(super) section: Section,
    /// In the section's own sign.
    pub(super) total: Total,
}

/// The section of the accounts of one type, with the amount that `amount` gives
/// for each. No account of the type is lost, whatever its parents are.
pub(super) fn of(
    books: Books<'_>,
    account_type: AccountType,
    amount: impl Fn(&Account) -> AccountAmount,
) -> Summed {
    let members = books.accounts.iter();
    let members: Vec<&Account> = members
        .filter(|account| account.account_type == account_type)
        .collect();
    let mut tree = Tree {
        books,
        account_type,
        amounts: members
            .iter()
            .map(|account| (account.id, amount(account)))
            .collect(),
        below: HashMap::new(),
    };
    let mut tops = Vec::new();
    let mut nested = Vec::new();
    for account in &members {
        match account
            .parent
            .filter(|parent| tree.amounts.contains_key(parent))
        {
            Some(parent) => {
                tree.below.entry(parent).or_default().push(account.id);
                nested.push(account.id);
            }
            None => tops.push(account.id),
        }
    }
    // The nested ones that no account at the top leads to come last.
    tops.extend(nested);
    let placed: Vec<Placed> = tops
        .into_iter()
        .filter_map(|id| tree.place(id, 0))
        .collect();
    let total = placed.iter().map(|placed| placed.total).sum();
    let shown = placed.into_iter().filter(|placed| placed.is_shown);
    let section = Section {
        account_type,
        accounts: shown.map(|placed| placed.account).collect(),
        total: books.money(total),
    };
    Summed { section, total }
}

struct Tree<'company> {
    books: Books<'company>,
    account_type: AccountType,
    /// The amounts of the accounts that have no place yet.
    amounts: HashMap<AccountId, AccountAmount>,
    below: HashMap<AccountId, Vec<AccountId>>,
}

/// An account in its place, with those below it.
struct Placed {
    account: SectionAccount,
    total: Total,
    is_shown: bool,
}

impl Tree<'_> {
    /// The account `id`, `depth` accounts down, or `None` when it has a place.
    /// What is too deep below it stands at the top instead.
    fn place(&mut self, id: AccountId, depth: usize) -> Option<Placed> {
        let amount = self.amounts.remove(&id)?;
        let below = self.below.remove(&id).filter(|_| depth < DEEPEST);
        let below = below.unwrap_or_default().into_iter();
        let below: Vec<Placed> = below.filter_map(|id| self.place(id, depth + 1)).collect();

        let own = self.account_type.as_kept(amount.total);
        let total = own + below.iter().map(|placed| placed.total).sum();
        let shown = below.into_iter().filter(|placed| placed.is_shown);
        let children: Vec<SectionAccount> = shown.map(|placed| placed.account).collect();
        Some(Placed {
            is_shown: amount.is_shown || !children.is_empty(),
            total,
            account: SectionAccount {
                account: id,
                amount: self.books.money(own),
                total: self.books.money(total),
                children,
            },
        })
    }
}

//! A part of a statement.

use std::iter;

use crate::codes::AccountType;
use crate::ids::AccountId;
use crate::money::Money;

/// A part of a statement: the accounts of one type with their amounts, nested
/// as they are in the list.
///
/// Amounts are in the company's currency and have the sign that the section
/// grows by: income and debts are positive where they are in order, though
/// they are credits.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Section {
    /// The type of the accounts, which says what the section is.
    pub account_type: AccountType,
    /// The accounts at the top, in the list's order. Those that the statement
    /// has nothing to say about are left out.
    pub accounts: Vec<SectionAccount>,
    /// What all the accounts come to.
    pub total: Money,
}

/// An account of a [`Section`], with the accounts below it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SectionAccount {
    /// The account.
    pub account: AccountId,
    /// What is posted to the account itself.
    pub amount: Money,
    /// What the account and those below it come to.
    pub total: Money,
    /// The accounts right below it, in the list's order.
    pub children: Vec<SectionAccount>,
}

impl Section {
    /// Every account of the section in the list's order, each before the ones
    /// below it, with how many accounts it is below.
    pub fn walk(&self) -> impl Iterator<Item = (usize, &SectionAccount)> {
        let mut waiting: Vec<(usize, &SectionAccount)> = at_depth(0, &self.accounts);
        iter::from_fn(move || {
            let (depth, account) = waiting.pop()?;
            waiting.append(&mut at_depth(depth + 1, &account.children));
            Some((depth, account))
        })
    }
}

/// Accounts with their depth, the last one first.
fn at_depth(depth: usize, accounts: &[SectionAccount]) -> Vec<(usize, &SectionAccount)> {
    let accounts = accounts.iter().rev();
    accounts.map(|account| (depth, account)).collect()
}

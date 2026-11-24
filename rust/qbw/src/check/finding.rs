//! A difference.

use std::fmt;

use sa17::Decimal;

use crate::accounts::Account;
use crate::check::{Cause, CheckKind};
use crate::ids::{AccountId, ItemId, NameId, SiteId, TransactionId};
use crate::list::List;
use crate::money::Money;
use crate::side::Side;

/// Two figures that should be the same, and are not.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Finding {
    /// What was compared.
    pub what: CheckKind,
    /// What it is about.
    pub subject: Subject,
    /// What is worked out. `None` where there is no figure, such as of a chain
    /// of lines that is broken.
    ///
    /// Postings, and balances of accounts, customers and vendors, have a debit as
    /// positive. What is open is above zero when it is owed, and so are the
    /// balances of the accounts that it is compared with. A form's figures are as
    /// the form shows them, and those of a year as profit and loss does. The
    /// amounts of [`Self::causes`] have the sign of the figures, and
    /// [`Self::as_kept`] turns them all.
    pub computed: Option<Figure>,
    /// What it should be: what QuickBooks keeps, or what follows from the rest
    /// of the books. It has the sign of [`Self::computed`].
    pub expected: Option<Figure>,
    /// What explains the difference, and what may be behind it.
    pub causes: Vec<Cause>,
}

/// What a finding is about.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Subject {
    /// All of the books.
    Books,
    /// An account.
    Account(AccountId),
    /// A customer or a vendor.
    Name(NameId),
    /// A transaction.
    Transaction(TransactionId),
    /// An item.
    Item(ItemId),
    /// An item at a site or in a bin.
    ItemAt(ItemId, SiteId),
    /// A figure that QuickBooks worked out, by its key among the
    /// [facts](crate::settings::Facts).
    Fact(String),
}

/// A figure of a finding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Figure {
    /// An amount.
    Money(Money),
    /// A quantity.
    Quantity(Decimal),
}

impl Finding {
    /// Whether the amounts of the causes come to the difference. What only may be
    /// a cause explains nothing.
    #[must_use]
    pub fn is_explained(&self) -> bool {
        let (Some(Figure::Money(computed)), Some(Figure::Money(expected))) =
            (self.computed, self.expected)
        else {
            return false;
        };
        let mut parts = self.causes.iter().filter_map(|cause| cause.amount());
        let explained = parts.try_fold(computed, Money::checked_add);
        explained == Ok(expected)
    }
}

impl Finding {
    /// The same finding with its figures, and the amounts of its causes, as the
    /// account or the name of it is kept: above zero on the side that the account
    /// grows on, as [`AccountType::as_kept`] has it, and above zero for what a
    /// vendor is owed. The others are above zero where they are in order as it is.
    /// `accounts` are the company's.
    ///
    /// [`AccountType::as_kept`]: crate::accounts::AccountType::as_kept
    #[must_use]
    pub fn as_kept(&self, accounts: &List<Account>) -> Self {
        let turned = |amount: Money| match (self.what, &self.subject) {
            (CheckKind::VendorBalance, _) => Side::Payable.owed(amount),
            (CheckKind::AccountBalance | CheckKind::InventoryAsset, Subject::Account(account)) => {
                let account = accounts.get(*account);
                account.map_or(amount, |account| account.account_type.as_kept(amount))
            }
            _ => amount,
        };
        let figure = |figure| match figure {
            Figure::Money(amount) => Figure::Money(turned(amount)),
            Figure::Quantity(_) => figure,
        };
        let causes = self.causes.iter().map(|cause| cause.map_amount(turned));
        Self {
            what: self.what,
            subject: self.subject.clone(),
            computed: self.computed.map(figure),
            expected: self.expected.map(figure),
            causes: causes.collect(),
        }
    }
}

impl From<Money> for Figure {
    fn from(amount: Money) -> Self {
        Self::Money(amount)
    }
}

impl From<Decimal> for Figure {
    fn from(quantity: Decimal) -> Self {
        Self::Quantity(quantity)
    }
}

impl fmt::Display for Figure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Money(amount) => amount.fmt(formatter),
            Self::Quantity(quantity) => quantity.fmt(formatter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::CurrencyId;

    fn money(text: &str) -> Money {
        Money::new(text.parse().unwrap(), CurrencyId::COMPANY)
    }

    fn is_explained(computed: Option<Figure>, causes: &[Cause]) -> bool {
        let found = Finding {
            what: CheckKind::InventoryAsset,
            subject: Subject::Books,
            computed,
            expected: Some(money("100").into()),
            causes: causes.to_vec(),
        };
        found.is_explained()
    }

    #[test]
    fn amounts_explain_a_difference_when_they_come_to_it() {
        let found = Some(money("90").into());
        let parts = [
            Cause::OtherPostings(money("7")),
            Cause::KeptAsOpen(money("3")),
        ];
        assert!(is_explained(found, &parts));
        assert!(!is_explained(found, &parts[..1]));
        assert!(!is_explained(found, &[]));
        assert!(!is_explained(None, &parts));
        // What may be a cause neither explains nor takes away.
        assert!(!is_explained(found, &[Cause::OutOfDate]));
        assert!(!is_explained(found, &[Cause::BuildCosts, parts[0]]));
        let with_a_guess = [parts[0], parts[1], Cause::Unread];
        assert!(is_explained(found, &with_a_guess));
    }
}

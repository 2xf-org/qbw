//! What is left to pay of a transaction.

use crate::ids::CurrencyId;
use crate::money::Money;

/// What is left to pay of an invoice or a bill, or to use of a payment or a
/// credit.
///
/// It is what is left of the lines that customers owe or vendors are owed: the
/// own line of an invoice and of a bill, and another line of a payment. A
/// transaction without one, such as a check that pays an expense, has nothing
/// that could be left.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum OpenState {
    /// Nothing: it was paid, or used up.
    Paid,
    /// So much, as the form shows amounts: above zero, also for a payment or a
    /// credit that is not used up. A journal entry shows a debit as positive,
    /// and what is left of one can come to zero over several names.
    Open(Money),
    /// Nothing is ever paid or used of it, or not while it does not post.
    NotTracked,
}

impl OpenState {
    /// What is left of a transaction in `currency`. `None` when nothing is ever
    /// paid or used of it.
    pub(crate) fn amount(self, currency: CurrencyId) -> Option<Money> {
        match self {
            Self::Paid => Some(Money::zero(currency)),
            Self::Open(open) => Some(open),
            Self::NotTracked => None,
        }
    }

    /// Whether it is paid or used up, of what can be.
    #[must_use]
    pub const fn is_paid(self) -> Option<bool> {
        match self {
            Self::Paid => Some(true),
            Self::Open(_) => Some(false),
            Self::NotTracked => None,
        }
    }
}

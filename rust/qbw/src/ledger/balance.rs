//! What postings add up to.

use std::ops::Neg;

use crate::left_out::LeftOut;
use crate::money::Money;

/// The balance of an account, or what a customer owes or a vendor is owed.
///
/// A debit is positive. An account that grows by credits has a balance below
/// zero when it is in order, and so has a vendor who is owed something.
/// [`AccountType::as_kept`](crate::accounts::AccountType::as_kept) turns it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Balance {
    /// The balance in the currency of the account or of the name.
    pub amount: Money,
    /// The balance in the company's currency. For an account or a name in that
    /// currency it is the same amount.
    pub home_amount: Money,
    /// How much of the ledger could not be read. Anything that is left out may
    /// belong to this balance.
    pub left_out: LeftOut,
}

impl Neg for Balance {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            amount: -self.amount,
            home_amount: -self.home_amount,
            left_out: self.left_out,
        }
    }
}

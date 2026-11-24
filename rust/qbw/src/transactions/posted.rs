//! What a line posts.

use crate::ids::AccountId;
use crate::money::Money;

/// What a line puts into the books. A debit is positive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Posted {
    /// The account.
    pub account: AccountId,
    /// The amount in the transaction's currency. Zero for an exchange
    /// difference, which changes nothing in that currency.
    pub amount: Money,
    /// The amount in the company's currency.
    pub home_amount: Money,
}

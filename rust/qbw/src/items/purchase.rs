//! The side of an item that vendors see.

use crate::ids::{AccountId, NameId};
use crate::money::Money;

/// What an item costs that is bought and sold.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Purchase {
    /// The account that its cost goes to: when it is sold for an item in
    /// stock, and when it is bought for any other.
    pub account: Option<AccountId>,
    /// What one costs.
    pub cost: Option<Money>,
    /// The vendor it is bought from.
    pub vendor: Option<NameId>,
    /// What the lines of purchases say.
    pub description: Option<String>,
}

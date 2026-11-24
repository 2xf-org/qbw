//! The side of an item that customers see.

use crate::ids::AccountId;
use crate::price::Price;

/// What a line of an item says and posts to.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Sales {
    /// The account that it posts to when it is sold, or the only one that an
    /// item has. A subtotal and a group have none.
    pub account: Option<AccountId>,
    /// What it is sold for: the rate of a sales tax, and of a tax group the
    /// sum of its taxes.
    pub price: Option<Price>,
    /// What its lines say.
    pub description: Option<String>,
}

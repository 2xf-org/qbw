//! A link between two transactions.

use sa17::Decimal;

use crate::amount::Total;
use crate::links::{LinkedTransaction, Relation};
use crate::money::Money;

/// How a transaction belongs to another, with what one of them pays, credits,
/// received or invoiced of the other in all.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Link {
    /// How it belongs to the other.
    pub relation: Relation,
    /// The other transaction.
    pub other: LinkedTransaction,
    /// The amount, above zero for what is paid, credited or taken off. `None`
    /// where only goods were received.
    pub amount: Option<Money>,
    /// The quantity received or invoiced, over all items.
    pub quantity: Option<Decimal>,
    /// The amount, to add it to others.
    pub(super) sum: Total,
}

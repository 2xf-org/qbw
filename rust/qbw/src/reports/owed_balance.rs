//! What a name owes or is owed.

use crate::ids::CurrencyId;
use crate::ledger::Sums;
use crate::money::Money;
use crate::side::Side;

/// What a customer owes or a vendor is owed. Either is above zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OwedBalance {
    /// In the currency that the name trades in.
    pub amount: Money,
    /// In the company's currency.
    pub home_amount: Money,
}

impl OwedBalance {
    /// The balance `sums` on `side`, of a company that keeps its books in `home`.
    pub(super) fn new(sums: Sums, side: Side, home: CurrencyId) -> Self {
        Self {
            amount: side.owed(sums.own).money(sums.own_currency),
            home_amount: side.owed(sums.home).money(home),
        }
    }
}

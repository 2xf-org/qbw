//! The balance that QuickBooks keeps.

use crate::money::Money;

/// The balance that QuickBooks keeps with an entry: a total over all time.
///
/// It can be out of date until the file is rebuilt, so it serves to check the
/// balance that the postings give.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct StoredBalance {
    /// The balance in the entry's own currency. A debit is positive.
    pub amount: Money,
    /// The balance in the company's currency, where the file keeps it.
    pub home_amount: Option<Money>,
}

//! What is open, by age.

use chrono::NaiveDate;

use crate::aging::Bucket;
use crate::as_of::AsOf;
use crate::ids::NameId;
use crate::left_out::LeftOut;
use crate::money::Money;
use crate::open_items::OpenItem;
use crate::settings::AgingBasis;
use crate::side::Side;

/// What is open at a moment, by how late it is.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Aging {
    /// Receivable or payable.
    pub side: Side,
    /// The moment.
    pub as_of: AsOf,
    /// The day that ages are counted to: that of the moment.
    pub day: NaiveDate,
    /// What ages are counted from.
    pub basis: AgingBasis,
    /// The buckets: what is not late, then bucket after bucket.
    pub buckets: Vec<Bucket>,
    /// The rows. Those of names are in the list's order, and those of items by
    /// bucket and by date.
    pub rows: Vec<AgingRow>,
    /// What the rows come to, in the company's currency.
    pub totals: Aged,
    /// What changes of exchange rates added to accounts in other currencies. It
    /// is of no item and has no age, and the totals with it are the balances of
    /// the accounts.
    pub adjustments: Money,
    /// How much could not be read.
    pub left_out: LeftOut,
}

/// A row of an [`Aging`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AgingRow {
    /// The customer or the vendor.
    pub name: Option<NameId>,
    /// The item, in a row of one.
    pub item: Option<AgedItem>,
    /// The amounts in the currency that the name trades in.
    pub aged: Aged,
    /// The amounts in the company's currency.
    pub home_aged: Aged,
}

/// An open item with its age.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AgedItem {
    /// The item.
    pub item: OpenItem,
    /// Its age in days. Below zero before it is due.
    pub age: i64,
}

/// Amounts by age.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Aged {
    /// An amount for each of the buckets.
    pub amounts: Vec<Money>,
    /// What they come to.
    pub total: Money,
}

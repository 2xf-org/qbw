//! Everything that is open on one side.

use std::collections::BTreeMap;
use std::slice;

use chrono::NaiveDate;

use crate::amount::Total;
use crate::as_of::AsOf;
use crate::ids::{CurrencyId, NameId};
use crate::left_out::LeftOut;
use crate::money::Money;
use crate::open_items::OpenItem;
use crate::side::Side;
use crate::totals::Totals;

/// Everything that is open on one side, ordered by date.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OpenItems {
    /// Receivable or payable.
    pub side: Side,
    /// The moment.
    pub as_of: AsOf,
    /// What is open.
    pub items: Vec<OpenItem>,
    /// What changes of exchange rates added to accounts in other currencies, in
    /// the company's currency. It belongs to no item, and nothing of it is open
    /// in the other currency.
    pub adjustments: Money,
    /// How much could not be read.
    pub left_out: LeftOut,
    /// How much could not be read that the amounts rest on, which the numbers of
    /// the items do not.
    pub(crate) amounts_left_out: LeftOut,
    /// The adjustments of each name.
    pub(super) adjusted: BTreeMap<Option<NameId>, Total>,
}

impl OpenItems {
    /// The items in order.
    pub fn iter(&self) -> slice::Iter<'_, OpenItem> {
        self.items.iter()
    }

    /// The items of one customer or vendor.
    pub fn of(&self, name: NameId) -> impl Iterator<Item = &OpenItem> {
        self.iter().filter(move |item| item.name == Some(name))
    }

    /// The items that are owed and were due before the day `date`.
    pub fn overdue(&self, date: NaiveDate) -> impl Iterator<Item = &OpenItem> {
        self.iter().filter(move |item| item.is_overdue(date))
    }

    /// What the items come to, for each of their currencies.
    #[must_use]
    pub fn totals(&self) -> Totals {
        totals(&self.items)
    }

    /// What the items come to in the company's currency, without
    /// [`Self::adjustments`].
    #[must_use]
    pub fn home_total(&self) -> Money {
        home_total(&self.items, self.adjustments.currency())
    }
}

impl<'items> IntoIterator for &'items OpenItems {
    type Item = &'items OpenItem;
    type IntoIter = slice::Iter<'items, OpenItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// What items come to, for each of their currencies.
pub(super) fn totals<'items>(items: impl IntoIterator<Item = &'items OpenItem>) -> Totals {
    let mut sums: BTreeMap<CurrencyId, Total> = BTreeMap::new();
    for item in items {
        *sums.entry(item.open.currency()).or_default() += item.sums.open;
    }
    let sums = sums.into_iter();
    Totals::of(sums.map(|(currency, total)| total.money(currency)))
}

/// What items come to in the currency of the books, which is `home`.
pub(super) fn home_total<'items>(
    items: impl IntoIterator<Item = &'items OpenItem>,
    home: CurrencyId,
) -> Money {
    let sums = items.into_iter().map(|item| item.sums.home_open);
    sums.sum::<Total>().money(home)
}

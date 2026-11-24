//! Adding up what is open by age.

use crate::aging::Aged;
use crate::amount::Total;
use crate::ids::CurrencyId;
use crate::open_items::OpenSums;

/// Sums of open items, one for each bucket.
pub(super) struct AgedSums {
    buckets: Vec<OpenSums>,
}

impl AgedSums {
    pub(super) fn new(buckets: usize) -> Self {
        Self {
            buckets: vec![OpenSums::default(); buckets],
        }
    }

    /// Adds an item to the bucket at `place`.
    pub(super) fn add(&mut self, place: usize, item: OpenSums) {
        if let Some(bucket) = self.buckets.get_mut(place) {
            bucket.open += item.open;
            bucket.home_open += item.home_open;
        }
    }

    /// The sums of items of one currency.
    pub(super) fn own(&self, currency: CurrencyId) -> Aged {
        aged(self.buckets.iter().map(|bucket| bucket.open), currency)
    }

    /// The sums in the company's currency.
    pub(super) fn home(&self, home: CurrencyId) -> Aged {
        aged(self.buckets.iter().map(|bucket| bucket.home_open), home)
    }
}

fn aged(sums: impl Iterator<Item = Total> + Clone, currency: CurrencyId) -> Aged {
    let total: Total = sums.clone().sum();
    Aged {
        amounts: sums.map(|sum| sum.money(currency)).collect(),
        total: total.money(currency),
    }
}

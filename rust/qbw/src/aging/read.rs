//! Aging what is open.

use std::collections::BTreeMap;
use std::mem;

use chrono::NaiveDate;

use crate::aging::sums::AgedSums;
use crate::aging::{AgedItem, Aging, AgingBy, AgingOptions, AgingRow, Bucket};
use crate::ids::{CurrencyId, NameId};
use crate::left_out::LeftOut;
use crate::open_items::{OpenItem, OpenItems};
use crate::settings::AgingBasis;

/// Ages what is open.
pub(crate) struct AgingReader<'company> {
    pub(crate) open: OpenItems,
    /// The day that ages are counted to.
    pub(crate) day: NaiveDate,
    pub(crate) options: &'company AgingOptions,
    /// What the company's preferences age from.
    pub(crate) preferred: AgingBasis,
    /// The place of a name in its list.
    pub(crate) place: &'company dyn Fn(NameId) -> Option<usize>,
    /// How much could not be read of the names of the side.
    pub(crate) names_left_out: LeftOut,
}

/// An item that is in a bucket.
struct Placed {
    item: OpenItem,
    age: i64,
    bucket: usize,
}

/// Where a row of a name stands. Names that are in no list come last.
type NamePlace = (bool, Option<usize>, Option<NameId>, CurrencyId);

impl AgingReader<'_> {
    pub(crate) fn read(mut self) -> Aging {
        let buckets = Bucket::all(self.options.buckets, self.options.days);
        let items = mem::take(&mut self.open.items).into_iter();
        let placed: Vec<Placed> = items.map(|item| self.placed(item, &buckets)).collect();
        let mut totals = AgedSums::new(buckets.len());
        for placed in &placed {
            totals.add(placed.bucket, placed.item.sums());
        }
        let rows = match self.options.by {
            AgingBy::Names => self.names(&placed, buckets.len()),
            AgingBy::Items => self.items(placed, buckets.len()),
        };
        let left_out = match self.options.by {
            AgingBy::Names => self.open.amounts_left_out.union(self.names_left_out),
            AgingBy::Items => self.open.left_out,
        };
        Aging {
            side: self.open.side,
            as_of: self.options.as_of,
            day: self.day,
            basis: self.basis(),
            buckets,
            rows,
            totals: totals.home(self.home()),
            adjustments: self.open.adjustments,
            left_out,
        }
    }

    /// The currency that the books are kept in.
    fn home(&self) -> CurrencyId {
        self.open.adjustments.currency()
    }

    fn basis(&self) -> AgingBasis {
        self.options.basis.unwrap_or(self.preferred)
    }

    fn placed(&self, item: OpenItem, buckets: &[Bucket]) -> Placed {
        let since = self.since(&item);
        let age = self.day.signed_duration_since(since).num_days();
        let bucket = buckets.iter().position(|bucket| bucket.contains(age));
        Placed {
            item,
            age,
            // Every age is in a bucket.
            bucket: bucket.unwrap_or_default(),
        }
    }

    /// The date that the age of an item is counted from. What takes off what is
    /// owed, such as a credit, is never due.
    fn since(&self, item: &OpenItem) -> NaiveDate {
        match self.basis() {
            AgingBasis::DueDate if !item.open.is_negative() => item.due_date.unwrap_or(item.date),
            AgingBasis::DueDate | AgingBasis::TransactionDate => item.date,
        }
    }

    fn names(&self, placed: &[Placed], buckets: usize) -> Vec<AgingRow> {
        let mut sums: BTreeMap<NamePlace, AgedSums> = BTreeMap::new();
        for Placed { item, bucket, .. } in placed {
            let place = item.name.and_then(self.place);
            let place = (place.is_none(), place, item.name, item.open.currency());
            let sums = sums.entry(place).or_insert_with(|| AgedSums::new(buckets));
            sums.add(*bucket, item.sums());
        }
        let rows = sums
            .into_iter()
            .map(|((.., name, currency), sums)| AgingRow {
                name,
                item: None,
                aged: sums.own(currency),
                home_aged: sums.home(self.home()),
            });
        rows.collect()
    }

    fn items(&self, mut placed: Vec<Placed>, buckets: usize) -> Vec<AgingRow> {
        placed.sort_by_key(|placed| (placed.bucket, self.since(&placed.item)));
        let rows = placed.into_iter().map(|Placed { item, age, bucket }| {
            let mut sums = AgedSums::new(buckets);
            sums.add(bucket, item.sums());
            AgingRow {
                name: item.name,
                aged: sums.own(item.open.currency()),
                home_aged: sums.home(self.home()),
                item: Some(AgedItem { item, age }),
            }
        });
        rows.collect()
    }
}

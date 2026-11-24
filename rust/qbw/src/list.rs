//! Lists, in the order QuickBooks shows them.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::mem;

use crate::case;
use crate::damage::Damage;
use crate::ids::{
    AccountId, CategoryId, ClassId, CurrencyId, CustomerTypeId, ItemId, JobTypeId, MemorizedId,
    MessageId, NameId, PaymentMethodId, PriceLevelId, SalesRepId, SalesTaxCodeId, ShipMethodId,
    SiteId, TemplateId, TermsId, VendorTypeId,
};
use crate::left_out::LeftOut;
use crate::origin::Origin;

pub(crate) mod order;

pub(crate) mod sealed {
    /// Keeps [`ListEntry`](super::ListEntry) to the records of this crate, and
    /// [`ListId`](super::ListId) to their ids.
    pub trait Sealed {}
}

/// QuickBooks nests entries five deep. What goes on past this is a ring, or no
/// list of QuickBooks'.
pub(crate) const DEEPEST: usize = 8;

/// What an entry of a list is known by, such as an
/// [`AccountId`](crate::AccountId).
pub trait ListId: Copy + Eq + Hash + sealed::Sealed {
    /// The number that the file stores.
    fn number(self) -> i32;
}

macro_rules! list_ids {
    ($($id:ident,)+) => {$(
        impl sealed::Sealed for $id {}

        impl ListId for $id {
            fn number(self) -> i32 {
                self.number()
            }
        }
    )+};
}

list_ids! {
    AccountId,
    CategoryId,
    ClassId,
    CurrencyId,
    CustomerTypeId,
    ItemId,
    JobTypeId,
    MemorizedId,
    MessageId,
    NameId,
    PaymentMethodId,
    PriceLevelId,
    SalesRepId,
    SalesTaxCodeId,
    ShipMethodId,
    SiteId,
    TemplateId,
    TermsId,
    VendorTypeId,
}

/// An entry of a list, such as an account.
pub trait ListEntry: sealed::Sealed {
    /// What the entry is known by.
    type Id: ListId;

    /// What the entry is known by.
    fn id(&self) -> Self::Id;

    /// The entry's own name, without those of the entries above it.
    fn name(&self) -> &str;

    /// The entry above it, for a nested one.
    fn parent(&self) -> Option<Self::Id> {
        None
    }

    /// Whether the entry can be used in new transactions. An inactive one
    /// keeps its transactions and its place in reports.
    fn is_active(&self) -> bool;

    /// The stored row.
    fn origin(&self) -> Origin;
}

/// A list of entries in QuickBooks' own order: the one its windows start with
/// and its reports follow.
///
/// Deleted entries are not in it. Nothing in the books refers to one.
#[derive(Clone, Debug)]
pub struct List<T: ListEntry> {
    entries: Vec<T>,
    places: HashMap<T::Id, usize>,
    full_names: Vec<String>,
    damage: Vec<Damage>,
}

impl<T: ListEntry> List<T> {
    /// The list of `entries`, which are in order already, without what
    /// `damage` says could not be read. Of entries with one id the first is in
    /// it, and the others are damage.
    pub(crate) fn new(mut entries: Vec<T>, mut damage: Vec<Damage>) -> Self {
        let mut places = HashMap::new();
        entries.retain(|entry| {
            let is_first = !places.contains_key(&entry.id());
            if is_first {
                places.insert(entry.id(), places.len());
            } else {
                damage.push(Damage::repeated(entry.origin()));
            }
            is_first
        });
        let mut list = Self {
            entries,
            places,
            full_names: Vec::new(),
            damage,
        };
        list.full_names = list.entries.iter().map(|entry| list.path(entry)).collect();
        list
    }

    /// The entry with `id`.
    #[must_use]
    pub fn get(&self, id: T::Id) -> Option<&T> {
        self.entries.get(*self.places.get(&id)?)
    }

    /// The entry called `name`, ignoring case: by its full name, or else by its
    /// own, where no other entry has that. [`Self::all_named`] tells a name that
    /// several entries have from one that none has.
    #[must_use]
    pub fn named(&self, name: &str) -> Option<&T> {
        let name = case::folded(name);
        self.with_full_name(&name).next().or_else(|| {
            let mut entries = self.with_own_name(&name);
            let found = entries.next();
            found.filter(|_| entries.next().is_none())
        })
    }

    /// Every entry that `name` can mean, ignoring case: the ones with that full
    /// name, or else the ones whose own name it is, in the list's order.
    #[must_use]
    pub fn all_named(&self, name: &str) -> Vec<&T> {
        let name = case::folded(name);
        let mut found: Vec<&T> = self.with_full_name(&name).collect();
        if found.is_empty() {
            found.extend(self.with_own_name(&name));
        }
        found
    }

    /// The name of the entry with `id` after those of the entries above it, as
    /// QuickBooks writes it: `Parent:Child`.
    #[must_use]
    pub fn full_name(&self, id: T::Id) -> Option<&str> {
        let place = *self.places.get(&id)?;
        self.full_names.get(place).map(String::as_str)
    }

    /// The name of the entry with `id` itself, without those of the entries above
    /// it: what a line shows that is below its parent already.
    #[must_use]
    pub fn own_name(&self, id: T::Id) -> Option<&str> {
        self.get(id).map(ListEntry::name)
    }

    /// The entries in order, inactive ones among them.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.entries.iter()
    }

    /// The entries that can be used in new transactions, in order.
    pub fn active(&self) -> impl Iterator<Item = &T> {
        self.iter().filter(|entry| entry.is_active())
    }

    /// The entries right below `parent` in order, or the ones at the top.
    pub fn children(&self, parent: Option<T::Id>) -> impl Iterator<Item = &T> {
        self.iter().filter(move |entry| entry.parent() == parent)
    }

    /// The list as a tree: every entry with how many entries it is below, each
    /// before the ones right below it, which are in the list's order. An entry
    /// that no entry at the top leads to comes last, as one at the top.
    pub fn walk(&self) -> impl Iterator<Item = (usize, &T)> {
        let mut below: HashMap<T::Id, Vec<usize>> = HashMap::new();
        let mut tops = Vec::new();
        for (place, entry) in self.entries.iter().enumerate() {
            let parent = entry.parent().filter(|&parent| self.get(parent).is_some());
            match parent {
                Some(parent) => below.entry(parent).or_default().push(place),
                None => tops.push(place),
            }
        }
        let mut walked = Vec::with_capacity(self.entries.len());
        let mut seen = vec![false; self.entries.len()];
        // Those in a ring are reached by nothing but themselves.
        for top in tops.into_iter().chain(0..self.entries.len()) {
            let mut waiting = vec![(0, top)];
            while let Some((depth, place)) = waiting.pop() {
                let (Some(entry), Some(is_seen)) = (self.entries.get(place), seen.get_mut(place))
                else {
                    continue;
                };
                if mem::replace(is_seen, true) {
                    continue;
                }
                walked.push((depth, entry));
                let children = below.get(&entry.id()).into_iter().flatten().rev();
                waiting.extend(children.map(|&child| (depth + 1, child)));
            }
        }
        walked.into_iter()
    }

    /// The number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether there are no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// How much could not be read.
    #[must_use]
    pub fn left_out(&self) -> LeftOut {
        LeftOut::of(&self.damage)
    }

    /// What could not be read.
    #[must_use]
    pub fn damage(&self) -> &[Damage] {
        &self.damage
    }

    /// The entries with a full name, which is given without its case.
    fn with_full_name(&self, name: &str) -> impl Iterator<Item = &T> {
        let entries = self.entries.iter().zip(&self.full_names);
        let entries = entries.filter(move |(_, full_name)| case::folded(full_name) == name);
        entries.map(|(entry, _)| entry)
    }

    /// The entries with a name of their own, which is given without its case.
    fn with_own_name(&self, name: &str) -> impl Iterator<Item = &T> {
        let entries = self.entries.iter();
        entries.filter(move |entry| case::folded(entry.name()) == name)
    }

    /// The entry with `id` and the ones above it, from it up, each once. Of an
    /// entry that the list does not have, only the id.
    pub(crate) fn lineage(&self, id: T::Id) -> Vec<T::Id> {
        let mut lineage = vec![id];
        let mut above = self.get(id).and_then(ListEntry::parent);
        while let Some(entry) = above.and_then(|id| self.get(id))
            && lineage.len() < DEEPEST
            && !lineage.contains(&entry.id())
        {
            lineage.push(entry.id());
            above = entry.parent();
        }
        lineage
    }

    /// The entry with `id` and all of those below it, each once.
    pub(crate) fn subtree(&self, id: T::Id) -> HashSet<T::Id> {
        let mut found = HashSet::from([id]);
        let mut waiting = vec![id];
        while let Some(parent) = waiting.pop() {
            let below = self.children(Some(parent)).map(ListEntry::id);
            waiting.extend(below.filter(|&id| found.insert(id)));
        }
        found
    }

    /// The full name of `entry`, read up from it.
    fn path(&self, entry: &T) -> String {
        let lineage = self.lineage(entry.id()).into_iter().rev();
        let names = lineage.filter_map(|id| self.get(id)).map(ListEntry::name);
        let names: Vec<&str> = names.collect();
        names.join(":")
    }
}

impl<'list, T: ListEntry> IntoIterator for &'list List<T> {
    type Item = &'list T;
    type IntoIter = std::slice::Iter<'list, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use sa17::RowId;

    use super::*;

    struct Plant {
        id: u8,
        name: &'static str,
        parent: Option<u8>,
        is_active: bool,
    }

    impl sealed::Sealed for u8 {}

    impl ListId for u8 {
        fn number(self) -> i32 {
            self.into()
        }
    }

    impl sealed::Sealed for Plant {}

    impl ListEntry for Plant {
        type Id = u8;

        fn id(&self) -> u8 {
            self.id
        }

        fn name(&self) -> &str {
            self.name
        }

        fn parent(&self) -> Option<u8> {
            self.parent
        }

        fn is_active(&self) -> bool {
            self.is_active
        }

        fn origin(&self) -> Origin {
            Origin::new("plants", RowId::Added(self.id.into()))
        }
    }

    fn plant(id: u8, name: &'static str, parent: Option<u8>) -> Plant {
        Plant {
            id,
            name,
            parent,
            is_active: id != 9,
        }
    }

    fn list(plants: Vec<Plant>) -> List<Plant> {
        List::new(plants, Vec::new())
    }

    fn names<'list>(plants: impl Iterator<Item = &'list Plant>) -> Vec<&'list str> {
        plants.map(|plant| plant.name).collect()
    }

    #[test]
    fn entries_keep_their_order_and_nest() {
        let plants = list(vec![
            plant(3, "Fern", None),
            plant(1, "Royal", Some(3)),
            plant(9, "Lost", Some(3)),
            plant(2, "Moss", None),
        ]);
        assert_eq!(names(plants.iter()), ["Fern", "Royal", "Lost", "Moss"]);
        assert_eq!(names(plants.active()), ["Fern", "Royal", "Moss"]);
        assert_eq!(names(plants.children(None)), ["Fern", "Moss"]);
        assert_eq!(names(plants.children(Some(3))), ["Royal", "Lost"]);
        assert_eq!(plants.full_name(1), Some("Fern:Royal"));
        assert_eq!((plants.get(2).unwrap().name, plants.len()), ("Moss", 4));
        assert!(plants.get(7).is_none() && plants.full_name(7).is_none());
        assert!(plants.left_out().is_nothing() && !plants.is_empty());
    }

    #[test]
    fn a_full_name_is_found_before_a_name() {
        let plants = list(vec![
            plant(1, "Moss", Some(2)),
            plant(2, "Fern", None),
            plant(3, "Moss", None),
        ]);
        assert_eq!(plants.named("moss").unwrap().id, 3);
        assert_eq!(plants.named("Fern:MOSS").unwrap().id, 1);
        let nested = list(vec![plant(1, "Moss", Some(2)), plant(2, "Fern", None)]);
        assert_eq!(nested.named("moss").unwrap().id, 1);
        assert!(nested.named("Pine").is_none());
    }

    #[test]
    fn a_ring_of_parents_ends() {
        let plants = list(vec![plant(1, "Moss", Some(2)), plant(2, "Fern", Some(1))]);
        assert_eq!(plants.full_name(1), Some("Fern:Moss"));
        assert_eq!(plants.lineage(1), [1, 2]);
        assert_eq!(plants.subtree(1), HashSet::from([1, 2]));
        let lost_parent = list(vec![plant(1, "Moss", Some(5))]);
        assert_eq!(lost_parent.full_name(1), Some("Moss"));
        assert_eq!(lost_parent.lineage(7), [7]);
    }

    #[test]
    fn an_id_that_comes_twice_is_in_the_list_once() {
        let twice = list(vec![
            plant(3, "Fern", None),
            plant(1, "Moss", None),
            plant(3, "Ivy", None),
        ]);
        assert_eq!(names(twice.iter()), ["Fern", "Moss"]);
        assert_eq!(twice.get(3).map(ListEntry::name), Some("Fern"));
        let repeated = twice.damage().iter().map(|damage| damage.to_string());
        let expected = format!(
            "table plants, {}: a row before it has the same key",
            RowId::Added(3)
        );
        assert_eq!(repeated.collect::<Vec<_>>(), [expected]);
    }

    #[test]
    fn a_name_finds_an_entry_when_it_is_of_one() {
        let plants = list(vec![
            plant(1, "Fern", None),
            plant(2, "Royal", Some(1)),
            plant(3, "Moss", None),
            plant(4, "Royal", Some(3)),
            plant(5, "Caf\u{e9}", None),
        ]);
        let named = |name| plants.named(name).map(|plant| plant.id);
        assert_eq!(named("moss:royal"), Some(4));
        assert_eq!(named("ROYAL"), None);
        assert_eq!(named("fern"), Some(1));
        assert_eq!(named("CAF\u{c9}"), Some(5));
    }

    #[test]
    fn a_name_of_several_entries_has_them_all() {
        let plants = list(vec![
            plant(1, "Royal", Some(2)),
            plant(2, "Fern", None),
            plant(3, "Moss", None),
            plant(4, "Royal", Some(3)),
            plant(5, "Ivy", None),
            plant(6, "Ivy", None),
            plant(7, "Ivy", Some(3)),
        ]);
        let all_named = |name| -> Vec<u8> {
            let plants = plants.all_named(name).into_iter();
            plants.map(|plant| plant.id).collect()
        };
        assert_eq!(all_named("ROYAL"), [1, 4]);
        assert_eq!(all_named("moss:royal"), [4]);
        // The third is called something else in full.
        assert_eq!(all_named("ivy"), [5, 6]);
        assert_eq!(plants.named("ivy").unwrap().id, 5);
        assert!(all_named("Pine").is_empty());
    }

    #[test]
    fn a_list_is_walked_as_a_tree() {
        let walked = |plants: Vec<Plant>| -> Vec<(usize, u8)> {
            let plants = list(plants);
            let walk = plants.walk().map(|(depth, plant)| (depth, plant.id));
            walk.collect()
        };
        let plants = vec![
            plant(1, "Fern", None),
            plant(2, "Moss", None),
            plant(3, "Royal", Some(1)),
            plant(4, "Dwarf", Some(3)),
            plant(5, "Tall", Some(1)),
            // Its parent is in no list.
            plant(6, "Lost", Some(40)),
        ];
        let expected = [(0, 1), (1, 3), (2, 4), (1, 5), (0, 2), (0, 6)];
        assert_eq!(walked(plants), expected);
        let ring = vec![plant(1, "Fern", Some(2)), plant(2, "Moss", Some(1))];
        assert_eq!(walked(ring), [(0, 1), (1, 2)]);
    }
}

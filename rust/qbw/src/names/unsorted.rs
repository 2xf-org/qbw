//! A list of names before it is in order.

use crate::damage::Damage;
use crate::ids::NameId;
use crate::list::{List, ListEntry, order};

/// The entries of one of the four lists, before they are put in order. The
/// order goes through all four.
pub(super) struct Unsorted<T> {
    pub(super) entries: Vec<T>,
    /// The number of each name and of the one after it, deleted names among
    /// them.
    pub(super) links: Vec<(i32, i32)>,
    pub(super) damage: Vec<Damage>,
}

impl<T: ListEntry<Id = NameId>> Unsorted<T> {
    /// The list, in the order that `links` of the four lists give.
    pub(super) fn sorted(mut self, links: &[(i32, i32)]) -> List<T> {
        let links = links.iter().copied();
        order::sort(&mut self.entries, links, |entry| entry.id().number());
        List::new(self.entries, self.damage)
    }
}

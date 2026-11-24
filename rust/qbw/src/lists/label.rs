//! What entries are sorted by.

use crate::list::{ListEntry, ListId, sealed};
use crate::origin::Origin;

/// What entries or lines are sorted by, such as a class or a type of
/// customers. The lists of them differ in their ids only.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Label<I> {
    /// What the label is known by.
    pub id: I,
    /// Its own name, without those of the labels above it.
    pub name: String,
    /// The label it is under.
    pub parent: Option<I>,
    /// Whether it can be used in new entries and transactions.
    pub is_active: bool,
    /// The stored row.
    pub origin: Origin,
}

impl<I> sealed::Sealed for Label<I> {}

impl<I: ListId> ListEntry for Label<I> {
    type Id = I;

    fn id(&self) -> I {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn parent(&self) -> Option<I> {
        self.parent
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn origin(&self) -> Origin {
        self.origin
    }
}

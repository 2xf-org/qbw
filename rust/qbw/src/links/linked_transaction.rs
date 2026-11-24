//! The other end of a link.

use chrono::NaiveDate;

use crate::codes::TransactionKind;
use crate::ids::{NameId, TransactionId};
use crate::origin::Origin;
use crate::transactions::Header;

/// A transaction that another is linked to.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct LinkedTransaction {
    /// What the transaction is known by.
    pub id: TransactionId,
    /// Its kind.
    pub kind: TransactionKind,
    /// Its date.
    pub date: NaiveDate,
    /// Its number, as the form shows it.
    pub number: Option<String>,
    /// Who it is with.
    pub name: Option<NameId>,
    /// The stored row of its header.
    pub origin: Origin,
}

impl LinkedTransaction {
    pub(super) fn new(header: &Header) -> Self {
        Self {
            id: header.id,
            kind: header.kind,
            date: header.date,
            number: header.number(),
            name: header.name(),
            origin: header.origin(),
        }
    }
}

//! Sales reps.

use sa17::Source;

use crate::error::Result;
use crate::ids::{NameId, SalesRepId};
use crate::list::{List, ListEntry, sealed};
use crate::lists::reading;
use crate::origin::Origin;
use crate::tables::lists::SALES_REPS;

/// Someone who makes sales: an employee, a vendor or another name.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SalesRep {
    /// What the rep is known by.
    pub id: SalesRepId,
    /// Who it is.
    pub person: Option<NameId>,
    /// The initials that forms show, which the list calls the rep by.
    pub initials: String,
    /// Whether the rep can be used in new transactions.
    pub is_active: bool,
    /// The stored row.
    pub origin: Origin,
}

impl sealed::Sealed for SalesRep {}

impl ListEntry for SalesRep {
    type Id = SalesRepId;

    fn id(&self) -> SalesRepId {
        self.id
    }

    fn name(&self) -> &str {
        &self.initials
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn origin(&self) -> Origin {
        self.origin
    }
}

/// Reads the sales reps.
///
/// # Errors
///
/// Returns an error when their table lacks a column.
pub(crate) fn read(source: &dyn Source) -> Result<List<SalesRep>> {
    reading::by_name(source, &SALES_REPS, |origin, row| {
        (!row.is_deleted).then(|| SalesRep {
            id: row.id,
            person: row.person,
            initials: row.initials.unwrap_or_default(),
            is_active: !row.is_hidden,
            origin,
        })
    })
}

//! Sites and bins.

use sa17::Source;

use crate::error::Result;
use crate::ids::SiteId;
use crate::list::{List, ListEntry, sealed};
use crate::lists::reading;
use crate::lists::reading::Place;
use crate::origin::Origin;
use crate::tables::lists::{SITES, SiteRow};

/// A site that stock is kept at, or a bin of one.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Site {
    /// What the site is known by.
    pub id: SiteId,
    /// Its own name, without the site's that a bin is at.
    pub name: String,
    /// The site that a bin is at.
    pub parent: Option<SiteId>,
    /// What it is.
    pub description: Option<String>,
    /// The number of what QuickBooks uses it for, when it made the site
    /// itself. What the numbers stand for is not known.
    pub special: Option<i32>,
    /// Whether it can be used in new transactions.
    pub is_active: bool,
    /// The stored row.
    pub origin: Origin,
}

impl sealed::Sealed for Site {}

impl ListEntry for Site {
    type Id = SiteId;

    fn id(&self) -> SiteId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn parent(&self) -> Option<SiteId> {
        self.parent
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn origin(&self) -> Origin {
        self.origin
    }
}

/// Reads the sites.
///
/// # Errors
///
/// Returns an error when their table lacks a column.
pub(crate) fn read(source: &dyn Source) -> Result<List<Site>> {
    let place = |row: &SiteRow| Place::of(row.id.number(), row.next);
    reading::in_stored_order(source, &SITES, place, |origin, row| {
        (!row.is_deleted).then(|| Site {
            id: row.id,
            name: row.name.unwrap_or_default(),
            parent: row.parent,
            description: row.description,
            special: row.special.filter(|&special| special != 0),
            is_active: !row.is_hidden,
            origin,
        })
    })
}

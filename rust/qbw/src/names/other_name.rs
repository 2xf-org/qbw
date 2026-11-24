//! Another name, and the reading of the list of them.

use std::collections::HashMap;

use chrono::NaiveDate;
use sa17::Source;

use crate::damage::Damage;
use crate::error::Result;
use crate::ids::{NameId, OwnerKey};
use crate::list::{ListEntry, sealed};
use crate::names::details::DetailsByName;
use crate::names::unsorted::Unsorted;
use crate::names::{Channel, Contact};
use crate::origin::Origin;
use crate::tables::names::{OTHER_NAME_TOTALS, OTHER_NAMES, OtherNameRow, OtherNameTotalsRow};
use crate::unix_time::UnixTime;

/// A name that is neither a customer nor a vendor nor an employee.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OtherName {
    /// What the name is known by.
    pub id: NameId,
    /// The name.
    pub name: String,
    /// The contact that is in use.
    pub contact: Option<Contact>,
    /// Its phone numbers, e-mail addresses and the like, in order.
    pub channels: Vec<Channel>,
    /// Whether it can be used in new transactions.
    pub is_active: bool,
    /// Since when it is in use.
    pub opened: Option<NaiveDate>,
    /// What its custom fields are kept by.
    pub owner_key: Option<OwnerKey>,
    /// What QuickBooks calls it towards other programs.
    pub list_id: Option<String>,
    /// When it was made.
    pub created: Option<UnixTime>,
    /// When it was last changed.
    pub modified: Option<UnixTime>,
    /// The stored row.
    pub origin: Origin,
}

impl sealed::Sealed for OtherName {}

impl ListEntry for OtherName {
    type Id = NameId;

    fn id(&self) -> NameId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn origin(&self) -> Origin {
        self.origin
    }
}

/// Reads the other names.
///
/// # Errors
///
/// Returns an error when the file lacks a table or a column of it.
pub(super) fn read(
    source: &dyn Source,
    details: &mut DetailsByName,
) -> Result<Unsorted<OtherName>> {
    let mut damage: Vec<Damage> = Vec::new();
    let mut totals: HashMap<NameId, OtherNameTotalsRow> = OTHER_NAME_TOTALS
        .open(source)?
        .scan(&mut damage)
        .map(|(_, totals)| (totals.id, totals))
        .collect();
    let links = totals.values();
    let links = links.filter_map(|totals| Some((totals.id.number(), totals.next?)));
    let links = links.collect();
    let entries = OTHER_NAMES
        .open(source)?
        .scan(&mut damage)
        .filter(|(_, row)| !row.is_deleted)
        .map(|(origin, row)| {
            let totals = totals.remove(&row.id);
            other_name(origin, row, totals, details)
        })
        .collect();
    Ok(Unsorted {
        entries,
        links,
        damage,
    })
}

fn other_name(
    origin: Origin,
    row: OtherNameRow,
    totals: Option<OtherNameTotalsRow>,
    details: &mut DetailsByName,
) -> OtherName {
    let details = details.take(row.id);
    OtherName {
        id: row.id,
        name: row.name.unwrap_or_default(),
        contact: details.contact,
        channels: details.channels,
        is_active: !row.is_hidden,
        opened: totals.as_ref().and_then(|totals| totals.opened),
        owner_key: row.owner_key,
        list_id: totals.and_then(|totals| totals.list_id),
        created: row.created,
        modified: row.modified,
        origin,
    }
}

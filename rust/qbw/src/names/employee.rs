//! An employee, and the reading of the list of them.

use chrono::NaiveDate;
use sa17::Source;

use crate::error::Result;
use crate::ids::{NameId, OwnerKey};
use crate::list::{ListEntry, sealed};
use crate::names::details::DetailsByName;
use crate::names::unsorted::Unsorted;
use crate::names::{Channel, Contact};
use crate::origin::Origin;
use crate::tables::names::{EMPLOYEES, EmployeeRow};
use crate::unix_time::UnixTime;

/// An employee, as far as a name and a contact go.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Employee {
    /// What the employee is known by.
    pub id: NameId,
    /// The name.
    pub name: String,
    /// The day of hire, which is inferred from the column's name.
    pub hired: Option<NaiveDate>,
    /// The name that checks are made out to.
    pub name_on_check: Option<String>,
    /// The contact that is in use.
    pub contact: Option<Contact>,
    /// Phone numbers, e-mail addresses and the like, in order.
    pub channels: Vec<Channel>,
    /// Whether the name can be used in new transactions.
    pub is_active: bool,
    /// What the custom fields are kept by.
    pub owner_key: Option<OwnerKey>,
    /// What QuickBooks calls the employee towards other programs.
    pub list_id: Option<String>,
    /// When the entry was made.
    pub created: Option<UnixTime>,
    /// When it was last changed.
    pub modified: Option<UnixTime>,
    /// The stored row.
    pub origin: Origin,
}

impl sealed::Sealed for Employee {}

impl ListEntry for Employee {
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

/// Reads the employees.
///
/// # Errors
///
/// Returns an error when the file lacks a table or a column of it.
pub(super) fn read(source: &dyn Source, details: &mut DetailsByName) -> Result<Unsorted<Employee>> {
    let mut damage = Vec::new();
    let rows: Vec<(Origin, EmployeeRow)> = EMPLOYEES.open(source)?.scan(&mut damage).collect();
    let links = rows.iter();
    let links = links.filter_map(|(_, row)| Some((row.id.number(), row.next?)));
    let links = links.collect();
    let entries = rows
        .into_iter()
        .filter(|(_, row)| !row.is_deleted)
        .map(|(origin, row)| employee(origin, row, details))
        .collect();
    Ok(Unsorted {
        entries,
        links,
        damage,
    })
}

fn employee(origin: Origin, row: EmployeeRow, details: &mut DetailsByName) -> Employee {
    let details = details.take(row.id);
    Employee {
        id: row.id,
        name: row.name.unwrap_or_default(),
        hired: row.hired,
        name_on_check: row.name_on_check,
        contact: details.contact,
        channels: details.channels,
        is_active: !row.is_hidden,
        owner_key: row.owner_key,
        list_id: row.list_id,
        created: row.created,
        modified: row.modified,
        origin,
    }
}

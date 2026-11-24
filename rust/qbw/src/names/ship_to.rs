//! An address that goods are shipped to, and the reading of one.

use chrono::NaiveDate;
use sa17::{RowId, Source};

use crate::address::Address;
use crate::error::Result;
use crate::ids::{AddressId, NameId};
use crate::names::KeyIndex;
use crate::origin::Origin;
use crate::tables::contacts::{SHIP_TO, SHIP_TO_KEYS, ShipToRow};

/// An address that goods are shipped to: one of a name's list, or the one that
/// a form was made with.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ShipTo {
    /// What the address is known by.
    pub id: AddressId,
    /// The name it is of.
    pub name: Option<NameId>,
    /// What the name's list calls it. An address of a form has none.
    pub label: Option<String>,
    /// Since when it is out of the list, for one that only forms point at.
    pub expired: Option<NaiveDate>,
    /// The address.
    pub address: Address,
    /// The stored row.
    pub origin: Origin,
}

impl ShipTo {
    pub(super) fn new(origin: Origin, row: ShipToRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            label: row.label,
            expired: row.expired,
            address: Address {
                printed: Address::lines([
                    row.printed_1,
                    row.printed_2,
                    row.printed_3,
                    row.printed_4,
                    row.printed_5,
                ]),
                street: Address::lines([
                    row.street_1,
                    row.street_2,
                    row.street_3,
                    row.street_4,
                    row.street_5,
                ]),
                city: row.city,
                state: row.state,
                postal_code: row.postal_code,
                country: row.country,
            },
            origin,
        }
    }
}

/// Where every address is stored, those that only forms point at among them.
///
/// # Errors
///
/// Returns an error when the table of addresses lacks a column.
pub(crate) fn ship_to_index(source: &dyn Source) -> Result<KeyIndex<AddressId>> {
    KeyIndex::read(source, &SHIP_TO_KEYS, |row| row.id)
}

/// Reads the address that is stored in `row`, or `None` when there is none.
///
/// # Errors
///
/// Returns an error when the row cannot be read.
pub(crate) fn ship_to(source: &dyn Source, row: RowId) -> Result<Option<ShipTo>> {
    let Some(table) = SHIP_TO.find(source)? else {
        return Ok(None);
    };
    let read = table.fetch(row)?;
    let origin = Origin::new(SHIP_TO.name(), row);
    Ok(read.map(|read| ShipTo::new(origin, read)))
}

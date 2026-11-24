//! Which QuickBooks a file is of.

use std::collections::HashMap;

use sa17::Source;

use crate::damage::Damage;
use crate::error::Result;
use crate::tables::information::{SYSTEM_VALUES, system_value};

/// The year that the releases are counted from.
const FIRST_YEAR: i32 = 1990;

/// The QuickBooks that a file is of.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ProductVersion {
    /// The year in the product's name, such as 2024.
    pub year: Option<i32>,
    /// The number of the update to it.
    pub update: Option<i32>,
    /// The code of the edition, as the file keeps it.
    pub edition: Option<String>,
    /// The newest version that has opened the file, as the file keeps it.
    pub newest_used: Option<String>,
}

/// Reads the version. What the file does not say is `None`.
///
/// # Errors
///
/// Returns an error when the table of it lacks a column.
pub(super) fn read(source: &dyn Source, damage: &mut Vec<Damage>) -> Result<ProductVersion> {
    let Some(table) = SYSTEM_VALUES.find(source)? else {
        return Ok(ProductVersion::default());
    };
    let mut values: HashMap<String, String> = table
        .scan(damage)
        .map(|(_, row)| (row.name, row.value))
        .collect();
    let mut text = |name| values.remove(name).filter(|value| !value.is_empty());
    let release = text(system_value::RELEASE).and_then(|release| release.parse().ok());
    let update = text(system_value::UPDATE).and_then(|update| update.parse().ok());
    Ok(ProductVersion {
        year: release.and_then(|release: i32| release.checked_add(FIRST_YEAR)),
        update,
        edition: text(system_value::EDITION),
        newest_used: text(system_value::NEWEST_VERSION),
    })
}

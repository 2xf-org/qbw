//! Every custom field, found by the record that has it.

use sa17::Source;

use crate::custom_fields::{CustomField, CustomFieldDefinition};
use crate::damage::Damage;
use crate::error::Result;
use crate::ids::OwnerKey;
use crate::left_out::LeftOut;
use crate::tables::custom_fields::{CUSTOM_FIELD_DEFINITIONS, CUSTOM_FIELDS};

/// The fields that the company defined, and what its records have in them.
///
/// A record has its fields under a key, such as
/// [`Customer::owner_key`](crate::names::Customer::owner_key).
#[derive(Clone, Debug)]
pub struct CustomFields {
    definitions: Vec<CustomFieldDefinition>,
    /// Ordered by owner and position.
    values: Vec<CustomField>,
    damage: Vec<Damage>,
}

impl CustomFields {
    /// The fields that the company defined, ordered by position.
    #[must_use]
    pub fn definitions(&self) -> &[CustomFieldDefinition] {
        &self.definitions
    }

    /// What the record that `owner` stands for has in its fields, ordered by
    /// position. A field it has nothing in is left out.
    #[must_use]
    pub fn of(&self, owner: OwnerKey) -> &[CustomField] {
        let first = self.values.partition_point(|value| value.owner < owner);
        let rest = self.values.get(first..).unwrap_or_default();
        let own = rest.partition_point(|value| value.owner == owner);
        rest.get(..own).unwrap_or_default()
    }

    /// What the record that `owner` stands for has in the field that
    /// `definition` defines. The field has to be one of its kind of record's.
    #[must_use]
    pub fn value(
        &self,
        owner: OwnerKey,
        definition: &CustomFieldDefinition,
    ) -> Option<&CustomField> {
        let position = definition.position_in_records();
        let mut values = self.of(owner).iter();
        values.find(|value| value.position == position)
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
}

/// Reads the custom fields, of which a file may have none.
///
/// # Errors
///
/// Returns an error when a table of them lacks a column.
pub(crate) fn read(source: &dyn Source) -> Result<CustomFields> {
    let mut damage = Vec::new();
    let mut definitions: Vec<CustomFieldDefinition> = match CUSTOM_FIELD_DEFINITIONS.find(source)? {
        Some(table) => table
            .scan(&mut damage)
            .map(|(origin, row)| CustomFieldDefinition::new(origin, row))
            .collect(),
        None => Vec::new(),
    };
    definitions.sort_by_key(|definition| definition.position);
    let mut values = values(source, &mut damage)?;
    values.sort_unstable_by_key(|value| (value.owner, value.position));
    Ok(CustomFields {
        definitions,
        values,
        damage,
    })
}

fn values(source: &dyn Source, damage: &mut Vec<Damage>) -> Result<Vec<CustomField>> {
    let Some(table) = CUSTOM_FIELDS.find(source)? else {
        return Ok(Vec::new());
    };
    let values = table.scan(damage).filter_map(|(origin, row)| {
        Some(CustomField {
            owner: row.owner,
            position: row.position,
            text: row.text?,
            origin,
        })
    });
    Ok(values.collect())
}

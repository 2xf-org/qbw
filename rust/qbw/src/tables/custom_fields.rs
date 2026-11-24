//! The fields that a company defined.

use crate::ids::{CustomFieldId, OwnerKey};
#[cfg(test)]
use crate::tables::tests::Declared;
use crate::tables::{Table, stored_row};

pub(crate) const CUSTOM_FIELD_DEFINITIONS: Table<CustomFieldDefinitionRow> =
    Table::new("abmc_custom_field_type");

pub(crate) const CUSTOM_FIELDS: Table<CustomFieldRow> = Table::new("abmc_custom_field");

stored_row! {
    /// A field that the company defined.
    pub(crate) struct CustomFieldDefinitionRow {
        required {
            id: CustomFieldId = "custom_field_type_id",
            name: Option<String> = "custom_field_name",
            /// From 0 on for names, and from 30 on for items.
            position: i32 = "custom_field_position",
        }
        optional {
            is_for_customers: bool = "is_for_customer_bool",
            is_for_vendors: bool = "is_for_vendor_bool",
            is_for_employees: bool = "is_for_employee_bool",
            is_for_items: bool = "is_for_item_bool",
        }
    }
}

stored_row! {
    /// What a record has in such a field.
    pub(crate) struct CustomFieldRow {
        required {
            owner: OwnerKey = "abm_id",
            /// From 0 on, also for items.
            position: i32 = "position",
            text: Option<String> = "field",
        }
        optional {}
    }
}

#[cfg(test)]
pub(super) fn declared() -> Vec<Box<dyn Declared>> {
    vec![Box::new(CUSTOM_FIELD_DEFINITIONS), Box::new(CUSTOM_FIELDS)]
}

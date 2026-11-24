//! Fields that the company defined, and what its records have in them.

mod custom_field;
mod custom_field_definition;
mod directory;

pub use custom_field::CustomField;
pub use custom_field_definition::CustomFieldDefinition;
pub use directory::CustomFields;

pub(crate) use directory::read;

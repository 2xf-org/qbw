//! The fields that a company defined.

use sa17::Source;

use crate::company::Company;
use crate::custom_fields;
use crate::custom_fields::CustomFields;
use crate::error::Result;

/// # Custom fields
impl<S: Source> Company<S> {
    /// The fields that the company defined, and what its names, items,
    /// transactions and lines have in them.
    ///
    /// # Errors
    ///
    /// Returns an error when a table of them lacks a column.
    pub fn custom_fields(&self) -> Result<&CustomFields> {
        let pieces = self.pieces();
        pieces
            .custom_fields
            .get_or_build(|| custom_fields::read(self.source()))
    }
}

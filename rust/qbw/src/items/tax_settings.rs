//! Whose tax a sales tax is.

use crate::ids::NameId;

/// Whose tax a sales tax item charges. Its rate is the price of the item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TaxSettings {
    /// The vendor that the tax is paid to.
    pub agency: Option<NameId>,
}

//! One preference, as the file keeps it.

use std::fmt;

use crate::origin::Origin;

/// A preference of the whole company, as the file keeps it.
///
/// Some preferences hold secrets, and the names of some hold a person's e-mail
/// address, so neither is in its `Debug` output.
#[derive(Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Preference {
    /// The name that QuickBooks gives it, in capitals.
    pub name: String,
    /// Its value.
    pub value: String,
    /// The stored value.
    pub origin: Origin,
}

impl fmt::Debug for Preference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Preference")
            .field("origin", &self.origin)
            .finish_non_exhaustive()
    }
}

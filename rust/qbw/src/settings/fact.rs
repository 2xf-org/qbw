//! A figure that QuickBooks keeps about the company.

use std::fmt;

use crate::origin::Origin;

/// A figure that QuickBooks worked out for itself when it was last asked for
/// it, so it can be out of date.
///
/// Some hold an e-mail address, so the value is not in the `Debug` output.
#[derive(Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Fact {
    /// What QuickBooks calls it.
    pub key: String,
    /// Its value.
    pub value: String,
    /// The stored row.
    pub origin: Origin,
}

impl fmt::Debug for Fact {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Fact")
            .field("key", &self.key)
            .field("origin", &self.origin)
            .finish_non_exhaustive()
    }
}

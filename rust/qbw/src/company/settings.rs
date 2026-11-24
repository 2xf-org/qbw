//! How a company keeps its books.

use sa17::Source;

use crate::company::Company;
use crate::error::Result;
use crate::settings;
use crate::settings::Preferences;

/// # Preferences
impl<S: Source> Company<S> {
    /// The preferences of the whole company, such as its home currency.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Damaged`](crate::Error::Damaged) when a preference
    /// that every figure depends on, such as the home currency, cannot be
    /// understood, or may be one that cannot be read.
    pub fn preferences(&self) -> Result<&Preferences> {
        let pieces = self.pieces();
        pieces
            .preferences
            .get_or_build(|| settings::read(self.source()))
    }
}

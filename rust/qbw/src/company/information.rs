//! Who a company is, what QuickBooks keeps about it, and who works on it.

use sa17::Source;

use crate::company::Company;
use crate::error::Result;
use crate::settings;
use crate::settings::{CompanyInfo, Facts, Users};

/// # What the file says about the company
impl<S: Source> Company<S> {
    /// Who the company is, and which QuickBooks its file is of.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::preferences`], and an error when a table
    /// lacks a column.
    pub fn info(&self) -> Result<&CompanyInfo> {
        let pieces = self.pieces();
        pieces.information.get_or_build(|| {
            let preferences = self.preferences()?;
            settings::information(self.source(), preferences)
        })
    }

    /// The figures that QuickBooks keeps about the company for itself.
    ///
    /// # Errors
    ///
    /// Returns an error when the table of them lacks a column.
    pub fn facts(&self) -> Result<&Facts> {
        let pieces = self.pieces();
        pieces.facts.get_or_build(|| settings::facts(self.source()))
    }

    /// The people who work on the books in QuickBooks, as the audit trail
    /// names them.
    ///
    /// # Errors
    ///
    /// Returns an error when the table of the audit trail lacks a column.
    pub fn users(&self) -> Result<&Users> {
        let pieces = self.pieces();
        pieces.users.get_or_build(|| settings::users(self.source()))
    }
}

//! The ledger of a company.

use sa17::Source;

use crate::company::Company;
use crate::error::Result;
use crate::ledger;
use crate::ledger::Ledger;

/// # The books
impl<S: Source> Company<S> {
    /// Every posting of the books. The first call reads every line of every
    /// transaction, and the ledger is kept.
    ///
    /// # Errors
    ///
    /// Returns an error when the file lacks a table or a column of it, or when
    /// the company's currency cannot be understood.
    pub fn ledger(&self) -> Result<&Ledger> {
        let pieces = self.pieces();
        pieces.ledger.get_or_build(|| {
            let home = self.preferences()?.home_currency;
            ledger::build(self.source(), self.accounts()?, home)
        })
    }
}

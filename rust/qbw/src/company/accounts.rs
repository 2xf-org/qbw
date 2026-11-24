//! The accounts of a company.

use sa17::Source;

use crate::accounts;
use crate::accounts::Account;
use crate::company::Company;
use crate::error::Result;
use crate::list::List;

/// # Accounts
impl<S: Source> Company<S> {
    /// The chart of accounts, inactive accounts among them.
    ///
    /// # Errors
    ///
    /// Returns an error when the file lacks a table or a column of it.
    pub fn accounts(&self) -> Result<&List<Account>> {
        let pieces = self.pieces();
        pieces.accounts.get_or_build(|| {
            let home = self.preferences()?.home_currency;
            accounts::read(self.source(), home)
        })
    }
}

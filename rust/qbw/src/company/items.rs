//! The items of a company.

use sa17::Source;

use crate::company::Company;
use crate::error::Result;
use crate::items;
use crate::items::Item;
use crate::list::List;

/// # Items
impl<S: Source> Company<S> {
    /// The items, inactive ones among them. Their prices and costs are in the
    /// company's currency.
    ///
    /// # Errors
    ///
    /// Returns an error when the file lacks a table or a column of it.
    pub fn items(&self) -> Result<&List<Item>> {
        let pieces = self.pieces();
        pieces.items.get_or_build(|| {
            let home = self.preferences()?.home_currency;
            items::read(self.source(), home)
        })
    }
}

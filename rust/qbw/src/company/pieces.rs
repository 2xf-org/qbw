//! What is worked out from the file once and kept.

use std::sync::OnceLock;

use crate::accounts::Account;
use crate::company::lists::Lists;
use crate::company::per_form::PerForm;
use crate::custom_fields::CustomFields;
use crate::error::Result;
use crate::ids::{AddressId, ContactId};
use crate::inventory::{History, ItemLines};
use crate::items::Item;
use crate::ledger::Ledger;
use crate::links::{LinkDates, LinkIndex};
use crate::list::List;
use crate::names::{KeyIndex, Names};
use crate::settings::{CompanyInfo, Facts, Preferences, Users};
use crate::tables::OpenedTables;
use crate::transactions::{Headers, LineIndex, OwnLines};

/// Everything that a company has worked out from its file so far.
///
/// To add a piece: give it a field here, and beside this file a method of
/// [`Company`](crate::Company) that fills it, as in `settings.rs`. A method
/// asks for the pieces it needs through the other methods, so that each is
/// worked out when it is first needed. Editing the file forgets them all.
#[derive(Debug, Default)]
pub(super) struct Pieces {
    pub(super) accounts: Piece<List<Account>>,
    pub(super) contacts: Piece<KeyIndex<ContactId>>,
    pub(super) custom_fields: Piece<CustomFields>,
    pub(super) facts: Piece<Facts>,
    pub(super) headers: Piece<Headers>,
    pub(super) information: Piece<CompanyInfo>,
    pub(super) item_lines: Piece<ItemLines>,
    pub(super) items: Piece<List<Item>>,
    pub(super) ledger: Piece<Ledger>,
    pub(super) line_indexes: PerForm<LineIndex>,
    pub(super) link_dates: Piece<LinkDates>,
    pub(super) links: Piece<LinkIndex>,
    pub(super) lists: Lists,
    pub(super) names: Piece<Names>,
    pub(super) opened_tables: OpenedTables,
    pub(super) order_lines: PerForm<ItemLines>,
    pub(super) own_lines: PerForm<OwnLines>,
    pub(super) preferences: Piece<Preferences>,
    pub(super) ship_to_addresses: Piece<KeyIndex<AddressId>>,
    pub(super) stock_history: Piece<History>,
    pub(super) users: Piece<Users>,
}

/// One thing that is worked out from the file once.
#[derive(Debug)]
pub(super) struct Piece<T>(OnceLock<T>);

impl<T> Piece<T> {
    /// The piece, if it was worked out.
    pub(super) fn get(&self) -> Option<&T> {
        self.0.get()
    }

    /// The piece, worked out by `build` unless it was before.
    ///
    /// # Errors
    ///
    /// Returns the error of `build`, and then keeps nothing.
    pub(super) fn get_or_build(&self, build: impl FnOnce() -> Result<T>) -> Result<&T> {
        match self.0.get() {
            Some(piece) => Ok(piece),
            None => {
                let built = build()?;
                Ok(self.0.get_or_init(|| built))
            }
        }
    }
}

impl<T> Default for Piece<T> {
    fn default() -> Self {
        Self(OnceLock::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[test]
    fn a_piece_is_built_once_and_an_error_is_not_kept() {
        let piece: Piece<u32> = Piece::default();
        let missing = piece.get_or_build(|| Err(Error::NotACompany { table: "accounts" }));
        assert!(missing.is_err());
        assert_eq!(*piece.get_or_build(|| Ok(7)).unwrap(), 7);
        assert_eq!(*piece.get_or_build(|| Ok(8)).unwrap(), 7);
    }
}

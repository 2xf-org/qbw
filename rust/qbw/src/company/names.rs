//! The names a company deals with, and how to reach them.

use sa17::Source;

use crate::company::Company;
use crate::error::Result;
use crate::ids::{AddressId, ContactId};
use crate::names;
use crate::names::{Contact, Name, Names, ShipTo};

/// # Names
impl<S: Source> Company<S> {
    /// The customers and jobs, the vendors, the employees and the other names,
    /// inactive ones among them, with how to reach them.
    ///
    /// # Errors
    ///
    /// Returns an error when the file lacks a table or a column of it.
    pub fn names(&self) -> Result<&Names> {
        let pieces = self.pieces();
        pieces.names.get_or_build(|| {
            let home = self.preferences()?.home_currency;
            names::read(self.source(), home)
        })
    }

    /// The contact with `id`, as it was when a form used it. `None` when the
    /// file has no such contact.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Damaged`](crate::Error::Damaged) when the contact
    /// cannot be read, or may be one that cannot, and the errors of
    /// [`Self::names`].
    pub fn contact(&self, id: ContactId) -> Result<Option<Contact>> {
        let contacts = &self.pieces().contacts;
        let contacts = contacts.get_or_build(|| names::contact_index(self.source()))?;
        let Some(row) = contacts.get(id)? else {
            return Ok(None);
        };
        let contact = names::contact(self.source(), row)?;
        let name = contact.as_ref().and_then(|contact| contact.name);
        let in_use = match name {
            Some(name) => self.names()?.get(name).and_then(Name::contact),
            None => None,
        };
        Ok(in_use.filter(|in_use| in_use.id == id).cloned().or(contact))
    }

    /// The address with `id` that goods were shipped to, as it was when a form
    /// used it. `None` when the file has no such address.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Damaged`](crate::Error::Damaged) when the address
    /// cannot be read, or may be one that cannot.
    pub fn ship_to(&self, id: AddressId) -> Result<Option<ShipTo>> {
        let addresses = &self.pieces().ship_to_addresses;
        let addresses = addresses.get_or_build(|| names::ship_to_index(self.source()))?;
        match addresses.get(id)? {
            Some(row) => names::ship_to(self.source(), row),
            None => Ok(None),
        }
    }
}

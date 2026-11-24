//! What the file keeps apart from a name: how to reach it.

use std::collections::HashMap;

use sa17::Source;

use crate::damage::Damage;
use crate::error::Result;
use crate::ids::NameId;
use crate::names::{Channel, Contact, ShipTo, channel};
use crate::tables::contacts::{CONTACTS, SHIP_TO};

/// How to reach a name.
#[derive(Debug, Default)]
pub(super) struct Details {
    pub(super) contact: Option<Contact>,
    pub(super) additional_contacts: Vec<Contact>,
    pub(super) channels: Vec<Channel>,
    pub(super) ship_to_addresses: Vec<ShipTo>,
}

/// How to reach every name.
#[derive(Debug, Default)]
pub(super) struct DetailsByName(HashMap<NameId, Details>);

impl DetailsByName {
    /// Reads what is in use. What only old forms point at is left in the file.
    ///
    /// # Errors
    ///
    /// Returns an error when a table of it lacks a column.
    pub(super) fn read(source: &dyn Source, damage: &mut Vec<Damage>) -> Result<Self> {
        let mut details = Self::default();
        details.add_contacts(source, damage)?;
        details.add_ship_to_addresses(source, damage)?;
        for channel in channel::read(source, damage)? {
            details.of(channel.name).channels.push(channel);
        }
        for details in details.0.values_mut() {
            if let Some(contact) = &mut details.contact {
                contact.take_channels(&details.channels);
            }
        }
        Ok(details)
    }

    /// Gives up the details of one name.
    pub(super) fn take(&mut self, name: NameId) -> Details {
        self.0.remove(&name).unwrap_or_default()
    }

    fn of(&mut self, name: NameId) -> &mut Details {
        self.0.entry(name).or_default()
    }

    fn add_contacts(&mut self, source: &dyn Source, damage: &mut Vec<Damage>) -> Result<()> {
        let Some(table) = CONTACTS.find(source)? else {
            return Ok(());
        };
        let current = table.scan(damage).filter(|(_, row)| row.expired.is_none());
        for (origin, row) in current {
            if let Some(name) = row.name {
                let main = &mut self.of(name).contact;
                main.get_or_insert_with(|| Contact::new(origin, row));
            } else if let Some(name) = row.additional_of {
                let additional = &mut self.of(name).additional_contacts;
                additional.push(Contact::new(origin, row));
            }
        }
        for details in self.0.values_mut() {
            let additional = &mut details.additional_contacts;
            additional.sort_by_key(|contact| (contact.kind.code(), contact.id));
        }
        Ok(())
    }

    fn add_ship_to_addresses(
        &mut self,
        source: &dyn Source,
        damage: &mut Vec<Damage>,
    ) -> Result<()> {
        let Some(table) = SHIP_TO.find(source)? else {
            return Ok(());
        };
        let current = table.scan(damage).filter(|(_, row)| row.expired.is_none());
        for (origin, row) in current {
            if let Some(name) = row.name {
                let addresses = &mut self.of(name).ship_to_addresses;
                addresses.push(ShipTo::new(origin, row));
            }
        }
        for details in self.0.values_mut() {
            let addresses = &mut details.ship_to_addresses;
            addresses.sort_by(|left, right| (&left.label, left.id).cmp(&(&right.label, right.id)));
        }
        Ok(())
    }
}

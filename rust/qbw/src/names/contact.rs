//! A contact, and the reading of one.

use chrono::NaiveDate;
use sa17::{RowId, Source};

use crate::address::Address;
use crate::codes::{ChannelKind, ContactKind};
use crate::error::Result;
use crate::ids::{ContactId, NameId};
use crate::names::{Channel, KeyIndex};
use crate::origin::Origin;
use crate::tables::contacts::{CONTACT_KEYS, CONTACTS, ContactRow};

/// Who to ask for and where, as it was at some time.
///
/// Every name has one that is in use. When it changes, QuickBooks keeps the
/// one before, which forms of that time go on to point at.
///
/// QuickBooks keeps the phone numbers and e-mail addresses of a name in its
/// [channels](crate::names::Channel), and a copy with the contact that can be cut
/// off. The one that is in use has the channel's, where the name has one. An
/// earlier one has the copy, which is all that is left of it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Contact {
    /// What the contact is known by.
    pub id: ContactId,
    /// Which of the name's contacts it is.
    pub kind: ContactKind,
    /// The name it is of.
    pub name: Option<NameId>,
    /// When it was replaced by another, for one that is no longer in use.
    pub expired: Option<NaiveDate>,
    /// The name of the company.
    pub company: Option<String>,
    /// How to address the person, such as `Ms.`.
    pub salutation: Option<String>,
    /// The person's first name.
    pub first_name: Option<String>,
    /// The person's middle name or its initial.
    pub middle_name: Option<String>,
    /// The person's last name.
    pub last_name: Option<String>,
    /// What the person does there.
    pub job_title: Option<String>,
    /// Who to ask for.
    pub contact: Option<String>,
    /// Who else to ask for.
    pub alt_contact: Option<String>,
    /// The phone number.
    pub phone: Option<String>,
    /// Another phone number.
    pub alt_phone: Option<String>,
    /// The number of a mobile phone.
    pub mobile: Option<String>,
    /// The number of a fax.
    pub fax: Option<String>,
    /// The e-mail address.
    pub email: Option<String>,
    /// Who else gets e-mails.
    pub cc: Option<String>,
    /// The number that the name knows the company by, or the company it.
    pub account_number: Option<String>,
    /// The address: a customer's for invoices.
    pub address: Address,
    /// The stored row.
    pub origin: Origin,
}

impl Contact {
    pub(super) fn new(origin: Origin, row: ContactRow) -> Self {
        Self {
            id: row.id,
            kind: row.kind.unwrap_or_default(),
            name: row.name.or(row.additional_of),
            expired: row.expired,
            company: row.company,
            salutation: row.salutation,
            first_name: row.first_name,
            middle_name: row.middle_name,
            last_name: row.last_name,
            job_title: row.job_title,
            contact: row.contact,
            alt_contact: row.alt_contact,
            phone: row.phone,
            alt_phone: row.alt_phone,
            mobile: row.mobile,
            fax: row.fax,
            email: row.email,
            cc: row.cc,
            account_number: row.account_number,
            address: Address {
                printed: Address::lines([
                    row.printed_1,
                    row.printed_2,
                    row.printed_3,
                    row.printed_4,
                    row.printed_5,
                ]),
                street: Address::lines([
                    row.street_1,
                    row.street_2,
                    row.street_3,
                    row.street_4,
                    row.street_5,
                ]),
                city: row.city,
                state: row.state,
                postal_code: row.postal_code,
                country: row.country,
            },
            origin,
        }
    }
}

impl Contact {
    /// Takes the numbers and addresses of `channels`, the ones of its name.
    pub(super) fn take_channels(&mut self, channels: &[Channel]) {
        for channel in channels {
            let copy = match channel.kind {
                ChannelKind::MainPhone => &mut self.phone,
                ChannelKind::AltPhone => &mut self.alt_phone,
                ChannelKind::Mobile => &mut self.mobile,
                ChannelKind::Fax => &mut self.fax,
                ChannelKind::MainEmail => &mut self.email,
                ChannelKind::CcEmail => &mut self.cc,
                _ => continue,
            };
            *copy = Some(channel.value.clone());
        }
    }
}

/// Where every contact is stored, those that are no longer in use among them.
///
/// # Errors
///
/// Returns an error when the table of contacts lacks a column.
pub(crate) fn contact_index(source: &dyn Source) -> Result<KeyIndex<ContactId>> {
    KeyIndex::read(source, &CONTACT_KEYS, |row| row.id)
}

/// Reads the contact that is stored in `row`, or `None` when there is none.
///
/// # Errors
///
/// Returns an error when the row cannot be read.
pub(crate) fn contact(source: &dyn Source, row: RowId) -> Result<Option<Contact>> {
    let Some(table) = CONTACTS.find(source)? else {
        return Ok(None);
    };
    let read = table.fetch(row)?;
    let origin = Origin::new(CONTACTS.name(), row);
    Ok(read.map(|read| Contact::new(origin, read)))
}

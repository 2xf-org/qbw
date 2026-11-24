//! How a name is reached: its contacts, phone numbers and addresses.

use chrono::NaiveDate;

use crate::codes::{ChannelKind, ContactKind};
use crate::ids::{AddressId, ContactId, NameId};
#[cfg(test)]
use crate::tables::tests::Declared;
use crate::tables::{Table, stored_row};

pub(crate) const CONTACTS: Table<ContactRow> = Table::new("abmc_contacts");

pub(crate) const CONTACT_KEYS: Table<ContactKeyRow> = Table::new("abmc_contacts");

pub(crate) const CHANNELS: Table<ChannelRow> = Table::new("abmc_contacts_info");

pub(crate) const CHANNEL_LABELS: Table<ChannelLabelRow> = Table::new("abmc_contacts_channel");

pub(crate) const SHIP_TO: Table<ShipToRow> = Table::new("abmc_ship_to");

pub(crate) const SHIP_TO_KEYS: Table<ShipToKeyRow> = Table::new("abmc_ship_to");

stored_row! {
    /// A contact of a name.
    pub(crate) struct ContactRow {
        required {
            id: ContactId = "contacts_abm_id",
            /// Whose main contact it is. Nothing on an additional one.
            name: Option<NameId> = "name_id",
            /// Nothing on a contact that is still in use.
            expired: Option<NaiveDate> = "expired_date",
        }
        optional {
            kind: Option<ContactKind> = "additional_contacts_type",
            /// Whose additional contact it is.
            additional_of: Option<NameId> = "alternate_id",
            company: Option<String> = "company_name",
            salutation: Option<String> = "salutation",
            first_name: Option<String> = "first_name",
            middle_name: Option<String> = "middle_initial",
            last_name: Option<String> = "last_name",
            job_title: Option<String> = "title",
            contact: Option<String> = "contact1",
            alt_contact: Option<String> = "contact2",
            phone: Option<String> = "telephone_num1",
            alt_phone: Option<String> = "telephone_num2",
            mobile: Option<String> = "cellular_num",
            fax: Option<String> = "fax_num",
            email: Option<String> = "email",
            cc: Option<String> = "emailCC",
            account_number: Option<String> = "account_num",
            printed_1: Option<String> = "address_line1",
            printed_2: Option<String> = "address_line2",
            printed_3: Option<String> = "address_line3",
            printed_4: Option<String> = "address_line4",
            printed_5: Option<String> = "address_line5",
            street_1: Option<String> = "street_line1",
            street_2: Option<String> = "street_line2",
            street_3: Option<String> = "street_line3",
            street_4: Option<String> = "street_line4",
            street_5: Option<String> = "street_line5",
            city: Option<String> = "city",
            state: Option<String> = "state",
            postal_code: Option<String> = "postal_code",
            country: Option<String> = "country",
        }
    }
}

stored_row! {
    /// The key of a contact.
    pub(crate) struct ContactKeyRow {
        required {
            id: ContactId = "contacts_abm_id",
        }
        optional {}
    }
}

stored_row! {
    /// A slot for a phone number, an e-mail address or the like.
    pub(crate) struct ChannelRow {
        required {
            name: NameId = "name_id",
            kind: ChannelKind = "channel_category_id",
            /// Nothing in a slot that is empty.
            value: Option<String> = "field_value",
        }
        optional {
            position: Option<i32> = "position",
            is_deleted: bool = "is_deleted_bool",
        }
    }
}

stored_row! {
    /// What a slot is for, such as `Main Phone`.
    pub(crate) struct ChannelLabelRow {
        required {
            kind: ChannelKind = "channel_category_id",
            text: Option<String> = "channel_category_name",
        }
        optional {}
    }
}

stored_row! {
    /// An address that goods are shipped to.
    pub(crate) struct ShipToRow {
        required {
            id: AddressId = "ship_to_abm_id",
            name: Option<NameId> = "name_id",
            /// Nothing on an address that is still in the name's list.
            expired: Option<NaiveDate> = "expired_date",
        }
        optional {
            label: Option<String> = "name",
            printed_1: Option<String> = "address_line1",
            printed_2: Option<String> = "address_line2",
            printed_3: Option<String> = "address_line3",
            printed_4: Option<String> = "address_line4",
            printed_5: Option<String> = "address_line5",
            street_1: Option<String> = "street_line1",
            street_2: Option<String> = "street_line2",
            street_3: Option<String> = "street_line3",
            street_4: Option<String> = "street_line4",
            street_5: Option<String> = "street_line5",
            city: Option<String> = "city",
            state: Option<String> = "state",
            postal_code: Option<String> = "postal_code",
            country: Option<String> = "country",
        }
    }
}

stored_row! {
    /// The key of an address that goods are shipped to.
    pub(crate) struct ShipToKeyRow {
        required {
            id: AddressId = "ship_to_abm_id",
        }
        optional {}
    }
}

#[cfg(test)]
pub(super) fn declared() -> Vec<Box<dyn Declared>> {
    vec![
        Box::new(CONTACTS),
        Box::new(CONTACT_KEYS),
        Box::new(CHANNELS),
        Box::new(CHANNEL_LABELS),
        Box::new(SHIP_TO),
        Box::new(SHIP_TO_KEYS),
    ]
}

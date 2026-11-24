//! A vendor, and the reading of the list of them.

use std::collections::HashMap;

use chrono::NaiveDate;
use sa17::Source;

use crate::accounts::StoredBalance;
use crate::damage::Damage;
use crate::error::Result;
use crate::ids::{CurrencyId, NameId, OwnerKey, TermsId, VendorTypeId};
use crate::list::{ListEntry, sealed};
use crate::money::Money;
use crate::names::details::DetailsByName;
use crate::names::unsorted::Unsorted;
use crate::names::{Channel, Contact, ShipTo};
use crate::origin::Origin;
use crate::tables::names::{VENDOR_TOTALS, VENDORS, VendorRow, VendorTotalsRow};
use crate::unix_time::UnixTime;

/// A vendor.
///
/// Nothing on a vendor says that sales tax is paid to it. The sales tax items
/// do, with [`TaxSettings::agency`](crate::items::TaxSettings::agency).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Vendor {
    /// What the vendor is known by.
    pub id: NameId,
    /// Its name.
    pub name: String,
    /// Its type.
    pub vendor_type: Option<VendorTypeId>,
    /// When it is paid.
    pub terms: Option<TermsId>,
    /// The currency it sells in.
    pub currency: CurrencyId,
    /// How much the company may owe it.
    pub credit_limit: Option<Money>,
    /// The name that its checks are made out to.
    pub name_on_check: Option<String>,
    /// Whether what it is paid goes on a form 1099.
    pub is_1099: bool,
    /// The contact that is in use.
    pub contact: Option<Contact>,
    /// Further people to ask for.
    pub additional_contacts: Vec<Contact>,
    /// Its phone numbers, e-mail addresses and the like, in order.
    pub channels: Vec<Channel>,
    /// The addresses that goods it sells can go to, ordered by what they are
    /// called.
    pub ship_to_addresses: Vec<ShipTo>,
    /// Whether it can be used in new transactions.
    pub is_active: bool,
    /// When it became a vendor.
    pub opened: Option<NaiveDate>,
    /// What it is owed, as QuickBooks keeps it: below zero where the company
    /// owes it something.
    pub stored: Option<StoredBalance>,
    /// What its custom fields are kept by.
    pub owner_key: Option<OwnerKey>,
    /// What QuickBooks calls it towards other programs.
    pub list_id: Option<String>,
    /// When it was made.
    pub created: Option<UnixTime>,
    /// When it was last changed.
    pub modified: Option<UnixTime>,
    /// The stored row.
    pub origin: Origin,
}

impl sealed::Sealed for Vendor {}

impl ListEntry for Vendor {
    type Id = NameId;

    fn id(&self) -> NameId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn origin(&self) -> Origin {
        self.origin
    }
}

/// Reads the vendors of a company that keeps its books in `home`.
///
/// # Errors
///
/// Returns an error when the file lacks a table or a column of it.
pub(super) fn read(
    source: &dyn Source,
    home: CurrencyId,
    details: &mut DetailsByName,
) -> Result<Unsorted<Vendor>> {
    let mut damage: Vec<Damage> = Vec::new();
    let mut totals: HashMap<NameId, VendorTotalsRow> = VENDOR_TOTALS
        .open(source)?
        .scan(&mut damage)
        .map(|(_, totals)| (totals.id, totals))
        .collect();
    let links = totals.values();
    let links = links.filter_map(|totals| Some((totals.id.number(), totals.next?)));
    let links = links.collect();
    let entries = VENDORS
        .open(source)?
        .scan(&mut damage)
        .filter(|(_, row)| !row.is_deleted)
        .map(|(origin, row)| {
            let totals = totals.remove(&row.id);
            vendor(origin, row, totals, details, home)
        })
        .collect();
    Ok(Unsorted {
        entries,
        links,
        damage,
    })
}

fn vendor(
    origin: Origin,
    row: VendorRow,
    totals: Option<VendorTotalsRow>,
    details: &mut DetailsByName,
    home: CurrencyId,
) -> Vendor {
    let details = details.take(row.id);
    let currency = row.currency.unwrap_or(home);
    Vendor {
        id: row.id,
        name: row.name.unwrap_or_default(),
        vendor_type: row.vendor_type,
        terms: row.terms,
        currency,
        credit_limit: row.credit_limit.map(|limit| Money::new(limit, currency)),
        name_on_check: row.name_on_check,
        is_1099: row.is_1099,
        contact: details.contact,
        additional_contacts: details.additional_contacts,
        channels: details.channels,
        ship_to_addresses: details.ship_to_addresses,
        is_active: !row.is_hidden,
        opened: totals.as_ref().and_then(|totals| totals.opened),
        stored: totals.as_ref().and_then(|totals| {
            Some(StoredBalance {
                amount: Money::new(totals.balance?, currency),
                home_amount: totals.home_balance.map(|amount| Money::new(amount, home)),
            })
        }),
        owner_key: row.owner_key,
        list_id: totals.and_then(|totals| totals.list_id),
        created: row.created,
        modified: row.modified,
        origin,
    }
}

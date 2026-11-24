//! A customer or a job, and the reading of the list of them.

use std::collections::HashMap;

use chrono::NaiveDate;
use sa17::Source;

use crate::accounts::StoredBalance;
use crate::damage::Damage;
use crate::error::Result;
use crate::ids::{
    AddressId, ClassId, CurrencyId, CustomerTypeId, ItemId, NameId, OwnerKey, PaymentMethodId,
    PriceLevelId, SalesRepId, SalesTaxCodeId, TermsId,
};
use crate::list::{ListEntry, sealed};
use crate::money::Money;
use crate::names::details::DetailsByName;
use crate::names::unsorted::Unsorted;
use crate::names::{Channel, Contact, Job, ShipTo};
use crate::origin::Origin;
use crate::tables::names::{CUSTOMER_TOTALS, CUSTOMERS, CustomerRow, CustomerTotalsRow};
use crate::unix_time::UnixTime;

/// A customer, or a job of one.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Customer {
    /// What the customer is known by.
    pub id: NameId,
    /// Its own name, without the customer's that a job is of.
    pub name: String,
    /// The customer or the job that a job is of.
    pub parent: Option<NameId>,
    /// How far it is, for a job, and for a customer that is followed as one.
    pub job: Option<Job>,
    /// Its type.
    pub customer_type: Option<CustomerTypeId>,
    /// When it pays.
    pub terms: Option<TermsId>,
    /// Who sells to it.
    pub sales_rep: Option<SalesRepId>,
    /// The prices it pays.
    pub price_level: Option<PriceLevelId>,
    /// How it pays.
    pub payment_method: Option<PaymentMethodId>,
    /// Whether it is taxed.
    pub tax_code: Option<SalesTaxCodeId>,
    /// The sales tax it pays.
    pub tax_item: Option<ItemId>,
    /// The class of what it buys. Unchecked.
    pub class: Option<ClassId>,
    /// The currency it buys in.
    pub currency: CurrencyId,
    /// How much it may owe.
    pub credit_limit: Option<Money>,
    /// The number that it buys for resale under. Unchecked.
    pub resale_number: Option<String>,
    /// The contact that is in use, with the address for invoices.
    pub contact: Option<Contact>,
    /// Further people to ask for.
    pub additional_contacts: Vec<Contact>,
    /// Its phone numbers, e-mail addresses and the like, in order.
    pub channels: Vec<Channel>,
    /// Where its goods go unless a form says otherwise: one of
    /// `ship_to_addresses`.
    pub ship_to: Option<AddressId>,
    /// The addresses its goods can go to, ordered by what they are called.
    pub ship_to_addresses: Vec<ShipTo>,
    /// Whether it can be used in new transactions.
    pub is_active: bool,
    /// When it became a customer.
    pub opened: Option<NaiveDate>,
    /// What it owes, as QuickBooks keeps it.
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

impl sealed::Sealed for Customer {}

impl ListEntry for Customer {
    type Id = NameId;

    fn id(&self) -> NameId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn parent(&self) -> Option<NameId> {
        self.parent
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn origin(&self) -> Origin {
        self.origin
    }
}

/// Reads the customers and jobs of a company that keeps its books in `home`.
///
/// # Errors
///
/// Returns an error when the file lacks a table or a column of it.
pub(super) fn read(
    source: &dyn Source,
    home: CurrencyId,
    details: &mut DetailsByName,
) -> Result<Unsorted<Customer>> {
    let mut damage: Vec<Damage> = Vec::new();
    let mut totals: HashMap<NameId, CustomerTotalsRow> = CUSTOMER_TOTALS
        .open(source)?
        .scan(&mut damage)
        .map(|(_, totals)| (totals.id, totals))
        .collect();
    let links = totals.values();
    let links = links.filter_map(|totals| Some((totals.id.number(), totals.next?)));
    let links = links.collect();
    let entries = CUSTOMERS
        .open(source)?
        .scan(&mut damage)
        .filter(|(_, row)| !row.is_deleted)
        .map(|(origin, row)| {
            let totals = totals.remove(&row.id);
            customer(origin, row, totals, details, home)
        })
        .collect();
    Ok(Unsorted {
        entries,
        links,
        damage,
    })
}

fn customer(
    origin: Origin,
    row: CustomerRow,
    totals: Option<CustomerTotalsRow>,
    details: &mut DetailsByName,
    home: CurrencyId,
) -> Customer {
    let details = details.take(row.id);
    let currency = row.currency.unwrap_or(home);
    let job = Job {
        status: row.job_status.unwrap_or_default(),
        job_type: row.job_type,
        description: row.job_description,
        start: row.job_start,
        projected_end: row.job_projected_end,
        end: row.job_end,
    };
    let is_followed_as_job = row.parent.is_some() || job != Job::default();
    Customer {
        id: row.id,
        name: row.name.unwrap_or_default(),
        parent: row.parent,
        job: is_followed_as_job.then_some(job),
        customer_type: row.customer_type,
        terms: row.terms,
        sales_rep: row.sales_rep,
        price_level: row.price_level,
        payment_method: row.payment_method,
        tax_code: row.tax_code,
        tax_item: row.tax_item,
        class: row.class,
        currency,
        credit_limit: row.credit_limit.map(|limit| Money::new(limit, currency)),
        resale_number: row.resale_number,
        contact: details.contact,
        additional_contacts: details.additional_contacts,
        channels: details.channels,
        ship_to: row.ship_to,
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

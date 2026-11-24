//! Names, and how to reach them.

use std::collections::{HashMap, HashSet};

use qbw::names::Name;
use qbw::{AddressId, Company, ContactId, NameId};

use crate::agreement::Agreement;
use crate::{agreement, books, chain};

/// The tables of names, with the column of their keys.
const NAMES: [(&str, &str); 4] = [
    ("abmc_customer_user", "customer_id"),
    ("abmc_vendor_user", "vendor_id"),
    ("abmc_employee", "employee_id"),
    ("abmc_other_user", "other_id"),
];

/// The ids of a list, deleted names among them.
fn ids(company: &Company, (table, key): (&str, &str)) -> Vec<i32> {
    let rows = agreement::rows(company, table, &[key]);
    rows.iter().map(|row| row.read(0).unwrap()).collect()
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn names_read_and_lead_to_their_rows() {
    for company in books::books() {
        let names = company.names().unwrap();
        println!("{} names", names.len());
        assert!(names.left_out().is_nothing() && !names.is_empty());
        for name in names.iter() {
            assert!(!name.name().is_empty());
            let row = company.stored_row(name.origin()).unwrap().unwrap();
            assert_eq!(row.id(), name.origin().row);
        }
        let customers = names.customers();
        assert!(customers.iter().all(|customer| {
            let parent = customer.parent;
            parent.is_none_or(|parent| customers.get(parent).is_some())
        }));
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn the_lists_of_names_share_no_key() {
    for company in books::books() {
        let ids: Vec<i32> = NAMES
            .iter()
            .flat_map(|&table| ids(company, table))
            .collect();
        let distinct: HashSet<i32> = ids.iter().copied().collect();
        println!("{} names, {} keys", ids.len(), distinct.len());
        assert_eq!(distinct.len(), ids.len());
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn every_name_has_one_contact_in_use() {
    let columns = ["name_id", "expired_date", "additional_contacts_type"];
    for company in books::books() {
        let mut in_use: HashMap<i32, usize> = HashMap::new();
        for row in agreement::rows(company, "abmc_contacts", &columns) {
            let name: Option<i32> = row.read(0).unwrap();
            let is_in_use = row.value(1).unwrap().is_null();
            let kind: i32 = row.read(2).unwrap();
            if let (Some(name), true, 0) = (name, is_in_use, kind) {
                *in_use.entry(name).or_default() += 1;
            }
        }
        let mut contacts = Agreement::default();
        for table in NAMES {
            for name in ids(company, table) {
                contacts.add(in_use.get(&name) == Some(&1));
            }
        }
        contacts.check("names with one contact in use");
        let names = company.names().unwrap();
        assert!(names.iter().all(|name| name.contact().is_some()));
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn additional_contacts_are_with_their_names() {
    let columns = ["alternate_id", "expired_date"];
    for company in books::books() {
        let names = company.names().unwrap();
        let rows = agreement::rows(company, "abmc_contacts", &columns);
        let stored = rows.iter().filter(|row| {
            let of: Option<i32> = row.read(0).unwrap();
            let is_in_use = row.value(1).unwrap().is_null();
            is_in_use && of.is_some_and(|of| names.get(NameId::new(of)).is_some())
        });
        let customers = names.customers().iter();
        let customers: usize = customers.map(|name| name.additional_contacts.len()).sum();
        let vendors = names.vendors().iter();
        let vendors: usize = vendors.map(|name| name.additional_contacts.len()).sum();
        println!("{} additional contacts", customers + vendors);
        assert_eq!(customers + vendors, stored.count());
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn one_chain_goes_through_every_name_list_after_list() {
    let tables = [
        ("abmc_customer_internal", "customer_id"),
        ("abmc_vendor_internal", "vendor_id"),
        ("abmc_employee", "employee_id"),
        ("abmc_other_internal", "other_id"),
    ];
    for company in books::books() {
        let links: HashMap<i32, i32> = tables
            .iter()
            .flat_map(|(table, key)| chain::links(company, table, key))
            .collect();
        let chain = chain::chain(&links);
        println!("{} of {} names are in the chain", chain.len(), links.len());
        assert_eq!(chain.len(), links.len());
        // Without the deleted names it is the order of the directory.
        let names = company.names().unwrap();
        let chain = chain.into_iter().map(NameId::new);
        let chain: Vec<NameId> = chain.filter(|&id| names.get(id).is_some()).collect();
        let order: Vec<NameId> = names.iter().map(Name::id).collect();
        assert!(chain == order, "the list is in another order");
    }
}

/// The contacts and addresses that the headers of a form point at.
fn pointed_at(company: &Company, column: &str) -> HashSet<String> {
    let rows = agreement::rows(company, "abmc_invoice_header", &[column]);
    let keys = rows
        .iter()
        .map(|row| row.read::<Option<String>>(0).unwrap());
    keys.flatten().filter(|key| !key.is_empty()).collect()
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn every_invoice_has_the_contact_and_the_address_it_was_made_with() {
    for company in books::books() {
        let mut contacts = Agreement::default();
        for key in pointed_at(company, "contact_id") {
            let contact = company.contact(ContactId::try_new(&key).unwrap());
            let contact = contact.unwrap();
            contacts.add(contact.is_some_and(|contact| contact.name.is_some()));
        }
        contacts.check("contacts of invoices");
        let mut addresses = Agreement::default();
        for key in pointed_at(company, "ship_to_id") {
            let address = company.ship_to(AddressId::try_new(&key).unwrap());
            addresses.add(address.unwrap().is_some());
        }
        addresses.check("addresses of invoices");
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn an_address_of_a_list_has_a_label_and_the_default_is_one_of_them() {
    for company in books::books() {
        let mut labels = Agreement::default();
        let mut defaults = Agreement::default();
        for customer in company.names().unwrap().customers() {
            let addresses = &customer.ship_to_addresses;
            for address in addresses {
                labels.add(address.label.is_some());
            }
            if let Some(default) = customer.ship_to {
                defaults.add(addresses.iter().any(|address| address.id == default));
            }
        }
        labels.check("addresses with a label");
        defaults.check("customers whose address is in their list");
    }
}

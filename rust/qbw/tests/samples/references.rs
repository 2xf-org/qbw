//! What a record of one list points at in another.

use std::hash::Hash;

use qbw::{Company, List, ListEntry};

use crate::agreement::Agreement;
use crate::books;

/// Counts the references to entries of one list.
struct References<'list, T: ListEntry> {
    to: &'list List<T>,
    hits: Agreement,
}

impl<'list, T: ListEntry> References<'list, T>
where
    T::Id: Copy + Eq + Hash,
{
    fn to(to: &'list List<T>) -> Self {
        Self {
            to,
            hits: Agreement::default(),
        }
    }

    fn add(&mut self, id: Option<T::Id>) {
        if let Some(id) = id {
            self.hits.add(self.to.get(id).is_some());
        }
    }

    fn check(&self, what: &str) {
        self.hits.check(&format!("references to {what}"));
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn every_reference_hits_an_entry_that_is_not_deleted() {
    for company in books::books() {
        check_names(company);
        check_items(company);
    }
}

fn check_names(company: &Company) {
    let names = company.names().unwrap();
    let mut terms = References::to(company.terms().unwrap());
    let mut customer_types = References::to(company.customer_types().unwrap());
    let mut vendor_types = References::to(company.vendor_types().unwrap());
    let mut sales_reps = References::to(company.sales_reps().unwrap());
    let mut price_levels = References::to(company.price_levels().unwrap());
    let mut payment_methods = References::to(company.payment_methods().unwrap());
    let mut tax_codes = References::to(company.sales_tax_codes().unwrap());
    let mut tax_items = References::to(company.items().unwrap());
    let mut currencies = References::to(company.currencies().unwrap());
    let uses_currencies = company.preferences().unwrap().uses_multicurrency;
    for customer in names.customers() {
        terms.add(customer.terms);
        customer_types.add(customer.customer_type);
        sales_reps.add(customer.sales_rep);
        price_levels.add(customer.price_level);
        payment_methods.add(customer.payment_method);
        tax_codes.add(customer.tax_code);
        tax_items.add(customer.tax_item);
        currencies.add(uses_currencies.then_some(customer.currency));
    }
    for vendor in names.vendors() {
        terms.add(vendor.terms);
        vendor_types.add(vendor.vendor_type);
        currencies.add(uses_currencies.then_some(vendor.currency));
    }
    terms.check("terms");
    customer_types.check("customer types");
    vendor_types.check("vendor types");
    sales_reps.check("sales reps");
    price_levels.check("price levels");
    payment_methods.check("payment methods");
    tax_codes.check("tax codes of customers");
    tax_items.check("tax items");
    currencies.check("currencies");
    let mut people = Agreement::default();
    for rep in company.sales_reps().unwrap() {
        let person = rep.person.and_then(|person| names.get(person));
        people.add(person.is_some());
    }
    people.check("sales reps that are names");
}

fn check_items(company: &Company) {
    let items = company.items().unwrap();
    let mut accounts = References::to(company.accounts().unwrap());
    let mut vendors = References::to(company.names().unwrap().vendors());
    let mut agencies = References::to(company.names().unwrap().vendors());
    let mut tax_codes = References::to(company.sales_tax_codes().unwrap());
    let mut categories = References::to(company.categories().unwrap());
    let mut members = References::to(items);
    let mut sites = References::to(company.sites().unwrap());
    for item in items {
        accounts.add(item.sales.account);
        accounts.add(item.purchase.as_ref().and_then(|purchase| purchase.account));
        accounts.add(item.stock.as_ref().and_then(|stock| stock.asset_account));
        vendors.add(item.purchase.as_ref().and_then(|purchase| purchase.vendor));
        agencies.add(item.tax.and_then(|tax| tax.agency));
        tax_codes.add(item.tax_code);
        categories.add(item.category);
        for member in &item.members {
            members.add(Some(member.item));
        }
        for at in &item.sites {
            sites.add(at.site);
        }
    }
    for level in company.price_levels().unwrap() {
        for price in &level.items {
            members.add(Some(price.item));
        }
    }
    accounts.check("accounts of items");
    vendors.check("vendors of items");
    agencies.check("tax agencies");
    tax_codes.check("tax codes of items");
    categories.check("categories");
    members.check("items");
    sites.check("sites");
}

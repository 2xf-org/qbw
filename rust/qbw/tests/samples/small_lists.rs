//! The small lists, custom fields and what the file says about the company.

use qbw::chrono::NaiveDate;
use qbw::custom_fields::CustomFieldDefinition;
use qbw::lists::{TemplateForm, TermsKind};
use qbw::sa17::Source;
use qbw::{Company, List, ListEntry, Origin, OwnerKey, SiteId, TemplateId, TermsId};

use crate::agreement::Agreement;
use crate::share::Share;
use crate::{agreement, books, chain};

/// Checks that a list reads, that its entries have a name and a parent in the
/// list, and that they lead to their rows.
fn check<T: ListEntry>(
    company: &Company,
    what: &str,
    list: &List<T>,
    origin: impl Fn(&T) -> Origin,
) {
    println!("{} {what}", list.len());
    assert!(list.left_out().is_nothing());
    for entry in list {
        assert!(!entry.name().is_empty());
        let parent = entry.parent();
        assert!(parent.is_none_or(|parent| list.get(parent).is_some()));
        let row = company.stored_row(origin(entry)).unwrap().unwrap();
        assert_eq!(row.id(), origin(entry).row);
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn small_lists_read_and_lead_to_their_rows() {
    for company in books::books() {
        let list = company.categories().unwrap();
        check(company, "categories", list, |entry| entry.origin);
        let list = company.classes().unwrap();
        check(company, "classes", list, |entry| entry.origin);
        let list = company.currencies().unwrap();
        check(company, "currencies", list, |entry| entry.origin);
        let list = company.customer_messages().unwrap();
        check(company, "messages", list, |entry| entry.origin);
        let list = company.customer_types().unwrap();
        check(company, "customer types", list, |entry| entry.origin);
        let list = company.job_types().unwrap();
        check(company, "job types", list, |entry| entry.origin);
        let list = company.memorized().unwrap();
        check(company, "memorized transactions", list, |entry| {
            entry.origin
        });
        let list = company.payment_methods().unwrap();
        check(company, "payment methods", list, |entry| entry.origin);
        let list = company.price_levels().unwrap();
        check(company, "price levels", list, |entry| entry.origin);
        let list = company.sales_reps().unwrap();
        check(company, "sales reps", list, |entry| entry.origin);
        let list = company.sales_tax_codes().unwrap();
        check(company, "sales tax codes", list, |entry| entry.origin);
        let list = company.ship_methods().unwrap();
        check(company, "ways of shipping", list, |entry| entry.origin);
        let list = company.sites().unwrap();
        check(company, "sites", list, |entry| entry.origin);
        let list = company.templates().unwrap();
        check(company, "templates", list, |entry| entry.origin);
        let list = company.terms().unwrap();
        check(company, "terms", list, |entry| entry.origin);
        let list = company.vendor_types().unwrap();
        check(company, "vendor types", list, |entry| entry.origin);
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn the_order_of_a_list_covers_every_row() {
    let tables = [
        ("abmc_categories", "category_id"),
        ("abmc_class", "class_id"),
        ("abmc_customer_type", "customer_type_id"),
        ("abmc_job_type", "job_type_id"),
        ("abmc_memorized_transactions", "memorized_transactions_id"),
        ("abmc_site", "site_id"),
        ("abmc_vendor_type", "vendor_type_id"),
    ];
    for company in books::books() {
        let mut lists = Agreement::default();
        for (table, key) in tables {
            let links = chain::links(company, table, key);
            lists.add(chain::chain(&links).len() == links.len());
        }
        lists.check("lists whose order covers every row");
        let chain = chain::chain(&chain::links(company, "abmc_site", "site_id"));
        let chain = chain.into_iter().map(SiteId::new);
        let sites = company.sites().unwrap();
        let chain: Vec<SiteId> = chain.filter(|&id| sites.get(id).is_some()).collect();
        let order: Vec<SiteId> = sites.iter().map(|site| site.id).collect();
        assert!(chain == order, "the list is in another order");
    }
}

/// The symbols that are not ASCII, in every book.
const SYMBOLS: usize = 55;

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn every_symbol_that_is_not_ascii_reads_as_utf_8() {
    for company in books::books() {
        let encoding = company.source().text_encoding();
        let mut symbols = Agreement::default();
        let mut looked_at = 0;
        for currency in company.currencies().unwrap() {
            let row = company.stored_row(currency.origin).unwrap().unwrap();
            let stored = row.get("symbol").unwrap().as_str().unwrap_or_default();
            if !stored.is_ascii() {
                let bytes = encoding.encode(stored).unwrap();
                let symbol = currency.symbol.as_deref().unwrap();
                symbols.add(symbol.as_bytes() == bytes);
                looked_at += 1;
            }
        }
        symbols.check("symbols that read as UTF-8");
        assert_eq!(looked_at, SYMBOLS);
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn most_forms_are_due_when_their_terms_say() {
    let mut standard = Share::default();
    let mut date_driven = Share::default();
    for company in books::books() {
        let terms = company.terms().unwrap();
        for table in ["abmc_invoice_header", "abmc_bill_header"] {
            let columns = ["terms_id", "transaction_date", "due_date"];
            for row in agreement::rows(company, table, &columns) {
                let id: Option<i32> = row.read(0).unwrap();
                let Some(terms) = id.and_then(|id| terms.get(TermsId::new(id))) else {
                    continue;
                };
                let from: NaiveDate = row.read(1).unwrap();
                let due: Option<NaiveDate> = row.read(2).unwrap();
                match terms.kind {
                    TermsKind::Standard => standard.add(terms.due_date(from) == due),
                    _ => date_driven.add(terms.due_date(from) == due),
                }
            }
        }
    }
    standard.check("forms due by standard terms", 95);
    date_driven.check("forms due by date-driven terms", 65);
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn a_template_is_for_the_form_that_uses_it() {
    let forms = [
        ("abmc_credit_memo_header", TemplateForm::CreditMemo),
        ("abmc_estimate_header", TemplateForm::Estimate),
        ("abmc_invoice_header", TemplateForm::Invoice),
        ("abmc_item_assembly_header", TemplateForm::AssemblyBuild),
        ("abmc_purchase_order_header", TemplateForm::PurchaseOrder),
        ("abmc_sales_order_header", TemplateForm::SalesOrder),
    ];
    for company in books::books() {
        let templates = company.templates().unwrap();
        let mut agreement = Agreement::default();
        for (table, form) in forms {
            for row in agreement::rows(company, table, &["custom_style_id"]) {
                let id: Option<i32> = row.read(0).unwrap();
                if let Some(id) = id.filter(|&id| id != 0) {
                    let template = templates.get(TemplateId::new(id));
                    agreement.add(template.is_some_and(|template| template.form == form));
                }
            }
        }
        agreement.check("forms with a template of their own");
    }
}

/// Counts the values of a record that a definition of its kind names.
fn add_values(
    company: &Company,
    owner: Option<OwnerKey>,
    is_for: impl Fn(&CustomFieldDefinition) -> bool,
    values: &mut Agreement,
) {
    let Some(owner) = owner else {
        return;
    };
    let fields = company.custom_fields().unwrap();
    for value in fields.of(owner) {
        let mut definitions = fields.definitions().iter().filter(|field| is_for(field));
        values.add(definitions.any(|field| fields.value(owner, field) == Some(value)));
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn every_value_of_a_name_or_an_item_is_of_a_field_for_it() {
    for company in books::books() {
        let fields = company.custom_fields().unwrap();
        assert!(fields.left_out().is_nothing());
        let mut values = Agreement::default();
        let names = company.names().unwrap();
        for customer in names.customers() {
            let is_for = |field: &CustomFieldDefinition| field.is_for_customers;
            add_values(company, customer.owner_key, is_for, &mut values);
        }
        for vendor in names.vendors() {
            let is_for = |field: &CustomFieldDefinition| field.is_for_vendors;
            add_values(company, vendor.owner_key, is_for, &mut values);
        }
        for item in company.items().unwrap() {
            let is_for = |field: &CustomFieldDefinition| field.is_for_items;
            add_values(company, item.owner_key, is_for, &mut values);
        }
        values.check("values of a field for its record");
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn the_company_its_facts_and_its_users_read() {
    for company in books::books() {
        let info = company.info().unwrap();
        assert!(info.left_out.is_nothing() && !info.name.is_empty());
        assert!(info.legal_name.is_some() && !info.legal_address.is_empty());
        assert!(info.started.is_some() && info.created.is_some());
        assert!(info.version.year.is_some() && info.version.edition.is_some());
        let lines = info.address_lines.len();
        assert!((2..=3).contains(&lines));
        let facts = company.facts().unwrap();
        assert!(facts.left_out().is_nothing() && !facts.is_empty());
        let keys = facts.iter().map(|fact| facts.get(&fact.key));
        assert!(keys.eq(facts.iter().map(Some)));
        let users = company.users().unwrap();
        assert!(users.left_out().is_nothing() && !users.is_empty());
        println!("{} facts and {} users", facts.len(), users.len());
    }
}

//! Fields that the company defined.

use qbw::custom_fields::{CustomField, CustomFieldDefinition};
use qbw::transactions::TransactionKind;
use qbw::{ItemId, NameId, OwnerKey};

use crate::{small_lists, support};

#[test]
fn definitions_say_which_records_have_a_field() {
    let company = support::books();
    let fields = company.custom_fields().unwrap();
    let [region, color] = fields.definitions() else {
        panic!("two fields: {:?}", fields.definitions());
    };
    assert_eq!(region.name, "Region");
    assert!(region.is_for_customers && !region.is_for_vendors);
    assert!(!region.is_for_employees && !region.is_for_items);
    assert_eq!(region.position, 0);
    assert_eq!(color.name, "Color");
    assert!(color.is_for_items && !color.is_for_customers);
    assert_eq!(color.position, 30);
    assert!(fields.left_out().is_nothing());
}

fn texts(values: &[CustomField]) -> Vec<&str> {
    let texts = values.iter().map(|value| value.text.as_str());
    texts.collect()
}

#[test]
fn a_record_has_its_values_under_its_key() {
    let company = support::books();
    let fields = company.custom_fields().unwrap();
    let names = company.names().unwrap();
    let ada = names.get(NameId::new(101)).unwrap().owner_key().unwrap();
    assert_eq!(texts(fields.of(ada)), ["North"]);
    let items = company.items().unwrap();
    let valve = items.get(ItemId::new(502)).unwrap().owner_key.unwrap();
    assert_eq!(texts(fields.of(valve)), ["Gold"]);
    // A transaction and a line have theirs.
    let invoice = OwnerKey::try_new("TXN0000000000400").unwrap();
    assert_eq!(texts(fields.of(invoice)), ["North"]);
    let line = OwnerKey::try_new("LINE000000000402").unwrap();
    assert_eq!(texts(fields.of(line)), ["Gold"]);
    // A job without a value, and a vendor.
    let annex = names.get(NameId::new(103)).unwrap().owner_key().unwrap();
    assert!(fields.of(annex).is_empty());
    let cobalt = names.get(NameId::new(201)).unwrap().owner_key().unwrap();
    assert!(fields.of(cobalt).is_empty());
}

#[test]
fn a_value_is_found_by_the_definition_of_its_field() {
    let company = support::books();
    let fields = company.custom_fields().unwrap();
    let definition = |name: &str| -> &CustomFieldDefinition {
        let mut definitions = fields.definitions().iter();
        definitions.find(|field| field.name == name).unwrap()
    };
    let items = company.items().unwrap();
    let valve = items.get(ItemId::new(502)).unwrap().owner_key.unwrap();
    let color = fields.value(valve, definition("Color")).unwrap();
    assert_eq!((color.text.as_str(), color.owner), ("Gold", valve));
    let row = company.stored_row(color.origin).unwrap().unwrap();
    assert_eq!(row.get("field").unwrap().as_str(), Some("Gold"));
    let installation = items.get(ItemId::new(501)).unwrap().owner_key.unwrap();
    assert!(fields.value(installation, definition("Color")).is_none());
}

#[test]
fn definitions_say_which_transactions_and_lines_have_a_field() {
    let company = support::books();
    let definitions = company.custom_fields().unwrap().definitions();
    let [region, color] = definitions else {
        panic!("two fields: {definitions:?}");
    };
    let invoice = TransactionKind::Invoice;
    let order = TransactionKind::PurchaseOrder;
    assert!(region.is_for_transactions(invoice) && !region.is_for_lines(invoice));
    assert!(color.is_for_lines(invoice) && !color.is_for_transactions(invoice));
    // A purchase order has the fields of vendors, and no customer's.
    assert!(!region.is_for_transactions(order) && color.is_for_lines(order));
    for kind in [
        TransactionKind::CreditMemo,
        TransactionKind::Estimate,
        TransactionKind::SalesOrder,
        TransactionKind::SalesReceipt,
    ] {
        assert!(region.is_for_transactions(kind) && color.is_for_lines(kind));
    }
    for kind in [TransactionKind::Bill, TransactionKind::AssemblyBuild] {
        assert!(!region.is_for_transactions(kind) && color.is_for_lines(kind));
    }
    for kind in [TransactionKind::Check, TransactionKind::Payment] {
        assert!(!region.is_for_transactions(kind) && !color.is_for_lines(kind));
    }
    // What the books have is what the definitions say.
    let fields = company.custom_fields().unwrap();
    let line = OwnerKey::try_new("LINE000000000402").unwrap();
    let value = fields.value(line, color).unwrap();
    assert_eq!(value.text, "Gold");
}

#[test]
fn a_file_that_lacks_the_tables_has_no_fields() {
    let company = small_lists::company([]);
    let fields = company.custom_fields().unwrap();
    assert!(fields.definitions().is_empty() && fields.left_out().is_nothing());
    let owner = OwnerKey::try_new("NAME000000000101").unwrap();
    assert!(fields.of(owner).is_empty());
}

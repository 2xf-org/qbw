//! Opening a company, changing what is below it, and stepping down to it.

use std::cell::Cell;
use std::mem;
use std::sync::Arc;

use qbw::sa17::{Database, Source};
use qbw::{AsOf, Company, Error, NameId, TransactionId};
use qbw_synthetic::{SyntheticColumn, SyntheticTable, company};

use crate::{counted, support};

#[test]
fn a_database_of_something_else_is_not_a_company() {
    let error = Company::from_bytes(company::sample().build()).unwrap_err();
    assert!(matches!(
        error,
        Error::NotACompany {
            table: "abmc_account_user"
        }
    ));
    assert_eq!(
        error.to_string(),
        "the database is not a QuickBooks company file: it has no table abmc_account_user"
    );
}

#[test]
fn a_file_that_is_no_database_says_why() {
    let error = Company::from_bytes(vec![0; 4096]).unwrap_err();
    let Error::Database(below) = &error else {
        panic!("the file is no database: {error:?}");
    };
    assert_eq!(error.to_string(), below.to_string());
    let missing = Company::open("no such file.qbw").unwrap_err();
    assert!(matches!(missing, Error::Database(_)));
}

#[test]
fn a_table_without_a_column_that_every_release_has_is_named() {
    let totals = SyntheticTable::new("abmc_account_internal")
        .column(SyntheticColumn::new("account_id", 2))
        .column(SyntheticColumn::new("special_type", 1).width(2));
    let database = support::preferences(&[])
        .table(support::no_accounts())
        .table(totals);
    let company = support::company(database);
    let error = company.accounts().unwrap_err();
    assert!(matches!(
        error,
        Error::Schema {
            table: "abmc_account_internal",
            column: "display_index_next"
        }
    ));
    assert_eq!(
        error.to_string(),
        "the table abmc_account_internal has no column display_index_next"
    );
}

#[test]
fn a_company_is_read_from_anything_that_answers_in_a_file_s_place() {
    let database = Arc::new(Database::from_bytes(company::books().build()).unwrap());
    let borrowed = Company::try_new(&*database).unwrap();
    assert_eq!(borrowed.accounts().unwrap().len(), 22);
    let shared = Company::try_new(Arc::clone(&database)).unwrap();
    assert_eq!(shared.accounts().unwrap().len(), 22);
    let boxed: Box<dyn Source> = Box::new(database);
    let boxed = Company::try_new(boxed).unwrap();
    assert_eq!(boxed.accounts().unwrap().len(), 22);
    let source = boxed.into_source();
    assert!(source.definition("abmc_account_user").is_ok());
}

#[test]
fn an_edit_forgets_what_was_worked_out() {
    let mut company = support::books();
    assert_eq!(company.accounts().unwrap().len(), 22);
    let other = support::with_preferences(&[("FISCALBEGIN", "4")]).into_source();
    let before = company.edit(|source| mem::replace(source, other));
    assert_eq!(
        company
            .preferences()
            .unwrap()
            .fiscal_year_start
            .number_from_month(),
        4
    );
    assert!(company.accounts().is_err());
    company.edit(|source| *source = before);
    assert_eq!(company.accounts().unwrap().len(), 22);
}

#[test]
fn an_edit_that_fails_forgets_nothing() {
    let mut company = counted::books();
    let customer = NameId::new(101);
    let before = company
        .ledger()
        .unwrap()
        .customer_balance(customer, AsOf::Latest);
    let scanned = company.source().scanned();
    let failed = company.try_edit(|_| Err::<(), &str>("no such row"));
    assert_eq!(failed, Err("no such row"));
    let after = company
        .ledger()
        .unwrap()
        .customer_balance(customer, AsOf::Latest);
    assert_eq!((after, company.source().scanned()), (before, scanned));
    let edited = company.try_edit(|_| Ok::<i32, &str>(7));
    assert_eq!(edited, Ok(7));
    company.ledger().unwrap();
    assert!(company.source().scanned() > scanned);
}

#[test]
fn an_edit_that_leaves_the_revision_forgets_nothing() {
    let mut company = counted::books();
    company.ledger().unwrap();
    let scanned = company.source().scanned();
    let revision = Cell::new(1);
    // Nothing is changed, whether that is an error or not.
    for outcome in [Ok(7), Err("no such row")] {
        let edited = company.edit_by_revision(|_| revision.get(), |_| outcome);
        assert_eq!(edited, outcome);
        company.ledger().unwrap();
        assert_eq!(company.source().scanned(), scanned);
    }
    // Something is, before it fails.
    let failed = company.edit_by_revision(
        |_| revision.get(),
        |_| {
            revision.set(2);
            Err::<(), &str>("the second row is locked")
        },
    );
    assert!(failed.is_err());
    company.ledger().unwrap();
    assert!(company.source().scanned() > scanned);
}

#[test]
fn a_form_is_read_again_without_looking_for_its_tables() {
    let company = counted::books();
    let read = |id| company.transaction(TransactionId::new(id)).unwrap();
    read(400).unwrap();
    let named = company.source().named();
    let again = read(400).unwrap();
    let other = read(800).unwrap();
    assert_eq!(company.source().named(), named);
    // It is what a company that reads it the first time gives.
    let whole = support::books();
    assert_eq!(whole.transaction(again.id).unwrap(), Some(again));
    assert_eq!(whole.transaction(other.id).unwrap(), Some(other));
}

#[test]
fn every_record_leads_to_its_stored_row() {
    let company = support::books();
    for account in company.accounts().unwrap() {
        let row = company.stored_row(account.origin).unwrap().unwrap();
        let key = row.get("account_id").unwrap().as_i64();
        assert_eq!(key, Some(account.id.number().into()));
    }
    for preference in company.preferences().unwrap().stored() {
        let row = company.stored_row(preference.origin).unwrap().unwrap();
        let value = row.get("preference_value").unwrap().as_str();
        assert_eq!(value, Some(preference.value.as_str()));
    }
}

#[test]
fn the_row_below_has_the_columns_that_a_record_leaves_out() {
    let company = support::books();
    let parts = company.accounts().unwrap().named("Sales:Parts").unwrap();
    let row = company.stored_row(parts.origin).unwrap().unwrap();
    let table = company.source().definition(parts.origin.table).unwrap();
    assert_eq!(row.values().len(), table.columns().len());
    assert_eq!(row.get("sublevel").unwrap().as_i64(), Some(1));
}

#[test]
fn a_company_can_be_shared_between_threads() {
    fn shared<T: Send + Sync>(_: &T) {}
    let company = support::books();
    shared(&company);
    assert_eq!(format!("{company:?}").find("Checking"), None);
}

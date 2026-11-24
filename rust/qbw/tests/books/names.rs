//! Customers and jobs, vendors, employees, other names and their contacts.

use qbw::names::{ChannelKind, Contact, ContactKind, JobStatus, Name, NameKind};
use qbw::sa17::Value;
use qbw::{
    AddressId, ContactId, CurrencyId, CustomerTypeId, Error, ItemId, NameId, PaymentMethodId,
    PriceLevelId, Problem, SalesRepId, SalesTaxCodeId, TermsId, VendorTypeId,
};
use qbw_synthetic::Cell;

use crate::changed::{Changed, Set, date, text, whole};
use crate::small_lists::{Kind, State};
use crate::{small_lists, support};

#[test]
fn names_are_in_quickbooks_order_list_after_list() {
    let company = support::books();
    let names = company.names().unwrap();
    let in_order: Vec<(NameKind, &str)> = names
        .iter()
        .map(|name| (name.kind(), names.full_name(name.id()).unwrap()))
        .collect();
    assert_eq!(
        in_order,
        [
            (NameKind::Customer, "Ada Stores"),
            (NameKind::Customer, "Ada Stores:Annex"),
            (NameKind::Customer, "Bell Works"),
            (NameKind::Vendor, "Cobalt Parts"),
            (NameKind::Vendor, "State Tax Office"),
            (NameKind::Employee, "Eve Moss"),
            (NameKind::Other, "Quay Fuel"),
        ]
    );
    assert_eq!((names.len(), names.is_empty()), (7, false));
    assert!(names.left_out().is_nothing() && names.damage().next().is_none());
}

#[test]
fn an_id_finds_its_name_whichever_list_it_is_in() {
    let company = support::books();
    let names = company.names().unwrap();
    let found = |number| names.get(NameId::new(number)).map(Name::name);
    assert_eq!(found(103), Some("Annex"));
    assert_eq!(found(202), Some("State Tax Office"));
    assert_eq!(found(301), Some("Eve Moss"));
    assert_eq!(found(401), Some("Quay Fuel"));
    // The deleted customer, and an id that nothing has.
    assert_eq!((found(104), found(999)), (None, None));
    assert!(names.full_name(NameId::new(104)).is_none());
    assert!(names.customers().get(NameId::new(201)).is_none());
    assert_eq!(names.vendors().len(), 2);
    assert_eq!((names.employees().len(), names.other_names().len()), (1, 1));
}

#[test]
fn a_customer_has_what_the_window_shows() {
    let company = support::books();
    let customers = company.names().unwrap().customers();
    let ada = customers.named("ada stores").unwrap();
    assert_eq!(ada.id, NameId::new(101));
    assert_eq!((ada.parent, ada.job.as_ref()), (None, None));
    assert_eq!(ada.customer_type, Some(CustomerTypeId::new(1)));
    assert_eq!(ada.terms, Some(TermsId::new(1)));
    assert_eq!(ada.sales_rep, Some(SalesRepId::new(1)));
    assert_eq!(ada.price_level, Some(PriceLevelId::new(1)));
    assert_eq!(ada.payment_method, Some(PaymentMethodId::new(1)));
    assert_eq!(ada.tax_code, Some(SalesTaxCodeId::new(1)));
    assert_eq!(ada.tax_item, Some(ItemId::new(503)));
    assert_eq!(ada.currency, CurrencyId::new(1));
    assert_eq!(ada.credit_limit.unwrap().to_string(), "5000.00");
    assert_eq!((ada.class, ada.resale_number.as_deref()), (None, None));
    assert_eq!(ada.opened, "2023-03-01".parse().ok());
    assert_eq!(ada.list_id.as_deref(), Some("80000065-1"));
    assert_eq!(ada.created.unwrap().to_string(), "2023-03-01 08:00:00 UTC");
    assert!(ada.modified > ada.created && ada.is_active);
    let stored = ada.stored.unwrap();
    assert_eq!(stored.amount.to_string(), "45.00");
    assert_eq!(stored.home_amount, Some(stored.amount));
}

#[test]
fn a_job_is_a_customer_below_its_customer() {
    let company = support::books();
    let customers = company.names().unwrap().customers();
    let annex = customers.named("Ada Stores:Annex").unwrap();
    assert_eq!(annex.parent, Some(NameId::new(101)));
    let job = annex.job.as_ref().unwrap();
    assert_eq!(job.status, JobStatus::Other(3));
    assert_eq!(job.description.as_deref(), Some("New wing"));
    assert_eq!(job.start, "2024-04-01".parse().ok());
    assert_eq!(
        (job.projected_end, job.end, job.job_type),
        (None, None, None)
    );
    let jobs: Vec<&str> = customers
        .children(Some(NameId::new(101)))
        .map(|job| job.name.as_str())
        .collect();
    assert_eq!(jobs, ["Annex"]);
}

#[test]
fn a_foreign_customer_owes_in_its_own_currency() {
    let company = support::books();
    let home = company.preferences().unwrap().home_currency;
    let customers = company.names().unwrap().customers();
    let bell = customers.named("Bell Works").unwrap();
    assert_eq!(bell.currency, CurrencyId::new(2));
    let stored = bell.stored.unwrap();
    assert_eq!(stored.amount.currency(), CurrencyId::new(2));
    assert_eq!(stored.home_amount.unwrap().currency(), home);
    assert!(stored.amount.is_zero() && bell.credit_limit.is_none());
}

#[test]
fn a_vendor_has_what_the_window_shows() {
    let company = support::books();
    let vendors = company.names().unwrap().vendors();
    let cobalt = vendors.named("Cobalt Parts").unwrap();
    assert_eq!(cobalt.id, NameId::new(201));
    assert_eq!(cobalt.vendor_type, Some(VendorTypeId::new(1)));
    assert_eq!(cobalt.terms, Some(TermsId::new(1)));
    assert_eq!(cobalt.name_on_check.as_deref(), Some("Cobalt Parts Inc"));
    assert!(cobalt.is_1099 && cobalt.is_active);
    // A debit is positive, so this is what the company owes.
    assert_eq!(cobalt.stored.unwrap().amount.to_string(), "-20.00");
    assert_eq!(cobalt.opened, "2023-05-01".parse().ok());
    let state = vendors.named("State Tax Office").unwrap();
    assert!(!state.is_1099 && state.vendor_type.is_none());
}

#[test]
fn the_sales_tax_items_say_which_vendors_are_agencies() {
    let company = support::books();
    let items = company.items().unwrap();
    let agencies: Vec<NameId> = items.iter().filter_map(|item| item.tax?.agency).collect();
    let names = company.names().unwrap();
    let agencies = agencies.iter().map(|&agency| names.full_name(agency));
    let agencies: Vec<Option<&str>> = agencies.collect();
    assert_eq!(agencies, [Some("State Tax Office")]);
}

#[test]
fn an_employee_and_another_name_have_a_contact_and_no_balance() {
    let company = support::books();
    let names = company.names().unwrap();
    let eve = names.employees().named("Eve Moss").unwrap();
    assert_eq!(eve.hired, "2023-02-01".parse().ok());
    assert_eq!(eve.list_id.as_deref(), Some("8000012D-1"));
    let contact = eve.contact.as_ref().unwrap();
    assert_eq!(contact.first_name.as_deref(), Some("Eve"));
    let fuel = names.get(NameId::new(401)).unwrap();
    assert_eq!(fuel.kind(), NameKind::Other);
    let contact = fuel.contact().unwrap();
    assert_eq!(contact.id.as_str(), "CONTACT000000401");
    assert!(contact.address.is_empty() && contact.phone.is_none());
    assert_eq!(fuel.origin().table, "abmc_other_user");
}

#[test]
fn a_name_has_the_contact_that_is_in_use() {
    let company = support::books();
    let customers = company.names().unwrap().customers();
    let ada = customers.named("Ada Stores").unwrap();
    let contact = ada.contact.as_ref().unwrap();
    assert_eq!(contact.id.as_str(), "CONTACT000000101");
    assert_eq!(
        (contact.kind, contact.name),
        (ContactKind::Main, Some(ada.id))
    );
    assert_eq!(contact.company.as_deref(), Some("Ada Stores"));
    assert_eq!(contact.first_name.as_deref(), Some("Ada"));
    assert_eq!(contact.last_name.as_deref(), Some("Reed"));
    assert_eq!(contact.phone.as_deref(), Some("555-0101"));
    assert_eq!(contact.email.as_deref(), Some("ada@ada-stores.example"));
    assert_eq!(contact.account_number.as_deref(), Some("A-101"));
    assert_eq!((contact.expired, contact.fax.as_deref()), (None, None));

    let address = &contact.address;
    assert_eq!(
        address.printed,
        ["Ada Stores", "7 Mill Lane", "Northport, NP 10002"]
    );
    assert_eq!(address.street, ["7 Mill Lane"]);
    assert_eq!(address.city.as_deref(), Some("Northport"));
    assert_eq!(address.state.as_deref(), Some("NP"));
    assert_eq!(address.postal_code.as_deref(), Some("10002"));
    assert_eq!(address.country, None);
}

#[test]
fn every_name_has_a_contact_and_the_customers_one_its_people() {
    let company = support::books();
    let names = company.names().unwrap();
    assert!(names.iter().all(|name| name.contact().is_some()));
    let ada = names.customers().named("Ada Stores").unwrap();
    let [sam] = ada.additional_contacts.as_slice() else {
        panic!("one more person: {:?}", ada.additional_contacts);
    };
    assert_eq!((sam.kind, sam.name), (ContactKind::Secondary, Some(ada.id)));
    assert_eq!(sam.first_name.as_deref(), Some("Sam"));
    assert_eq!(sam.job_title.as_deref(), Some("Buyer"));
}

#[test]
fn a_name_has_the_channels_that_have_something() {
    let company = support::books();
    let customers = company.names().unwrap().customers();
    let ada = customers.named("Ada Stores").unwrap();
    let channels: Vec<(Option<i32>, Option<&str>, &str)> = ada
        .channels
        .iter()
        .map(|channel| {
            (
                channel.position,
                channel.label.as_deref(),
                channel.value.as_str(),
            )
        })
        .collect();
    assert_eq!(
        channels,
        [
            (Some(1), Some("Main Phone"), "555-0101"),
            (Some(5), Some("Main Email"), "ada@ada-stores.example"),
        ]
    );
    assert!(ada.channels.iter().all(|channel| channel.name == ada.id));
}

/// The contact of Ada Stores with other rows.
fn contact_of_ada(books: Changed) -> Contact {
    let company = books.company();
    let names = company.names().unwrap();
    let ada = names.get(NameId::new(101)).unwrap();
    let in_use = ada.contact().unwrap();
    // It is the same contact, asked for by its key.
    assert_eq!(company.contact(in_use.id).unwrap().as_ref(), Some(in_use));
    in_use.clone()
}

const CONTACT: &str = "abmc_contacts";
const CHANNELS: &str = "abmc_contacts_info";

/// What the list of channels has of Ada Stores in one category.
fn channel(category: i32, value: &str) -> [Set; 4] {
    [
        ("name_id", whole(101)),
        ("channel_category_id", whole(category)),
        ("position", whole(category)),
        ("field_value", text(value)),
    ]
}

#[test]
fn a_channel_has_what_the_contact_has_a_cut_copy_of() {
    let long = "555-0101 extension 22, ask for Ada";
    let books = Changed::books()
        .set((CHANNELS, "position", 1), "field_value", text(long))
        .set(
            (CONTACT, "name_id", 101),
            "telephone_num1",
            text(&long[..22]),
        )
        .added(CHANNELS, &channel(4, "555-0144 in the afternoon"))
        .added(CHANNELS, &channel(6, "555-0166 at the warehouse"))
        .added(CHANNELS, &channel(10, "orders@ada-stores.example"))
        .set((CONTACT, "name_id", 101), "fax_num", text("555-0155"));
    let contact = contact_of_ada(books);
    assert_eq!(contact.phone.as_deref(), Some(long));
    assert_eq!(contact.mobile.as_deref(), Some("555-0144 in the afternoon"));
    assert_eq!(
        contact.alt_phone.as_deref(),
        Some("555-0166 at the warehouse")
    );
    assert_eq!(contact.cc.as_deref(), Some("orders@ada-stores.example"));
    // The channel of the fax has nothing in it, and the contact has a number.
    assert_eq!(contact.fax.as_deref(), Some("555-0155"));
    let fax = channel(5, "555-0177 in the office upstairs");
    let books = Changed::books()
        .deleted((CHANNELS, "position", 4))
        .added(CHANNELS, &fax);
    let contact = contact_of_ada(books);
    assert_eq!(
        contact.fax.as_deref(),
        Some("555-0177 in the office upstairs")
    );
}

#[test]
fn a_channel_that_was_deleted_says_nothing() {
    let deleted = Changed::books().set(
        (CHANNELS, "position", 1),
        "is_deleted_bool",
        Value::Boolean(true),
    );
    // What the contact itself has is the fallback.
    assert_eq!(contact_of_ada(deleted).phone.as_deref(), Some("555-0101"));
}

#[test]
fn channels_say_of_which_kind_they_are() {
    let company = support::books();
    let ada = company.names().unwrap().customers().get(NameId::new(101));
    let kinds = ada.unwrap().channels.iter().map(|channel| channel.kind);
    let kinds: Vec<ChannelKind> = kinds.collect();
    assert_eq!(kinds, [ChannelKind::MainPhone, ChannelKind::MainEmail]);
}

#[test]
fn an_expired_contact_is_nobody_s() {
    // It is the only one of the name.
    let expired = Changed::books().set(
        (CONTACT, "name_id", 102),
        "expired_date",
        date("2024-01-02"),
    );
    let company = expired.company();
    let names = company.names().unwrap();
    assert!(names.get(NameId::new(102)).unwrap().contact().is_none());
    let old = ContactId::try_new("CONTACT000000102").unwrap();
    assert!(company.contact(old).unwrap().is_some());
}

#[test]
fn a_name_is_found_whichever_list_it_is_in() {
    let company = support::books();
    let names = company.names().unwrap();
    let found = |name| names.named(name).map(|found| found.id().number());
    assert_eq!(found("Ada Stores"), Some(101));
    assert_eq!(found("ADA STORES:ANNEX"), Some(103));
    assert_eq!(found("annex"), Some(103));
    assert_eq!(found("Cobalt Parts"), Some(201));
    assert_eq!(found("Nobody"), None);
    // A vendor that is called as a job is: its full name is what it is asked for
    // by, as the job's is not.
    let books = Changed::books().set(
        ("abmc_vendor_user", "vendor_id", 201),
        "name",
        text("Annex"),
    );
    let company = books.company();
    let names = company.names().unwrap();
    let vendor = names.named("Annex").map(|found| found.id().number());
    assert_eq!(vendor, Some(201));
    let all = names.all_named("Annex").into_iter();
    assert_eq!(
        all.map(|found| found.id().number()).collect::<Vec<_>>(),
        [201]
    );
    let job = names.named("Ada Stores:Annex");
    assert_eq!(job.map(|found| found.id().number()), Some(103));
    // Two jobs that are called the same have no name alone.
    let books = Changed::books().copied(
        ("abmc_customer_user", "customer_id", 103),
        &[("customer_id", whole(105)), ("parent_id", whole(102))],
    );
    let company = books.company();
    let names = company.names().unwrap();
    assert!(names.named("Annex").is_none());
    let all = |name| -> Vec<i32> {
        let found = names.all_named(name).into_iter();
        found.map(|found| found.id().number()).collect()
    };
    assert_eq!(all("ANNEX"), [103, 105]);
    assert_eq!(all("Ada Stores:annex"), [103]);
    assert!(all("Nobody").is_empty());
    let job = names.named("Bell Works:Annex");
    assert_eq!(job.map(|found| found.id().number()), Some(105));
    let customers = names.customers();
    assert!(customers.named("Annex").is_none());
}

#[test]
fn case_is_ignored_beyond_the_letters_of_english() {
    let name = ("abmc_vendor_user", "vendor_id", 201);
    let books = Changed::books().set(name, "name", text("Übersee Öl"));
    let company = books.company();
    let names = company.names().unwrap();
    let vendor = names.vendors().named("ÜBERSEE ÖL").map(|vendor| vendor.id);
    assert_eq!(vendor, Some(NameId::new(201)));
    assert!(names.named("übersee öl").is_some());
}

#[test]
fn an_own_name_that_two_entries_have_finds_neither() {
    // Two accounts that are called Parts, below two others.
    let books = Changed::books().set(
        ("abmc_account_user", "account_id", 14),
        "name",
        text("Parts"),
    );
    let books = books.set(
        ("abmc_account_user", "account_id", 14),
        "parent_id",
        whole(15),
    );
    let company = books.company();
    let accounts = company.accounts().unwrap();
    assert!(accounts.named("Parts").is_none());
    let both = accounts.all_named("parts").into_iter();
    assert_eq!(
        both.map(|account| account.id.number()).collect::<Vec<_>>(),
        [13, 14]
    );
    assert!(accounts.named("Sales:Parts").is_some());
    let whole = support::books();
    let parts = whole.accounts().unwrap().named("parts").unwrap();
    assert_eq!(parts.id.number(), 13);
}

#[test]
fn a_form_of_earlier_days_has_the_contact_of_then() {
    let company = support::books();
    let old = ContactId::try_new("CONTACT000000100").unwrap();
    let contact = company.contact(old).unwrap().unwrap();
    assert_eq!((contact.id, contact.name), (old, Some(NameId::new(101))));
    assert_eq!(contact.expired, "2024-01-02".parse().ok());
    assert_eq!(contact.address.printed, ["Ada Stores", "3 Old Road"]);
    let row = company.stored_row(contact.origin).unwrap().unwrap();
    let key = row.get("contacts_abm_id").unwrap().as_str();
    assert_eq!(key, Some("CONTACT000000100"));
    // The one in use is found the same way.
    let current = ContactId::try_new("CONTACT000000101").unwrap();
    let current = company.contact(current).unwrap();
    let customers = company.names().unwrap().customers();
    let ada = customers.get(NameId::new(101)).unwrap();
    assert_eq!(current, ada.contact);
    let unknown = ContactId::try_new("CONTACT000000999").unwrap();
    assert_eq!(company.contact(unknown).unwrap(), None);
}

#[test]
fn a_customer_has_a_list_of_addresses_and_old_forms_have_theirs() {
    let company = support::books();
    let customers = company.names().unwrap().customers();
    let ada = customers.named("Ada Stores").unwrap();
    let [warehouse] = ada.ship_to_addresses.as_slice() else {
        panic!("one address: {:?}", ada.ship_to_addresses);
    };
    assert_eq!(warehouse.label.as_deref(), Some("Warehouse"));
    assert_eq!(ada.ship_to, Some(warehouse.id));
    assert_eq!(warehouse.address.printed, ["Ada Stores", "Dock 2"]);
    assert_eq!(warehouse.address.postal_code.as_deref(), Some("10003"));
    assert_eq!((warehouse.name, warehouse.expired), (Some(ada.id), None));

    let old = AddressId::try_new("SHIPTO0000000400").unwrap();
    let old = company.ship_to(old).unwrap().unwrap();
    assert_eq!(old.address.printed, ["Ada Stores", "Dock 1"]);
    assert_eq!(old.expired, "2023-11-15".parse().ok());
    assert!(old.label.is_none());
    let unknown = AddressId::try_new("SHIPTO0000000999").unwrap();
    assert_eq!(company.ship_to(unknown).unwrap(), None);
}

#[test]
fn every_name_leads_to_its_stored_row() {
    let company = support::books();
    for name in company.names().unwrap().iter() {
        let row = company.stored_row(name.origin()).unwrap().unwrap();
        let name_of_row = row.get("name").unwrap().as_str();
        assert_eq!(name_of_row, Some(name.name()));
    }
}

#[test]
fn an_older_file_has_names_without_what_came_later() {
    let names = [
        (Some(1), "Till", State::Shown),
        (Some(2), "Attic", State::Hidden),
    ];
    let company = small_lists::with_names(&names);
    let names = company.names().unwrap();
    assert!(names.left_out().is_nothing());
    let till = names.customers().named("Till").unwrap();
    assert_eq!(till.currency, CurrencyId::COMPANY);
    assert!(till.stored.is_none() && till.contact.is_none() && till.job.is_none());
    assert!(till.channels.is_empty() && till.ship_to_addresses.is_empty());
    let attic = names.get(NameId::new(2)).unwrap();
    assert!(!attic.is_active() && till.is_active);
    assert_eq!(names.customers().active().count(), 1);
    // The file has no contacts at all.
    let contact = ContactId::try_new("CONTACT000000101").unwrap();
    assert_eq!(company.contact(contact).unwrap(), None);
}

#[test]
fn a_file_without_names_has_no_list_of_them() {
    let company = small_lists::company([]);
    let error = company.names().map(|_| ()).unwrap_err();
    assert!(matches!(
        error,
        Error::NoTable {
            table: "abmc_customer_internal"
        }
    ));
}

#[test]
fn a_name_that_cannot_be_read_is_left_out_and_counted() {
    let company = small_lists::with_names(&[
        (Some(1), "Till", State::Shown),
        (None, "Lost", State::Shown),
    ]);
    let names = company.names().unwrap();
    assert_eq!((names.len(), names.left_out().rows), (1, 1));
    let damage = names.damage().next().unwrap();
    assert_eq!(damage.table, "abmc_customer_user");
    let problem = &damage.problem;
    assert!(matches!(
        problem,
        Problem::Column {
            column: "customer_id",
            ..
        }
    ));
}

#[test]
fn a_contact_that_cannot_be_read_may_be_the_one_asked_for() {
    let row = |key: Cell, name| [key, Cell::integer(name), Cell::Null];
    let contacts = small_lists::table(
        "abmc_contacts",
        [
            ("contacts_abm_id", Kind::Text),
            ("name_id", Kind::Whole),
            ("expired_date", Kind::Day),
        ],
        [row(Cell::text("CONTACT1"), 1), row(Cell::Null, 2)],
    );
    let mut tables = small_lists::name_tables(&[
        (Some(1), "Till", State::Shown),
        (Some(2), "Attic", State::Shown),
    ]);
    tables.push(contacts);
    let company = small_lists::company(tables);
    let names = company.names().unwrap();
    assert_eq!((names.len(), names.left_out().rows), (2, 1));
    assert!(names.customers().named("Attic").unwrap().contact.is_none());

    let known = ContactId::try_new("CONTACT1").unwrap();
    assert!(company.contact(known).unwrap().is_some());
    let lost = ContactId::try_new("CONTACT2").unwrap();
    assert!(matches!(company.contact(lost), Err(Error::Damaged(_))));
}

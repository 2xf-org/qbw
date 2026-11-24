//! Who the company is, what QuickBooks keeps about it, and who works on it.

use qbw::UserId;
use qbw_synthetic::Cell;

use crate::small_lists::Kind;
use crate::{small_lists, support};

#[test]
fn the_company_says_who_it_is() {
    let company = support::books();
    let info = company.info().unwrap();
    assert_eq!(info.name, "Harbor Valves");
    assert_eq!(info.legal_name.as_deref(), Some("Harbor Valves Ltd"));
    let legal = &info.legal_address;
    assert_eq!(legal.street, ["1 Quay Street"]);
    assert_eq!(legal.city.as_deref(), Some("Northport"));
    assert_eq!(legal.state.as_deref(), Some("NP"));
    assert_eq!(legal.postal_code.as_deref(), Some("10001"));
    assert_eq!(legal.country.as_deref(), Some("US"));
    assert!(legal.printed.is_empty());
    assert_eq!(info.address_lines, ["1 Quay Street", "Northport, NP 10001"]);
    assert_eq!(info.phone.as_deref(), Some("555-0100"));
    assert_eq!(info.fax, None);
    assert_eq!(info.email.as_deref(), Some("office@harbor-valves.example"));
    assert_eq!(info.website.as_deref(), Some("harbor-valves.example"));
    assert_eq!(info.started, "2023-01-01".parse().ok());
    let created = info.created.unwrap();
    assert_eq!(created.to_string(), "2023-01-01 09:30:00 UTC");
    assert!(info.left_out.is_nothing());
}

#[test]
fn the_file_says_which_quickbooks_it_is_of() {
    let company = support::books();
    let version = &company.info().unwrap().version;
    assert_eq!((version.year, version.update), (Some(2024), Some(10)));
    assert_eq!((&version.edition, &version.newest_used), (&None, &None));

    let row = |name, value| [Cell::text(name), Cell::text(value)];
    let values = small_lists::table(
        "CF_SYSINFO",
        [("sysinfo_name", Kind::Text), ("sysinfo_value", Kind::Text)],
        [
            row("DBMajorVersion", "33"),
            row("DBReleaseNumber", "seven"),
            row("DBFlavorStratum", "some_edition"),
            row("NewestQBVersionUsed", "33 0 7 1"),
        ],
    );
    let company = small_lists::company([values]);
    let version = &company.info().unwrap().version;
    assert_eq!((version.year, version.update), (Some(2023), None));
    assert_eq!(version.edition.as_deref(), Some("some_edition"));
    assert_eq!(version.newest_used.as_deref(), Some("33 0 7 1"));
}

#[test]
fn a_company_that_says_nothing_is_called_nothing() {
    let company = support::with_preferences(&[("STARTDATE", "soon"), ("CREATETIME", "early")]);
    let info = company.info().unwrap();
    assert_eq!((info.name.as_str(), &info.legal_name), ("", &None));
    assert!(info.legal_address.is_empty() && info.address_lines.is_empty());
    assert_eq!((info.started, info.created), (None, None));
    assert_eq!(info.version.year, None);
}

#[test]
fn facts_are_found_by_key() {
    let company = support::books();
    let facts = company.facts().unwrap();
    assert_eq!((facts.len(), facts.is_empty()), (2, false));
    let keys: Vec<&str> = facts.iter().map(|fact| fact.key.as_str()).collect();
    assert_eq!(
        keys,
        [
            "MultiCurrencyUsed",
            "QBF_NetIncomePnLAccrualInLastFiscalYear"
        ]
    );
    let used = facts.get("MultiCurrencyUsed").unwrap();
    assert_eq!(used.value, "true");
    assert_eq!(used.origin.table, "abmc_cf_facts");
    assert!(facts.get("multicurrencyused").is_none());
    assert!(facts.left_out().is_nothing() && facts.damage().is_empty());
}

#[test]
fn the_audit_trail_names_the_users() {
    let company = support::books();
    let users = company.users().unwrap();
    assert_eq!((users.len(), users.is_empty()), (1, false));
    let admin = users.get(UserId::new(1)).unwrap();
    assert_eq!(admin.names, ["Admin"]);
    assert!(users.get(UserId::new(2)).is_none());
}

#[test]
fn a_user_has_the_names_of_every_save_and_deletion() {
    let row = |user, name, time, deleting_user, deleting_name: &str, delete_time| {
        [
            Cell::integer(user),
            Cell::text(name),
            Cell::integer(time),
            Cell::integer(deleting_user),
            Cell::text(deleting_name),
            Cell::integer(delete_time),
        ]
    };
    let saves = small_lists::table(
        "abmc_audit_header",
        [
            ("mod_user_id", Kind::Whole),
            ("mod_user_name", Kind::Text),
            ("modtimestamp_tms", Kind::Whole),
            ("delete_user_id", Kind::Whole),
            ("delete_user_name", Kind::Text),
            ("delete_tms", Kind::Whole),
        ],
        [
            row(2, "Kim", 1_700_000_000, 0, "", 0),
            row(2, "Kim Lee", 1_700_000_500, 0, "", 0),
            row(2, "Kim", 1_700_000_100, 3, "Pat", 1_700_000_200),
        ],
    );
    let company = small_lists::company([saves]);
    let users = company.users().unwrap();
    let named: Vec<(i32, &[String])> = users
        .iter()
        .map(|user| (user.id.number(), user.names.as_slice()))
        .collect();
    let kim = ["Kim Lee".to_owned(), "Kim".to_owned()];
    assert_eq!(named, [(2, &kim[..]), (3, &["Pat".to_owned()][..])]);
}

#[test]
fn a_file_that_lacks_the_tables_has_no_facts_and_no_users() {
    let company = small_lists::company([]);
    assert!(company.facts().unwrap().is_empty());
    assert!(company.users().unwrap().is_empty());
}

#[test]
fn what_may_hold_an_address_is_not_in_the_debug_output() {
    let company = support::books();
    let facts = format!("{:?}", company.facts().unwrap());
    assert!(facts.contains("MultiCurrencyUsed") && !facts.contains("true"));
    let users = format!("{:?}", company.users().unwrap());
    assert!(users.contains("UserId(1)") && !users.contains("Admin"));
}

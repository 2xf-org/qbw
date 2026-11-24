//! Lists and settings.

use qbw::{Company, List, ListEntry, Origin};

use crate::support;

fn assert_stored(company: &Company, origin: Origin) {
    let row = company.stored_row(origin).unwrap();
    assert_eq!(row.map(|row| row.id()), Some(origin.row), "{origin}");
}

/// Checks that no name of the list is empty, and that every parent is there.
fn assert_nested<T: ListEntry>(list: &List<T>) {
    for entry in list {
        assert!(!entry.name().is_empty());
        assert!(
            entry
                .parent()
                .is_none_or(|parent| list.get(parent).is_some())
        );
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn accounts_and_preferences_read_and_lead_to_their_rows() {
    for company in support::samples() {
        let accounts = company.accounts().unwrap();
        assert!(accounts.left_out().is_nothing() && !accounts.is_empty());
        assert_nested(accounts);
        for account in accounts {
            assert_stored(&company, account.origin);
        }
        let preferences = company.preferences().unwrap();
        assert!(preferences.left_out.is_nothing());
        for preference in preferences.stored() {
            assert_stored(&company, preference.origin);
        }
        println!(
            "{} accounts and {} preferences",
            accounts.len(),
            preferences.stored().len()
        );
    }
}

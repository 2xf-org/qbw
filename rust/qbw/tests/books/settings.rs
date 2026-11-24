//! How the company keeps its books.

use qbw::chrono::Month;
use qbw::settings::AgingBasis;
use qbw::{CurrencyId, Error, ItemId};

use crate::support;

#[test]
fn preferences_are_the_company_s_not_a_user_s() {
    let company = support::books();
    let preferences = company.preferences().unwrap();
    assert_eq!(preferences.home_currency, CurrencyId::new(1));
    assert_eq!(preferences.fiscal_year_start, Month::January);
    assert_eq!(preferences.tax_year_start, Some(Month::January));
    assert_eq!(preferences.aging_basis, AgingBasis::DueDate);
    assert_eq!(preferences.default_tax_item, Some(ItemId::new(503)));
    assert_eq!(preferences.closing_date.as_deref(), Some("12/31/2022"));
    assert!(preferences.uses_multicurrency && preferences.uses_account_numbers);
    assert!(preferences.uses_inventory && preferences.uses_sales_tax);
    assert!(preferences.uses_sales_orders && !preferences.uses_classes);
    assert!(!preferences.uses_sites && !preferences.uses_bins);
    assert!(preferences.available_less_sales_orders && preferences.available_less_builds);
    assert!(preferences.left_out.is_nothing());

    let stored = preferences.stored();
    assert!(
        stored
            .iter()
            .all(|preference| preference.name != "SHOWLOWESTSUB")
    );
    let address: Vec<&str> = stored
        .iter()
        .filter(|preference| preference.name == "ADDR")
        .map(|preference| preference.value.as_str())
        .collect();
    assert_eq!(address, ["1 Quay Street", "Northport, NP 10001"]);
}

#[test]
fn a_company_of_one_currency_names_none() {
    let company = support::with_preferences(&[]);
    let preferences = company.preferences().unwrap();
    assert_eq!(preferences.home_currency, CurrencyId::COMPANY);
    assert_eq!(preferences.fiscal_year_start, Month::January);
    assert_eq!(preferences.tax_year_start, None);
    assert!(!preferences.uses_multicurrency);
}

#[test]
fn years_can_start_with_any_month() {
    let company = support::with_preferences(&[("FISCALBEGIN", "7"), ("TAXBEGIN", "10")]);
    let preferences = company.preferences().unwrap();
    assert_eq!(preferences.fiscal_year_start, Month::July);
    assert_eq!(preferences.tax_year_start, Some(Month::October));
}

#[test]
fn a_flag_is_set_by_a_one_or_a_t() {
    let company = support::with_preferences(&[
        ("USECLASSES", "T"),
        ("MULTI_SITE", "1"),
        ("INVENTORY", "F"),
        ("AGEFROMTXDATE", "1"),
    ]);
    let preferences = company.preferences().unwrap();
    assert!(preferences.uses_classes && preferences.uses_sites);
    assert!(!preferences.uses_inventory && !preferences.uses_sales_tax);
    assert_eq!(preferences.aging_basis, AgingBasis::TransactionDate);
}

#[test]
fn a_currency_or_a_year_that_cannot_be_understood_is_an_error() {
    for preference in [("HOMECURRENCY", "euro"), ("FISCALBEGIN", "13")] {
        let company = support::with_preferences(&[preference]);
        let Err(Error::Damaged(damage)) = company.preferences() else {
            panic!("{preference:?} decides every figure");
        };
        assert_eq!(damage.table, "cf_pref_attribute_value");
        // Nothing was kept, and the error comes again.
        assert!(company.preferences().is_err());
        assert!(matches!(company.accounts(), Err(Error::Damaged(_))));
    }
}

#[test]
fn a_preference_that_cannot_be_read_may_be_the_one_that_decides() {
    let lost = support::with_a_damaged_preference(&[("HOMECURRENCY", "1")]);
    assert!(matches!(lost.preferences(), Err(Error::Damaged(_))));
    let known = [("HOMECURRENCY", "1"), ("FISCALBEGIN", "1")];
    let company = support::with_a_damaged_preference(&known);
    let preferences = company.preferences().unwrap();
    assert_eq!(
        (preferences.stored().len(), preferences.left_out.rows),
        (2, 1)
    );
}

#[test]
fn a_preference_s_name_and_value_are_not_in_its_debug_output() {
    let company = support::books();
    let shown = format!("{:?}", company.preferences().unwrap());
    assert!(!shown.contains("Quay Street") && !shown.contains("LEGALADDR1"));
}

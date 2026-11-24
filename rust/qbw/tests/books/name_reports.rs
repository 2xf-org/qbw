//! What names owe and are owed, as reports.

use qbw::chrono::NaiveDate;
use qbw::reports::{NameBalances, OwedBalance};
use qbw::{AsOf, CurrencyId, NameId, Period, Side};

use crate::changed::Changed;
use crate::support;

fn day(text: &str) -> NaiveDate {
    text.parse().unwrap()
}

fn rows(balances: &NameBalances) -> Vec<(i32, String, String)> {
    let rows = balances.rows.iter().map(|row| {
        (
            row.name.number(),
            row.balance.home_amount.to_string(),
            row.total.to_string(),
        )
    });
    rows.collect()
}

fn row(name: i32, home: &str, total: &str) -> (i32, String, String) {
    (name, home.to_owned(), total.to_owned())
}

fn texts(balance: OwedBalance) -> (String, String) {
    (balance.amount.to_string(), balance.home_amount.to_string())
}

fn amounts(own: &str, home: &str) -> (String, String) {
    (own.to_owned(), home.to_owned())
}

#[test]
fn name_balances_are_what_is_owed_in_the_list_s_order() {
    let company = support::books();
    let receivable = company
        .name_balances(Side::Receivable, day("2024-02-29"))
        .unwrap();
    let expected = [row(101, "46.00", "46.00"), row(102, "110.00", "110.00")];
    assert_eq!(rows(&receivable), expected);
    let euros = receivable.rows[1].balance;
    assert_eq!(
        (texts(euros), euros.amount.currency()),
        (amounts("100.00", "110.00"), CurrencyId::new(2))
    );
    assert_eq!(receivable.total.to_string(), "156.00");
    let now = company
        .name_balances(Side::Receivable, AsOf::Latest)
        .unwrap();
    assert_eq!(rows(&now), [row(101, "45.00", "45.00")]);
    assert!(now.left_out.is_nothing());
}

#[test]
fn a_name_without_a_balance_has_one_of_nothing() {
    let company = support::books();
    let receivable = company.name_balances(Side::Receivable, AsOf::Latest);
    let receivable = receivable.unwrap();
    let ada = receivable.balance(NameId::new(101)).unwrap();
    assert_eq!(receivable.rows, [ada]);
    for (name, currency) in [(102, 2), (103, 1)] {
        let name = NameId::new(name);
        let nothing = receivable.balance(name).unwrap();
        assert_eq!(nothing.name, name);
        assert_eq!(texts(nothing.balance), amounts("0.00", "0.00"));
        assert_eq!(nothing.balance.amount.currency(), CurrencyId::new(currency));
        let home = [nothing.balance.home_amount, nothing.total];
        assert_eq!(home.map(|amount| amount.currency().number()), [1, 1]);
        assert!(nothing.total.is_zero());
    }
    // A deleted name, a vendor, and one that is not there.
    for name in [104, 201, 999].map(NameId::new) {
        assert_eq!(receivable.balance(name), None);
    }
    let payable = company.name_balances(Side::Payable, AsOf::Latest).unwrap();
    let state = payable.balance(NameId::new(202)).unwrap();
    assert_eq!(texts(state.balance), amounts("0.00", "0.00"));
    assert_eq!(payable.balance(NameId::new(101)), None);
}

#[test]
fn a_name_that_cannot_be_read_has_its_balance() {
    let customer = ("abmc_customer_user", "customer_id", 101);
    let company = Changed::books().unreadable(customer).company();
    let balances = company.name_balances(Side::Receivable, AsOf::Latest);
    let balances = balances.unwrap();
    let ada = balances.balance(NameId::new(101)).unwrap();
    assert_eq!(
        (balances.rows.last(), &ada.total.to_string()[..]),
        (Some(&ada), "45.00")
    );
}

#[test]
fn a_vendor_is_owed_bills_and_sales_tax_above_zero() {
    let company = support::books();
    let payable = company
        .name_balances(Side::Payable, day("2023-12-31"))
        .unwrap();
    assert_eq!(rows(&payable), [row(202, "12.00", "12.00")]);
    let now = company.name_balances(Side::Payable, AsOf::Latest).unwrap();
    assert_eq!(rows(&now), [row(201, "20.00", "20.00")]);
}

#[test]
fn a_name_ledger_runs_through_what_is_owed() {
    let company = support::books();
    let ada = NameId::new(101);
    let ledger = company
        .name_ledger(Side::Receivable, ada, Period::all())
        .unwrap()
        .unwrap();
    let lines: Vec<(i32, String, String)> = ledger
        .rows
        .iter()
        .map(|row| {
            let line = row.posting.line.number();
            (
                line,
                row.change.home_amount.to_string(),
                row.balance.home_amount.to_string(),
            )
        })
        .collect();
    let expected = [
        (401, "202.00", "202.00"),
        (502, "-100.00", "102.00"),
        (503, "-2.00", "100.00"),
        (701, "-54.00", "46.00"),
        (2401, "-1.00", "45.00"),
        (2402, "0.00", "45.00"),
    ];
    let expected =
        expected.map(|(line, amount, balance)| (line, amount.to_owned(), balance.to_owned()));
    assert_eq!(lines, expected);

    let year = company.fiscal_year(day("2024-06-30")).unwrap();
    let ledger = company
        .name_ledger(Side::Receivable, ada, year)
        .unwrap()
        .unwrap();
    assert_eq!(
        (texts(ledger.opening), ledger.rows.len()),
        (amounts("202.00", "202.00"), 5)
    );
    assert_eq!(texts(ledger.closing), amounts("45.00", "45.00"));
}

#[test]
fn a_name_ledger_is_in_the_name_s_currency_and_in_the_company_s() {
    let company = support::books();
    let bell = company
        .name_ledger(Side::Receivable, NameId::new(102), Period::all())
        .unwrap()
        .unwrap();
    let balances: Vec<_> = bell.rows.iter().map(|row| texts(row.balance)).collect();
    assert_eq!(
        balances,
        [amounts("100.00", "110.00"), amounts("0.00", "0.00")]
    );
    let cobalt = company
        .name_ledger(Side::Payable, NameId::new(201), Period::all())
        .unwrap()
        .unwrap();
    let balances: Vec<_> = cobalt
        .rows
        .iter()
        .map(|row| row.balance.home_amount.to_string())
        .collect();
    assert_eq!(balances, ["200.00", "0.00", "-20.00", "20.00"]);
    // A name that is not there, and one that is a customer.
    for name in [999, 101] {
        let nobody = company.name_ledger(Side::Payable, NameId::new(name), Period::all());
        assert_eq!(nobody.unwrap(), None, "name {name}");
    }
}

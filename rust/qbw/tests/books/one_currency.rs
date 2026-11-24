//! The small company that uses one currency.

use qbw::aging::{AgingBy, AgingOptions};
use qbw::chrono::NaiveDate;
use qbw::query::TransactionQuery;
use qbw::transactions::TransactionKind;
use qbw::{AccountId, AsOf, Company, CurrencyId, NameId, Page, Period, Side, TransactionId};
use qbw_synthetic::company;

use crate::forms;

fn books() -> Company {
    Company::from_bytes(company::one_currency_books().build()).unwrap()
}

fn today() -> NaiveDate {
    "2024-03-31".parse().unwrap()
}

#[test]
fn it_keeps_its_books_in_the_company_s_currency() {
    let company = books();
    let preferences = company.preferences().unwrap();
    assert_eq!(preferences.home_currency, CurrencyId::COMPANY);
    assert!(!preferences.uses_multicurrency && preferences.left_out.is_nothing());
    assert_eq!(company.info().unwrap().name, "Corner Valves");
    // QuickBooks fills the list of currencies in all the same.
    assert_eq!(company.currencies().unwrap().len(), 3);
    let accounts = company.accounts().unwrap();
    let of_accounts = accounts.iter().map(|account| account.currency);
    let names = company.names().unwrap();
    let of_customers = names.customers().iter().map(|customer| customer.currency);
    let of_vendors = names.vendors().iter().map(|vendor| vendor.currency);
    let mut all = of_accounts.chain(of_customers).chain(of_vendors);
    assert!(all.all(|currency| currency == CurrencyId::COMPANY));
}

#[test]
fn its_accounts_come_to_nothing_together() {
    let company = books();
    let balances = company.account_listing(AsOf::Latest).unwrap();
    let accounts = company.accounts().unwrap();
    let rows = balances.rows.iter().filter_map(|row| {
        let name = accounts.own_name(row.account)?;
        Some(format!("{name} {}", row.balance?.home_amount))
    });
    let expected = [
        "Checking 505.00",
        "Accounts Receivable 70.00",
        "Accounts Payable -15.00",
        "Opening Balance Equity -500.00",
        "Retained Earnings 0.00",
    ];
    assert_eq!(rows.collect::<Vec<_>>(), expected);
    let trial = company.trial_balance(today()).unwrap();
    let [debits, credits] = [trial.debits, trial.credits].map(|sum| sum.to_string());
    assert_eq!((debits.as_str(), credits.as_str()), ("635.00", "635.00"));
    let year = company.profit_and_loss(Period::all()).unwrap();
    let sums = [year.income.total, year.expense.total, year.net_income];
    assert_eq!(
        sums.map(|sum| sum.to_string()),
        ["120.00", "60.00", "60.00"]
    );
    let sheet = company.balance_sheet(today()).unwrap();
    assert_eq!(sheet.assets.total.to_string(), "575.00");
    assert_eq!(sheet.liabilities_and_equity, sheet.assets.total);
}

#[test]
fn what_is_owed_is_what_the_payments_left() {
    let company = books();
    for (side, name, rest, number) in [
        (Side::Receivable, 101, "70.00", "1041"),
        (Side::Payable, 201, "15.00", "7781"),
    ] {
        for as_of in [AsOf::Latest, AsOf::Date(today())] {
            let open = company.open_items(side, as_of).unwrap();
            let [item] = &open.items[..] else {
                panic!("one item is open");
            };
            assert_eq!(item.name, Some(NameId::new(name)));
            assert_eq!(item.number.as_deref(), Some(number));
            assert_eq!(
                (item.open.to_string(), item.open.currency()),
                (rest.into(), CurrencyId::COMPANY)
            );
            assert_eq!(item.open, item.home_open);
        }
        let options = AgingOptions::new(today()).by(AgingBy::Items);
        let aging = company.aging(side, &options).unwrap();
        assert_eq!(aging.totals.total.to_string(), rest);
        let balances = company.name_balances(side, AsOf::Latest).unwrap();
        let row = balances.balance(NameId::new(name)).unwrap();
        assert_eq!(row.balance.amount.to_string(), rest);
    }
}

#[test]
fn its_forms_are_of_one_currency() {
    let company = books();
    let page = company.transactions(&TransactionQuery::new(), Page::all());
    let page = page.unwrap();
    assert!(page.left_out.is_nothing());
    let mut rows = page.rows.iter();
    assert!(rows.all(|row| row.currency == CurrencyId::COMPANY));
    let ids: Vec<i32> = page.rows.iter().map(|row| row.id.number()).collect();
    assert_eq!(ids, [500, 300, 400, 200, 100]);
    let invoice = company.find(TransactionKind::Invoice, "1041").unwrap();
    let invoice = forms::transaction(&company, invoice.unwrap().number());
    assert_eq!(forms::money(invoice.total), "120.00 (0)");
    assert_eq!(forms::lines(&invoice), ["Item 12 501 3 40.00 120.00"]);
    assert_eq!(forms::money(invoice.open()), "70.00 (0)");
    assert_eq!(invoice.exchange_rate.to_string(), "1");
    let paid = company.paid_by(TransactionId::new(400)).unwrap().unwrap();
    assert_eq!(
        (paid.total.to_string(), paid.links.len()),
        ("50.00".into(), 1)
    );
    let checking = AccountId::new(1);
    let register = company.register(checking, Period::all(), Page::all());
    let register = register.unwrap().unwrap();
    let rows = register.rows.iter().map(|row| row.balance.to_string());
    assert_eq!(rows.collect::<Vec<_>>(), ["500.00", "455.00", "505.00"]);
}

#[test]
fn it_agrees_with_what_quickbooks_keeps() {
    let check = books().check().unwrap();
    assert!(check.findings.is_empty(), "{:?}", check.findings);
    assert!(check.left_out.is_nothing());
    assert!(check.compared.iter().any(|compared| compared.looked_at > 0));
}

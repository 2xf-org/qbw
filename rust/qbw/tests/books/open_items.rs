//! What is owed, now and on a day.

use qbw::chrono::NaiveDate;
use qbw::open_items::{OpenItem, OpenItems};
use qbw::transactions::TransactionKind;
use qbw::{AccountId, AsOf, CurrencyId, NameId, Side};

use crate::support;

fn day(text: &str) -> NaiveDate {
    text.parse().unwrap()
}

fn open(side: Side, as_of: AsOf) -> OpenItems {
    support::books().open_items(side, as_of).unwrap()
}

/// The transactions of items, with the total and what is open of it.
fn lines<'items>(items: impl IntoIterator<Item = &'items OpenItem>) -> Vec<(i32, String, String)> {
    let items = items.into_iter().map(|item| {
        (
            item.transaction.number(),
            item.total.to_string(),
            item.open.to_string(),
        )
    });
    items.collect()
}

fn line(transaction: i32, total: &str, open: &str) -> (i32, String, String) {
    (transaction, total.to_owned(), open.to_owned())
}

#[test]
fn what_is_open_now_is_what_the_file_keeps() {
    let receivable = open(Side::Receivable, AsOf::Latest);
    assert_eq!(lines(&receivable), [line(400, "202.00", "45.00")]);
    let invoice = &receivable.items[0];
    assert_eq!(
        (invoice.kind, invoice.date),
        (TransactionKind::Invoice, day("2023-11-15"))
    );
    assert_eq!(invoice.number.as_deref(), Some("1041"));
    assert_eq!(
        (invoice.name, invoice.account),
        (Some(NameId::new(101)), AccountId::new(3))
    );
    assert_eq!(invoice.due_date, Some(day("2023-12-15")));
    assert_eq!(invoice.home_open, invoice.open);
    assert!(receivable.adjustments.is_zero() && receivable.left_out.is_nothing());

    // A credit of a vendor takes off what the receipt is owed.
    let payable = open(Side::Payable, AsOf::Latest);
    let expected = [line(1200, "-20.00", "-20.00"), line(1500, "40.00", "40.00")];
    assert_eq!(lines(&payable), expected);
    assert_eq!(payable.home_total().to_string(), "20.00");
}

#[test]
fn what_was_open_on_a_day_is_less_what_was_settled_by_then() {
    let receivable = |date: &str| lines(&open(Side::Receivable, AsOf::Date(day(date))));
    assert_eq!(receivable("2023-11-14"), []);
    assert_eq!(receivable("2023-12-31"), [line(400, "202.00", "202.00")]);
    assert_eq!(receivable("2024-01-31"), [line(400, "202.00", "100.00")]);
    let february = [line(400, "202.00", "46.00"), line(800, "100.00", "100.00")];
    assert_eq!(receivable("2024-02-15"), february);
    assert_eq!(receivable("2024-05-31"), [line(400, "202.00", "46.00")]);
    // A discount without money in June.
    assert_eq!(receivable("2024-06-30"), [line(400, "202.00", "45.00")]);

    let payable = |date: &str| lines(&open(Side::Payable, AsOf::Date(day(date))));
    assert_eq!(payable("2023-05-31"), [line(200, "200.00", "200.00")]);
    assert_eq!(payable("2023-12-31"), []);
    let in_june = [line(1200, "-20.00", "-20.00"), line(1500, "40.00", "40.00")];
    assert_eq!(payable("2024-06-30"), in_june);
}

#[test]
fn open_items_come_to_the_balances_of_their_accounts() {
    let company = support::books();
    let ledger = company.ledger().unwrap();
    for date in ["2023-12-31", "2024-01-31", "2024-02-15", "2024-06-30"].map(day) {
        let receivable = company.open_items(Side::Receivable, date).unwrap();
        let accounts = [3, 4].map(|account| {
            ledger
                .balance(AccountId::new(account), date)
                .unwrap()
                .home_amount
        });
        let balance = accounts[0].checked_add(accounts[1]).unwrap();
        assert_eq!(receivable.home_total(), balance, "{date}");
        let payable = company.open_items(Side::Payable, date).unwrap();
        let balance = ledger.balance(AccountId::new(7), date).unwrap().home_amount;
        assert_eq!(payable.home_total(), -balance, "{date}");
    }
}

#[test]
fn an_item_in_another_currency_has_two_amounts() {
    let receivable = open(Side::Receivable, AsOf::Date(day("2024-02-29")));
    let [_, euros] = receivable.items.as_slice() else {
        panic!("two invoices are open: {receivable:?}");
    };
    assert_eq!(
        (euros.open.to_string(), euros.open.currency()),
        ("100.00".to_owned(), CurrencyId::new(2))
    );
    assert_eq!(
        (euros.home_open.to_string(), euros.home_open.currency()),
        ("110.00".to_owned(), CurrencyId::new(1))
    );
    let totals = receivable.totals();
    assert_eq!(totals.get(CurrencyId::new(1)).to_string(), "46.00");
    assert_eq!(totals.get(CurrencyId::new(2)).to_string(), "100.00");
    assert_eq!(receivable.home_total().to_string(), "156.00");
}

#[test]
fn one_call_says_what_a_customer_owes_and_what_is_overdue() {
    let company = support::books();
    let owed = |customer, date: &str| {
        let owed = company
            .owed_by(NameId::new(customer), day(date))
            .unwrap()
            .unwrap();
        (owed.home_total.to_string(), owed.home_overdue.to_string())
    };
    let texts = |total: &str, overdue: &str| (total.to_owned(), overdue.to_owned());
    // Due on the 15th of December.
    assert_eq!(owed(101, "2023-12-15"), texts("202.00", "0.00"));
    assert_eq!(owed(101, "2023-12-16"), texts("202.00", "202.00"));
    assert_eq!(owed(101, "2024-06-30"), texts("45.00", "45.00"));
    assert_eq!(owed(102, "2024-03-01"), texts("110.00", "0.00"));
    assert_eq!(owed(102, "2024-03-20"), texts("0.00", "0.00"));
    let euros = company
        .owed_by(NameId::new(102), day("2024-03-01"))
        .unwrap()
        .unwrap();
    assert_eq!(
        euros.total.single().map(|total| total.to_string()),
        Some("100.00".to_owned())
    );
    assert!(euros.overdue.is_empty());
    assert_eq!(lines(&euros.items), [line(800, "100.00", "100.00")]);
    // The job has nothing but a pending invoice.
    assert!(
        company
            .owed_by(NameId::new(103), day("2024-06-30"))
            .unwrap()
            .unwrap()
            .items
            .is_empty()
    );
}

#[test]
fn a_credit_takes_off_what_a_vendor_is_owed_but_not_what_is_overdue() {
    let company = support::books();
    let owed = company
        .owed_to(NameId::new(201), day("2024-06-30"))
        .unwrap()
        .unwrap();
    assert_eq!((owed.side, owed.items.len()), (Side::Payable, 2));
    assert_eq!(owed.home_total.to_string(), "20.00");
    assert_eq!(owed.home_overdue.to_string(), "40.00");
    let receipt = &owed.items[1];
    assert_eq!(receipt.days_overdue(owed.day), Some(51));
    assert!(receipt.is_overdue(owed.day) && !owed.items[0].is_overdue(owed.day));
}

#[test]
fn nobody_owes_and_is_owed_nothing() {
    let company = support::books();
    let today = day("2024-06-30");
    // A name that is not there, and one of the other list.
    for name in [999, 201].map(NameId::new) {
        assert_eq!(company.owed_by(name, today).unwrap(), None);
    }
    for name in [999, 101].map(NameId::new) {
        assert_eq!(company.owed_to(name, today).unwrap(), None);
    }
    // One that is there, and owes nothing.
    let job = company.owed_by(NameId::new(103), today).unwrap().unwrap();
    assert!(job.items.is_empty() && job.home_total.is_zero());
}

#[test]
fn what_cannot_be_read_is_said() {
    let company = support::damaged_books();
    let receivable = company.open_items(Side::Receivable, AsOf::Latest).unwrap();
    assert_eq!(receivable.left_out.rows, 1);
    assert_eq!(lines(&receivable), [line(400, "202.00", "45.00")]);
}

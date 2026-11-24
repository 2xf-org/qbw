//! Open items of a small company: a job, a journal entry of two customers and
//! one that debits and credits the same one, an invoice in euros that is paid in
//! part, and a payment that is not used up.

use qbw::open_items::OpenItems;
use qbw::{AccountId, AsOf, Company, NameId, Side, TransactionId};

use crate::small_owing;
use crate::small_owing::{
    BUILDER, EURO, JOB, Line, Link, RECEIVABLE, RECEIVABLE_EUR, day, item, opened,
};

const LINES: &[Line] = &[
    // An invoice of the job, and one of the builder itself.
    Line::new(101, "2024-01-05", RECEIVABLE, JOB, "70.00").open("70.00"),
    Line::new(201, "2024-01-06", RECEIVABLE, BUILDER, "30.00").paid(),
    // A payment of 45, of which 30 paid the invoice before.
    Line::new(301, "2024-01-20", RECEIVABLE, BUILDER, "-45.00").open("-15.00"),
    // A journal entry of two customers.
    Line::new(401, "2024-02-01", RECEIVABLE, BUILDER, "8.00").paid(),
    Line::new(402, "2024-02-01", RECEIVABLE, JOB, "-3.00").open("-3.00"),
    // A payment of the builder's part of it.
    Line::new(501, "2024-02-10", RECEIVABLE, BUILDER, "-8.00").paid(),
    // An invoice in euros at 1.10, 40 of them paid, and a change of the rate.
    Line::new(601, "2024-03-01", RECEIVABLE_EUR, EURO, "100.00")
        .home("110.00")
        .open("60.00"),
    Line::new(701, "2024-03-10", RECEIVABLE_EUR, EURO, "-40.00")
        .home("-44.00")
        .paid(),
    Line::new(801, "2024-03-31", RECEIVABLE_EUR, EURO, "0.00")
        .home("5.00")
        .paid(),
];

/// Payments of invoices.
const LINKS: &[Link] = &[
    (4, 301, 201, "-30.00"),
    (4, 501, 401, "-8.00"),
    (4, 701, 601, "-40.00"),
];

fn books() -> Company {
    small_owing::company(LINES, LINKS)
}

fn receivable(company: &Company, as_of: AsOf) -> OpenItems {
    company.open_items(Side::Receivable, as_of).unwrap()
}

#[test]
fn what_is_kept_is_what_the_links_leave() {
    let company = books();
    let expected = [
        item(1, JOB, "70.00"),
        item(3, BUILDER, "-15.00"),
        item(4, JOB, "-3.00"),
        item(6, EURO, "60.00"),
    ];
    let latest = receivable(&company, AsOf::Latest);
    assert_eq!(opened(&latest), expected);
    let linked = receivable(&company, AsOf::Date(day("2024-12-31")));
    assert_eq!(opened(&linked), expected);
    assert_eq!(
        (latest.items, latest.adjustments),
        (linked.items, linked.adjustments)
    );
}

#[test]
fn a_journal_entry_of_two_customers_is_two_items() {
    let company = books();
    let before = receivable(&company, AsOf::Date(day("2024-02-05")));
    let expected = [
        item(1, JOB, "70.00"),
        item(3, BUILDER, "-15.00"),
        item(4, BUILDER, "8.00"),
        item(4, JOB, "-3.00"),
    ];
    assert_eq!(opened(&before), expected);
}

#[test]
fn a_part_in_another_currency_keeps_the_rate_of_its_invoice() {
    let company = books();
    let latest = receivable(&company, AsOf::Latest);
    let euros = latest.of(NameId::new(EURO)).next().unwrap();
    let amounts = [euros.total, euros.open, euros.home_open].map(|amount| amount.to_string());
    assert_eq!(amounts, ["100.00", "60.00", "66.00"]);
    // The change of the rate is of no item, and makes the rest of the balance.
    assert_eq!(latest.adjustments.to_string(), "5.00");
    let balance = company
        .ledger()
        .unwrap()
        .balance(AccountId::new(RECEIVABLE_EUR), AsOf::Latest)
        .unwrap();
    assert_eq!(balance.home_amount.to_string(), "71.00");
}

#[test]
fn a_customer_owes_what_its_jobs_owe() {
    let company = books();
    let owed = company
        .owed_by(NameId::new(BUILDER), day("2024-12-31"))
        .unwrap()
        .unwrap();
    let expected = [
        item(1, JOB, "70.00"),
        item(3, BUILDER, "-15.00"),
        item(4, JOB, "-3.00"),
    ];
    assert_eq!(opened(&owed.items), expected);
    assert_eq!(owed.home_total.to_string(), "52.00");
    let job = company
        .owed_by(NameId::new(JOB), day("2024-12-31"))
        .unwrap()
        .unwrap();
    assert_eq!(job.home_total.to_string(), "67.00");

    let balances = company
        .name_balances(Side::Receivable, AsOf::Latest)
        .unwrap();
    let rows: Vec<(i32, String, String)> = balances
        .rows
        .iter()
        .map(|row| {
            (
                row.name.number(),
                row.balance.home_amount.to_string(),
                row.total.to_string(),
            )
        })
        .collect();
    let row = |name, own: &str, total: &str| (name, own.to_owned(), total.to_owned());
    let expected = [
        row(BUILDER, "-15.00", "52.00"),
        row(JOB, "67.00", "67.00"),
        row(EURO, "71.00", "71.00"),
    ];
    assert_eq!(rows, expected);
    assert_eq!(balances.total.to_string(), "123.00");
}

#[test]
fn a_customer_that_owes_nothing_itself_is_listed_for_its_job() {
    let of_job = Line::new(101, "2024-01-05", RECEIVABLE, JOB, "70.00");
    let company = small_owing::company(&[of_job], &[]);
    let balances = company.name_balances(Side::Receivable, AsOf::Latest);
    let balances = balances.unwrap();
    let listed: Vec<i32> = balances.rows.iter().map(|row| row.name.number()).collect();
    assert_eq!(listed, [BUILDER, JOB]);
    let builder = balances.balance(NameId::new(BUILDER)).unwrap();
    assert!(builder.balance.amount.is_zero());
    assert_eq!(builder.total.to_string(), "70.00");
}

#[test]
fn a_link_to_a_header_that_is_not_there_is_left_out_and_counted() {
    let company = books();
    let links = company.links(TransactionId::new(3)).unwrap().unwrap();
    assert!(links.links.is_empty());
    // The company has no headers: not its own, nor the other one's.
    assert_eq!(links.left_out.rows, 2);
    let line_links = company.line_links(TransactionId::new(3)).unwrap().unwrap();
    assert_eq!((line_links.links.len(), line_links.left_out.rows), (1, 1));
}

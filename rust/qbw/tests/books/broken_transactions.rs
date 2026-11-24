//! Transactions that the books do not have, damaged in every way.

use qbw::query::TransactionQuery;
use qbw::transactions::{Transaction, TransactionKind};
use qbw::{Error, Page, Problem, TransactionId};

use crate::odd_forms::{Form, Row};
use crate::{forms, odd_forms};

fn header(id: &'static str) -> Row {
    Row::new(&[
        ("transaction_id", id),
        ("target_id", "1"),
        ("transaction_date", "2024-05-01"),
        ("transaction_view_type", "9"),
    ])
}

/// A line of transaction 100: its id, the next one, its account and its amount.
fn line(id: &'static str, next: &'static str, account: &'static str, amount: &'static str) -> Row {
    Row::new(&[
        ("transaction_id", "100"),
        ("target_id", id),
        ("next_target_id", next),
        ("account_id", account),
        ("amount_amt", amount),
        ("foreign_amount_amt", amount),
        ("is_source_bool", if id == "1" { "yes" } else { "no" }),
    ])
}

/// The transaction 100 with `lines`.
fn bill(lines: Vec<Row>) -> Transaction {
    let company = odd_forms::with_one("abmc_bill", header("100"), lines);
    forms::transaction(&company, 100)
}

fn problems(transaction: &Transaction) -> Vec<&Problem> {
    let damage = transaction.damage.iter();
    damage.map(|damage| &damage.problem).collect()
}

fn ids(transaction: &Transaction) -> Vec<i32> {
    let lines = transaction.lines.iter();
    lines.map(|line| line.id.number()).collect()
}

#[test]
fn a_transaction_without_its_own_line_has_no_total() {
    let lost = bill(vec![
        line("1", "2", "7", "-50.00").unreadable(),
        line("3", "0", "12", "20.00"),
        line("2", "3", "12", "30.00"),
    ]);
    assert_eq!(problems(&lost), [&Problem::OwnLine]);
    assert_eq!((lost.total, lost.home_total), (None, None));
    assert!(lost.own_line.is_none() && lost.account().is_none());
    assert_eq!((lost.open(), lost.is_paid()), (None, None));
    assert!(!lost.is_overdue("2030-01-01".parse().unwrap()));
    // Nothing leads to the lines, so they are in the order of their ids.
    assert_eq!(ids(&lost), [2, 3]);
    // There is not even a table of lines.
    let nothing = bill(Vec::new());
    assert_eq!(problems(&nothing), [&Problem::OwnLine]);
    assert_eq!(nothing.lines, []);
}

#[test]
fn a_line_whose_value_makes_no_sense_is_the_damage() {
    let read = bill(vec![
        line("1", "2", "7", "-50.00"),
        // The file keeps five digits after the point.
        line("2", "3", "12", "30.000001"),
        line("3", "0", "12", "20.00"),
    ]);
    let [Problem::Column { column, .. }] = problems(&read)[..] else {
        panic!("the amount is the damage: {:?}", read.damage);
    };
    assert_eq!(*column, "amount_amt");
    assert_eq!(read.left_out().rows, 1);
    assert_eq!(ids(&read), [3]);
    assert_eq!(forms::money(read.total), "50.00 (0)");
}

#[test]
fn a_ring_of_lines_ends() {
    let ring = bill(vec![
        line("1", "2", "7", "-50.00"),
        line("2", "3", "12", "30.00"),
        line("3", "2", "12", "20.00"),
    ]);
    assert_eq!(problems(&ring), [&Problem::LineChain]);
    assert_eq!(ids(&ring), [2, 3]);
    let own_ring = bill(vec![line("1", "1", "7", "0.00")]);
    assert_eq!(problems(&own_ring), [&Problem::LineChain]);
}

#[test]
fn lines_that_nothing_leads_to_come_last() {
    let read = bill(vec![
        line("9", "0", "12", "5.00"),
        line("1", "4", "7", "-50.00"),
        line("6", "0", "12", "15.00"),
        line("4", "0", "12", "30.00"),
    ]);
    assert_eq!(problems(&read), [&Problem::LineChain]);
    assert_eq!(ids(&read), [4, 6, 9]);
    let last_reached = read.lines[0].origin;
    assert_eq!(read.damage[0].row, Some(last_reached.row));
    assert_eq!(read.damage[0].table, "abmc_bill_lineitem");
}

#[test]
fn lines_in_two_currencies_are_said() {
    let foreign = Row::new(&[
        ("transaction_id", "100"),
        ("target_id", "2"),
        ("account_id", "12"),
        ("currency_id", "2"),
        ("amount_amt", "50.00"),
        ("foreign_amount_amt", "40.00"),
    ]);
    let read = bill(vec![line("1", "2", "7", "-50.00"), foreign]);
    assert_eq!(problems(&read), [&Problem::Currencies]);
    assert_eq!(forms::money(read.lines[0].amount), "40.00 (2)");
}

#[test]
fn a_header_is_lost_among_others() {
    let lines = vec![line("1", "0", "7", "0.00")];
    let headers = vec![
        header("100").unreadable(),
        header("200").unreadable(),
        header("300"),
    ];
    let company = odd_forms::company(&[Form {
        tables: "abmc_bill",
        headers,
        lines,
    }]);
    let error = company.transaction(TransactionId::new(100)).unwrap_err();
    let Error::Damaged(damage) = error else {
        panic!("the header is damaged: {error}");
    };
    // Nothing tells which of the two it is.
    assert_eq!((damage.row, &damage.problem), (None, &Problem::Header));
    let message = "table abmc_bill_header: the transaction's header cannot be read";
    assert_eq!(damage.to_string(), message);
    // Nothing says that there was a transaction 200 either.
    assert_eq!(company.transaction(TransactionId::new(200)).unwrap(), None);

    let bills = TransactionQuery::new().kinds([TransactionKind::Bill]);
    let page = company.transactions(&bills, Page::all()).unwrap();
    assert_eq!((page.row_count, page.left_out.rows), (1, 2));
    assert_eq!((page.rows[0].account, page.rows[0].total), (None, None));
}

#[test]
fn a_header_without_a_day_is_damaged() {
    let dayless = Row::new(&[("transaction_id", "100"), ("target_id", "1")]);
    let lines = vec![line("1", "0", "7", "0.00")];
    let company = odd_forms::with_one("abmc_invoice", dayless, lines);
    let error = company.transaction(TransactionId::new(100)).unwrap_err();
    let Error::Damaged(damage) = error else {
        panic!("the header is damaged: {error}");
    };
    let Problem::Column { column, .. } = damage.problem else {
        panic!("the day is the problem: {damage}");
    };
    assert_eq!(column, "transaction_date");
}

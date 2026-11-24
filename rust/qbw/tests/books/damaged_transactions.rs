//! Transactions of books that cannot be read in full.

use qbw::query::TransactionQuery;
use qbw::transactions::{Transaction, TransactionKind};
use qbw::{Company, Damage, Error, Page, Problem, TransactionId};

use crate::{forms, support};

/// The ids of the lines, and what is wrong with them.
fn lines(transaction: &Transaction) -> (Vec<i32>, Vec<&Problem>) {
    let ids = transaction.lines.iter().map(|line| line.id.number());
    let problems = transaction.damage.iter().map(|damage| &damage.problem);
    (ids.collect(), problems.collect())
}

#[test]
fn a_transaction_is_given_without_a_line_that_cannot_be_read() {
    let company = support::damaged_books();
    let invoice = forms::transaction(&company, 400);
    // The line 403 is lost, which the chain shows.
    let expected = (vec![402, 404, 405], vec![&Problem::LineChain]);
    assert_eq!(lines(&invoice), expected);
    assert_eq!(invoice.left_out().rows, 1);
    assert_eq!(forms::money(invoice.total), "202.00 (1)");
    assert_eq!(forms::money(invoice.subtotal()), "150.00 (1)");
    let last_reached = invoice.damage[0].row;
    assert_eq!(last_reached, Some(invoice.lines[0].origin.row));
}

#[test]
fn a_broken_chain_is_said_and_its_lines_are_given() {
    let company = support::damaged_books();
    let bill = forms::transaction(&company, 200);
    let expected = (vec![202], vec![&Problem::LineChain]);
    assert_eq!(lines(&bill), expected);
    let own = bill.own_line.as_ref().unwrap();
    assert_eq!(bill.damage[0].row, Some(own.origin.row));
    assert_eq!(forms::lines(&bill), ["Item 6 502 10 20.00 200.00"]);
}

#[test]
fn a_transaction_whose_header_cannot_be_read_is_an_error() {
    let company = support::damaged_books();
    let error = company.transaction(TransactionId::new(1000)).unwrap_err();
    let Error::Damaged(Damage { table, problem, .. }) = error else {
        panic!("the header is damaged: {error}");
    };
    assert_eq!(table, "abmc_credit_cardcharge_header");
    assert!(matches!(problem, Problem::Row { .. }));
    // One that has no line either is not there.
    assert_eq!(company.transaction(TransactionId::new(7)).unwrap(), None);
}

#[test]
fn the_others_are_whole() {
    let company = support::damaged_books();
    for id in [700, 800, 1100, 1200, 1800] {
        let read = forms::transaction(&company, id);
        assert!(read.left_out().is_nothing(), "transaction {id}");
    }
}

#[test]
fn a_page_says_how_much_is_left_out_of_it() {
    let company = support::damaged_books();
    let rows_left_out = |query: &TransactionQuery| {
        let page = company.transactions(query, Page::new(0, 50)).unwrap();
        (page.row_count, page.left_out.rows)
    };
    // The header of the card charge, and a line of an invoice.
    assert_eq!(rows_left_out(&TransactionQuery::new()), (23, 2));
    let of = |kind| TransactionQuery::new().kinds([kind]);
    assert_eq!(rows_left_out(&of(TransactionKind::Invoice)), (3, 1));
    assert_eq!(
        rows_left_out(&of(TransactionKind::CreditCardCharge)),
        (0, 1)
    );
    assert_eq!(
        rows_left_out(&of(TransactionKind::CreditCardCredit)),
        (1, 1)
    );
    assert_eq!(rows_left_out(&of(TransactionKind::Deposit)), (1, 0));
    assert_eq!(rows_left_out(&of(TransactionKind::Bill)), (1, 0));
}

#[test]
fn a_damaged_header_is_not_found() {
    let company: Company = support::damaged_books();
    let found = company.find(TransactionKind::CreditCardCharge, "").unwrap();
    assert_eq!(found, None);
    let credit = company.find(TransactionKind::CreditMemo, "CM-3").unwrap();
    assert_eq!(credit, Some(TransactionId::new(700)));
}

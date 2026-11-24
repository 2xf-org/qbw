//! What paid, credited, deposited and received what.

use qbw::links::{LineLink, Link, LinkCode, Links, Relation};
use qbw::transactions::TransactionKind;
use qbw::{CurrencyId, LineId, NameId, TransactionId};

use crate::changed::{Changed, number, whole};
use crate::{counted, support};

fn links(transaction: i32) -> Links {
    let company = support::books();
    company
        .links(TransactionId::new(transaction))
        .unwrap()
        .unwrap()
}

/// The relation, the other transaction and the amount of each link.
fn lines<'links>(links: impl IntoIterator<Item = &'links Link>) -> Vec<(Relation, i32, String)> {
    let links = links.into_iter().map(|link| {
        let amount = link.amount.map(|amount| amount.to_string());
        (
            link.relation,
            link.other.id.number(),
            amount.unwrap_or_default(),
        )
    });
    links.collect()
}

fn line(relation: Relation, other: i32, amount: &str) -> (Relation, i32, String) {
    (relation, other, amount.to_owned())
}

#[test]
fn an_invoice_was_paid_by_payments_credits_and_discounts() {
    let invoice = links(400);
    assert_eq!(
        lines(&invoice),
        [
            line(Relation::PaidBy, 500, "100.00"),
            line(Relation::CreditedBy, 700, "54.00"),
            line(Relation::DiscountFrom, 500, "2.00"),
            line(Relation::DiscountFrom, 2400, "1.00"),
        ]
    );
    assert!(invoice.left_out.is_nothing());
    let payment = invoice.of(Relation::PaidBy).next().unwrap();
    assert_eq!(payment.other.kind, TransactionKind::Payment);
    assert_eq!(payment.other.date, "2024-01-20".parse().unwrap());
    assert_eq!(payment.other.number.as_deref(), Some("88"));
    assert_eq!(payment.other.name, Some(NameId::new(101)));
    assert_eq!(payment.quantity, None);
}

#[test]
fn a_payment_paid_invoices_and_was_deposited() {
    assert_eq!(
        lines(&links(500)),
        [
            line(Relation::AppliedTo, 400, "100.00"),
            line(Relation::DiscountOn, 400, "2.00"),
            line(Relation::DepositedIn, 600, "100.00"),
        ]
    );
    assert_eq!(
        lines(&links(600)),
        [line(Relation::DepositOf, 500, "100.00")]
    );
    assert_eq!(
        lines(&links(700)),
        [line(Relation::AppliedTo, 400, "54.00")]
    );
}

#[test]
fn one_call_says_what_paid_and_what_was_paid() {
    let company = support::books();
    let paid = company.paid_by(TransactionId::new(400)).unwrap().unwrap();
    assert_eq!(
        (paid.links.len(), paid.total.to_string()),
        (4, "157.00".to_owned())
    );
    let applied = company
        .applied_to(TransactionId::new(500))
        .unwrap()
        .unwrap();
    assert_eq!(
        lines(&applied),
        [
            line(Relation::AppliedTo, 400, "100.00"),
            line(Relation::DiscountOn, 400, "2.00"),
        ]
    );
    assert_eq!(applied.total.to_string(), "102.00");
    assert!(
        company
            .paid_by(TransactionId::new(500))
            .unwrap()
            .unwrap()
            .links
            .is_empty()
    );
    assert!(
        company
            .applied_to(TransactionId::new(400))
            .unwrap()
            .unwrap()
            .links
            .is_empty()
    );
    assert_eq!(
        lines(&company.paid_by(TransactionId::new(200)).unwrap().unwrap()),
        [line(Relation::PaidBy, 300, "200.00")]
    );
}

#[test]
fn amounts_are_in_the_transaction_s_currency() {
    let invoice = links(800);
    let euros = CurrencyId::new(2);
    assert_eq!(lines(&invoice), [line(Relation::PaidBy, 900, "100.00")]);
    let payment = invoice.iter().next().unwrap().amount.unwrap();
    assert_eq!((payment.currency(), invoice.currency), (euros, euros));
    assert_eq!(links(400).currency, CurrencyId::new(1));
}

#[test]
fn an_order_says_what_was_received_of_it() {
    let order = links(1400);
    let [received] = order.iter().as_slice() else {
        panic!("one receipt fills the order: {order:?}");
    };
    assert_eq!(
        (received.relation, received.other.id),
        (Relation::ReceivedOn, TransactionId::new(1500))
    );
    assert_eq!(
        (received.amount, received.quantity),
        (None, "2".parse().ok())
    );
    let receipt = links(1500);
    let ordered = receipt.of(Relation::OrderedOn).next().unwrap();
    assert_eq!(ordered.other.number.as_deref(), Some("P-12"));
}

#[test]
fn a_line_is_linked_to_a_line() {
    let company = support::books();
    let received = company
        .line_links(TransactionId::new(1500))
        .unwrap()
        .unwrap();
    let [link] = received.links.as_slice() else {
        panic!("one line fills the order: {received:?}");
    };
    let LineLink {
        line,
        other_line,
        code,
        is_source,
        quantity,
        ..
    } = *link;
    assert_eq!(
        (line, other_line),
        (Some(LineId::new(1502)), Some(LineId::new(1402)))
    );
    assert_eq!(
        (code, is_source, quantity),
        (LinkCode::Receipt, true, "2".parse().ok())
    );

    let payment = company
        .line_links(TransactionId::new(500))
        .unwrap()
        .unwrap();
    let parts: Vec<(i32, String)> = payment
        .links
        .iter()
        .filter(|link| link.code == LinkCode::Payment)
        .map(|link| {
            (
                link.other_line.unwrap().number(),
                link.amount.unwrap().to_string(),
            )
        })
        .collect();
    let parts_expected = [(402, "74.00"), (403, "20.00"), (405, "6.00")];
    assert_eq!(
        parts,
        parts_expected.map(|(line, amount)| (line, amount.to_owned()))
    );
}

#[test]
fn what_quickbooks_does_not_show_is_not_linked() {
    // The bill has its payment once more for information, and the invoice a
    // line that is linked to itself and a credit that is written twice.
    assert_eq!(lines(&links(200)), [line(Relation::PaidBy, 300, "200.00")]);
    let company = support::books();
    let invoice = company
        .line_links(TransactionId::new(400))
        .unwrap()
        .unwrap();
    assert!(
        invoice
            .links
            .iter()
            .all(|link| link.other != TransactionId::new(400))
    );
    let credits = invoice.links.iter();
    let credits = credits.filter(|link| link.code == LinkCode::Credit);
    assert_eq!(credits.count(), 2);
}

#[test]
fn the_kind_of_a_transaction_says_which_of_two_refunds_the_other() {
    // Check 501 refunds the credit memo.
    let link: &[(&str, _)] = &[
        ("link_id", whole(901)),
        ("transaction_id", whole(700)),
        ("transaction_2_id", whole(300)),
        ("link_type", whole(2)),
        ("amount_amt", number("-5.00")),
    ];
    let books = Changed::books().added("abmc_transaction_link", link);
    let mirror: &[(&str, _)] = &[
        ("link_id", whole(902)),
        ("transaction_id", whole(300)),
        ("transaction_2_id", whole(700)),
        ("link_type", whole(2)),
        ("amount_amt", number("-5.00")),
    ];
    let company = books.added("abmc_transaction_link", mirror).company();
    let relation = |of, with| {
        let links = company.links(TransactionId::new(of)).unwrap().unwrap();
        let mut links = links.iter();
        let link = links.find(|link| link.other.id == TransactionId::new(with));
        link.unwrap().relation
    };
    assert_eq!(relation(700, 300), Relation::RefundedBy);
    assert_eq!(relation(300, 700), Relation::RefundOf);
}

#[test]
fn a_link_has_the_header_of_the_other_transaction() {
    let company = support::books();
    let links = company.links(TransactionId::new(500)).unwrap().unwrap();
    for link in &links {
        let other = company.transaction(link.other.id).unwrap().unwrap();
        assert_eq!(link.other.origin, other.origin);
    }
    let tables: Vec<_> = links.iter().map(|link| link.other.origin.table).collect();
    assert!(tables.contains(&"abmc_invoice_header"), "{tables:?}");
}

#[test]
fn a_transaction_without_links_has_none() {
    for transaction in [100, 1700] {
        assert!(
            links(transaction).links.is_empty(),
            "transaction {transaction}"
        );
    }
    let company = support::books();
    let nothing = TransactionId::new(99_999);
    assert_eq!(company.links(nothing).unwrap(), None);
    assert_eq!(company.paid_by(nothing).unwrap(), None);
    assert_eq!(company.applied_to(nothing).unwrap(), None);
    assert_eq!(company.line_links(nothing).unwrap(), None);
}

#[test]
fn the_links_of_a_transaction_take_one_row_however_many_they_are() {
    let company = counted::books();
    let links = company.links(TransactionId::new(400)).unwrap().unwrap();
    assert!(links.links.len() > 1);
    // The header of the transaction, which has its currency.
    assert_eq!(company.source().fetched(), 1);
}

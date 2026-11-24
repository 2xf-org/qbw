//! What is left of a payment, which is on the line of what it pays.

use qbw::check::{CheckKind, Subject};
use qbw::query::{PaidState, TransactionQuery};
use qbw::sa17::Value;
use qbw::transactions::{OpenState, TransactionKind};
use qbw::{AsOf, Company, Page, Side, TransactionId};

use crate::changed::{Changed, Key, number, whole};

/// Payment 88: 100 of it on the receivable, and 2 of discount.
const PAYMENT: TransactionId = TransactionId::new(500);
const DEPOSITED: Key = ("abmc_received_payment_lineitem", "target_id", 501);
const APPLIED: Key = ("abmc_received_payment_lineitem", "target_id", 502);
/// The check that pays bill 77.
const BILL_PAYMENT: TransactionId = TransactionId::new(300);
const PAID_FROM: Key = ("abmc_bill_check_lineitem", "target_id", 301);
const PAYS: Key = ("abmc_bill_check_lineitem", "target_id", 302);

/// A line of which so much is left.
fn left(books: Changed, line: Key, open: &str) -> Changed {
    books.set(line, "is_paid_bool", Value::Boolean(false)).set(
        line,
        "open_balance_amt",
        number(open),
    )
}

fn open(company: &Company<Changed>, id: TransactionId) -> (Option<String>, Option<bool>) {
    let read = company.transaction(id).unwrap().unwrap();
    let summary = company.transaction_summary(id).unwrap().unwrap();
    assert_eq!(summary.open, read.open_state);
    assert_eq!(summary.is_paid(), read.is_paid());
    let open = read.open().map(|open| open.to_string());
    (open, read.is_paid())
}

/// The payments that a question for unpaid transactions finds.
fn unpaid(company: &Company<Changed>) -> Vec<TransactionId> {
    let kinds = [TransactionKind::Payment, TransactionKind::BillPaymentCheck];
    let query = TransactionQuery::new().kinds(kinds).paid(PaidState::Unpaid);
    let page = company.transactions(&query, Page::all()).unwrap();
    page.rows.iter().map(|row| row.id).collect()
}

/// What the open items have of a transaction.
fn items(company: &Company<Changed>, side: Side, id: TransactionId) -> Vec<String> {
    let open = company.open_items(side, AsOf::Latest).unwrap();
    let items = open.iter().filter(|item| item.transaction == id);
    items.map(|item| item.open.to_string()).collect()
}

#[test]
fn what_is_left_of_a_payment_is_on_the_receivable() {
    let books = left(Changed::books(), APPLIED, "-40").company();
    assert_eq!(open(&books, PAYMENT), (Some("40.00".into()), Some(false)));
    assert_eq!(unpaid(&books), [PAYMENT]);
    assert_eq!(items(&books, Side::Receivable, PAYMENT), ["-40.00"]);
}

#[test]
fn what_is_left_of_a_bill_payment_is_on_the_payable() {
    let books = left(Changed::books(), PAYS, "40").company();
    let some = (Some("40.00".into()), Some(false));
    assert_eq!(open(&books, BILL_PAYMENT), some);
    assert_eq!(unpaid(&books), [BILL_PAYMENT]);
    assert_eq!(items(&books, Side::Payable, BILL_PAYMENT), ["-40.00"]);
}

#[test]
fn the_line_of_the_bank_says_nothing_of_it() {
    // QuickBooks leaves what it likes on the line that the money goes to.
    let books = left(Changed::books(), DEPOSITED, "100");
    let books = left(books, PAID_FROM, "-50").company();
    for (id, side) in [(PAYMENT, Side::Receivable), (BILL_PAYMENT, Side::Payable)] {
        assert_eq!(open(&books, id), (Some("0.00".into()), Some(true)));
        assert!(items(&books, side, id).is_empty());
    }
    assert!(unpaid(&books).is_empty());
    let check = books.check().unwrap();
    let is_of_payments = |subject: &Subject| {
        [PAYMENT, BILL_PAYMENT]
            .map(Subject::Transaction)
            .contains(subject)
    };
    assert!(
        !check
            .findings
            .iter()
            .any(|found| is_of_payments(&found.subject))
    );
}

#[test]
fn the_check_compares_what_a_form_has_left_with_its_links() {
    let books = left(Changed::books(), APPLIED, "-40").company();
    let check = books.check().unwrap();
    let mut found = check.findings.iter();
    let form = found
        .find(|found| found.what == CheckKind::OpenForm)
        .unwrap();
    assert_eq!(form.subject, Subject::Transaction(PAYMENT));
    let figures = [form.computed, form.expected].map(|figure| figure.unwrap().to_string());
    assert_eq!(figures, ["40.00", "0.00"]);
    assert!(!form.is_explained());
}

#[test]
fn a_payment_without_a_line_on_the_receivable_keeps_nothing() {
    let account = whole(12);
    let books = Changed::books()
        .set(APPLIED, "account_id", account.clone())
        .set(
            ("abmc_received_payment_lineitem", "target_id", 503),
            "account_id",
            account,
        )
        .company();
    let payment = books.transaction(PAYMENT).unwrap().unwrap();
    assert_eq!(payment.open_state, OpenState::NotTracked);
    assert_eq!((payment.open(), payment.is_paid()), (None, None));
}

#[test]
fn a_line_that_comes_twice_is_left_once() {
    let books = left(Changed::books(), APPLIED, "-40");
    let books = books
        .copied(
            APPLIED,
            &[
                ("is_paid_bool", Value::Boolean(false)),
                ("open_balance_amt", number("-40")),
            ],
        )
        .copied(DEPOSITED, &[("amount_amt", number("7"))])
        .company();
    let payments = TransactionQuery::new().kinds([TransactionKind::Payment]);
    let page = books.transactions(&payments, Page::all()).unwrap();
    let payment = page.rows.iter().find(|row| row.id == PAYMENT).unwrap();
    assert!(matches!(payment.open, OpenState::Open(left) if left.to_string() == "40.00"));
    assert_eq!(payment.amount.unwrap().to_string(), "100.00");
    assert_eq!(page.left_out.rows, 2);
}

#[test]
fn something_can_be_left_of_a_second_own_line() {
    let line = ("abmc_general_journal_lineitem", "target_id", 2502);
    let books = Changed::books()
        .set(line, "account_id", whole(3))
        .set(line, "is_source_bool", Value::Boolean(true))
        .company();
    let journal = TransactionId::new(2500);
    let read = books.transaction(journal).unwrap().unwrap();
    let row = books.transaction_summary(journal).unwrap().unwrap();
    assert!(matches!(read.open_state, OpenState::Open(_)));
    assert_eq!(row.open, read.open_state);
}

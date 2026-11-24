//! What posts of a transaction, and what is left of it.

use qbw::chrono::NaiveDate;
use qbw::open_items::OpenItems;
use qbw::sa17::{Source, Value};
use qbw::transactions::{Details, LastAction, LineRole, OpenState, PostingState};
use qbw::{
    AddressId, Company, CurrencyId, LineId, NameId, PaymentMethodId, Period, SalesRepId, Side,
    TermsId, TransactionId,
};

use crate::changed::{Changed, date, number, whole};
use crate::{forms, support};

fn day(text: &str) -> NaiveDate {
    text.parse().unwrap()
}

/// What is left of the transactions `ids`: the amount, and whether that is
/// nothing.
fn left<const N: usize>(ids: [i32; N]) -> [(String, Option<bool>); N] {
    let company = support::books();
    ids.map(|id| {
        let read = forms::transaction(&company, id);
        (forms::money(read.open()), read.is_paid())
    })
}

fn is(amount: &str, is_paid: Option<bool>) -> (String, Option<bool>) {
    (amount.to_owned(), is_paid)
}

#[test]
fn what_is_left_is_what_the_file_keeps() {
    // An invoice that is partly paid, one that is paid in euros, a credit that
    // is used up, payments that are, and a bill that is paid.
    let paid = left([400, 800, 700, 500, 2400, 200, 300]);
    assert_eq!(
        paid,
        [
            is("45.00 (1)", Some(false)),
            is("0.00 (2)", Some(true)),
            is("0.00 (1)", Some(true)),
            is("0.00 (1)", Some(true)),
            is("0.00 (1)", Some(true)),
            is("0.00 (1)", Some(true)),
            is("0.00 (1)", Some(true)),
        ]
    );
    // Without an amount in the file, all of a vendor credit and of a receipt.
    let unpaid = left([1200, 1500]);
    let expected = [is("20.00 (1)", Some(false)), is("40.00 (1)", Some(false))];
    assert_eq!(unpaid, expected);
}

#[test]
fn nothing_is_left_of_what_is_neither_paid_nor_used() {
    // A journal entry, a deposit, a transfer, orders, an estimate, a check and
    // a card charge, and an invoice that is pending.
    let left = left([100, 600, 1300, 1400, 1600, 2000, 1700, 1000, 1800]);
    assert!(left.iter().all(|left| *left == is("-", None)), "{left:?}");
}

#[test]
fn an_invoice_is_overdue_after_its_day() {
    let company = support::books();
    let invoice = forms::transaction(&company, 400);
    assert!(!invoice.is_overdue(day("2023-12-15")));
    assert!(invoice.is_overdue(day("2023-12-16")));
    // Paid, a credit, and pending.
    for id in [800, 200, 700, 1200, 1800] {
        let read = forms::transaction(&company, id);
        assert!(!read.is_overdue(day("2030-01-01")), "transaction {id}");
    }
}

#[test]
fn a_transaction_posts_or_does_not() {
    let company = support::books();
    let posting = |id| forms::transaction(&company, id).posting;
    assert_eq!(posting(400), PostingState::Posted);
    assert_eq!(posting(1400), PostingState::NonPosting);
    assert_eq!(posting(1600), PostingState::NonPosting);
    assert_eq!(posting(2000), PostingState::NonPosting);
    assert_eq!(posting(1800), PostingState::Pending);
    assert_eq!(posting(1900), PostingState::Memorized);
    for id in [1400, 1600, 1800, 1900, 2000] {
        let read = forms::transaction(&company, id);
        assert_eq!(forms::posted(&read), [""; 0], "transaction {id}");
    }
}

#[test]
fn what_lines_post_is_the_other_side_of_the_own_one() {
    let company = support::books();
    let posted = |id| forms::posted(&forms::transaction(&company, id));
    assert_eq!(
        posted(400),
        ["13 -150.00 -150.00", "12 -40.00 -40.00", "9 -12.00 -12.00"]
    );
    assert_eq!(posted(1100), ["16 -5.00 -5.00"]);
    assert_eq!(posted(2100), ["6 20.00 20.00"]);
    // The own side is a line here, and posts once.
    assert_eq!(posted(2400), ["3 0.00 0.00", "18 1.00 1.00"]);
}

#[test]
fn a_voided_transaction_keeps_its_place() {
    let company = support::books();
    let check = forms::transaction(&company, 1700);
    assert!(check.is_voided());
    assert_eq!(check.last_action, LastAction::Voided);
    assert_eq!(check.posting, PostingState::Posted);
    assert_eq!(forms::money(check.total), "0.00 (1)");
    assert_eq!(check.memo.as_deref(), Some("VOID: Fuel"));
    assert!(!forms::transaction(&company, 400).is_voided());
}

#[test]
fn a_payment_without_money_is_a_discount() {
    let company = support::books();
    let payment = forms::transaction(&company, 2400);
    assert!(!payment.payment().unwrap().moved_money);
    assert_eq!(forms::money(payment.total), "0.00 (1)");
    assert_eq!(forms::money(payment.home_total), "0.00 (1)");
    let own = payment.own_line.as_ref().unwrap();
    assert_eq!(own.id, payment.lines[0].id);
    assert_eq!(payment.lines[0].role, LineRole::DiscountTaken);
    assert_eq!(payment.lines[0].posted(), None);
}

#[test]
fn a_foreign_invoice_has_amounts_in_both_currencies() {
    let company = support::books();
    let invoice = forms::transaction(&company, 800);
    assert_eq!(invoice.currency, CurrencyId::new(2));
    assert_eq!(invoice.exchange_rate.to_string(), "1.1");
    assert_eq!(forms::money(invoice.home_total), "110.00 (1)");
    let own = invoice.own_line.as_ref().unwrap();
    assert_eq!(forms::money(own.home_amount), "110.00 (1)");
    assert_eq!(forms::money(invoice.lines[0].home_amount), "110.00 (1)");
    assert_eq!(forms::posted(&invoice), ["12 -100.00 -110.00"]);
}

#[test]
fn an_exchange_difference_is_not_on_the_form() {
    let company = support::books();
    let payment = forms::transaction(&company, 900);
    let on_form: Vec<LineId> = payment.form_lines().map(|line| line.id).collect();
    assert_eq!(on_form, [LineId::new(902)]);
    let difference = &payment.lines[1];
    assert_eq!(difference.role, LineRole::ExchangeDifference);
    assert_eq!(forms::money(difference.home_amount), "10.00 (1)");
    assert_eq!(
        forms::posted(&payment),
        ["4 -100.00 -110.00", "17 0.00 -10.00"]
    );
}

#[test]
fn a_transfer_in_the_home_currency_keeps_to_it() {
    let company = support::books();
    let transfer = forms::transaction(&company, 1300);
    assert_eq!(transfer.currency, CurrencyId::new(1));
    assert_eq!(transfer.exchange_rate.to_string(), "1.1");
    assert_eq!(forms::money(transfer.home_total), "55.00 (1)");
    // The euro account gets 50, which is not the line's amount.
    assert_eq!(forms::money(transfer.lines[0].amount), "55.00 (1)");
    assert_eq!(forms::posted(&transfer), ["2 55.00 55.00"]);
}

#[test]
fn an_adjustment_of_the_home_currency_moves_no_euros() {
    let company = support::books();
    let adjustment = forms::transaction(&company, 2500);
    let Details::Journal(journal) = adjustment.details else {
        panic!("a journal entry has the details of one");
    };
    assert!(journal.is_home_currency_adjustment && journal.is_shown_in_home_currency);
    assert!(journal.is_adjustment);
    assert_eq!(
        forms::money(adjustment.own_line.unwrap().amount),
        "5.00 (1)"
    );
    let Details::Journal(opening) = forms::transaction(&company, 100).details else {
        panic!("a journal entry has the details of one");
    };
    assert!(!opening.is_home_currency_adjustment && !opening.is_adjustment);
}

/// What the form of a transaction has in its columns for debits and credits.
fn columns<S: Source>(company: &Company<S>, id: i32) -> Vec<[String; 2]> {
    let form = company
        .transaction(TransactionId::new(id))
        .unwrap()
        .unwrap();
    let lines = form.form_lines();
    let columns = lines.map(|line| [line.debit(), line.credit()].map(forms::money));
    columns.collect()
}

#[test]
fn a_journal_entry_has_debits_and_credits() {
    let company = support::books();
    let opening = [["1000.00 (1)", "-"], ["-", "1000.00 (1)"]];
    assert_eq!(columns(&company, 100), opening);
    assert_eq!(
        columns(&company, 2500),
        [["5.00 (1)", "-"], ["-", "5.00 (1)"]]
    );
    // No line of an invoice, a check or a credit memo has either.
    for id in [400, 300, 700] {
        let columns = columns(&company, id);
        assert!(columns.iter().all(|columns| *columns == ["-", "-"]), "{id}");
        assert!(!columns.is_empty());
    }
}

#[test]
fn a_journal_entry_shows_one_of_its_currencies() {
    let line = |id| ("abmc_general_journal_lineitem", "target_id", id);
    let header = ("abmc_general_journal_header", "transaction_id", 100);
    let euros = || {
        Changed::books()
            .set(header, "currency_id", whole(2))
            .set(line(101), "currency_id", whole(2))
            .set(line(102), "currency_id", whole(2))
            .set(line(101), "foreign_amount_amt", number("900.00"))
            .set(line(102), "foreign_amount_amt", number("-900.00"))
    };
    let in_euros = [["900.00 (2)", "-"], ["-", "900.00 (2)"]];
    assert_eq!(columns(&euros().company(), 100), in_euros);
    let in_home = euros().set(
        header,
        "is_shown_in_home_currency_bool",
        Value::Boolean(true),
    );
    let in_dollars = [["1000.00 (1)", "-"], ["-", "1000.00 (1)"]];
    assert_eq!(columns(&in_home.company(), 100), in_dollars);
}

#[test]
fn a_line_of_a_journal_entry_without_an_amount_has_neither() {
    let line = ("abmc_general_journal_lineitem", "target_id", 102);
    let books = Changed::books().set(line, "amount_amt", Value::Null).set(
        line,
        "foreign_amount_amt",
        Value::Null,
    );
    assert_eq!(
        columns(&books.company(), 100),
        [["1000.00 (1)", "-"], ["-", "-"]]
    );
}

#[test]
fn an_order_knows_how_far_it_is_filled() {
    let company = support::books();
    let order = forms::transaction(&company, 1400);
    let progress = order.lines[0].order.unwrap();
    assert_eq!(progress.open, Some(3.into()));
    assert_eq!(progress.filled, Some(2.into()));
    assert!(!progress.is_done && !progress.is_closed);
    assert!(!order.is_fully_received() && !order.is_closed());
    // Nothing of it was invoiced.
    let order = forms::transaction(&company, 1600);
    let progress = order.lines[0].order.unwrap();
    assert_eq!(progress.open, Some(4.into()));
    assert_eq!(progress.filled, Some(0.into()));
    assert!(!order.is_fully_invoiced() && !order.is_fully_received());
    assert_eq!(order.customer(), Some(NameId::new(103)));
    // An estimate and a bill are no orders.
    for id in [2000, 200] {
        let read = forms::transaction(&company, id);
        assert!(read.lines.iter().all(|line| line.order.is_none()));
        assert!(!read.is_closed());
    }
}

#[test]
fn the_members_of_a_group_lead_to_its_start() {
    let company = support::books();
    let estimate = forms::transaction(&company, 2000);
    let groups: Vec<Option<LineId>> = estimate.lines.iter().map(|line| line.group).collect();
    let group = Some(LineId::new(2002));
    assert_eq!(groups, [None, group, group, group, None, None]);
    assert_eq!(estimate.item_lines().count(), 3);
    assert_eq!(estimate.expense_lines().count(), 0);
    assert_eq!(estimate.form_lines().count(), 6);
    assert_eq!(
        forms::transaction(&company, 1000).expense_lines().count(),
        1
    );
}

#[test]
fn open_states_tell_whether_they_are_paid() {
    let company = support::books();
    let open = |id| forms::transaction(&company, id).open_state;
    assert_eq!(open(800).is_paid(), Some(true));
    assert_eq!(open(400).is_paid(), Some(false));
    assert_eq!(open(100), OpenState::NotTracked);
}

#[test]
fn a_form_has_the_names_of_what_it_shows() {
    let company = support::books();
    let invoice = forms::transaction(&company, 400);
    assert_eq!(invoice.terms(), Some(TermsId::new(1)));
    assert_eq!(invoice.terms_name.as_deref(), Some("2% 10 Net 30"));
    assert_eq!(invoice.sales_rep(), Some(SalesRepId::new(1)));
    assert_eq!(invoice.sales_rep_initials.as_deref(), Some("EM"));
    assert_eq!(invoice.ship_to(), AddressId::try_new("SHIPTO0000000400"));
    assert_eq!(invoice.payment_method(), None);
    let payment = forms::transaction(&company, 500);
    assert_eq!(payment.payment_method(), Some(PaymentMethodId::new(1)));
    assert_eq!((payment.terms(), payment.terms_name), (None, None));
    let bill = forms::transaction(&company, 200);
    assert_eq!(bill.terms_name.as_deref(), Some("2% 10 Net 30"));
    assert_eq!((bill.sales_rep(), bill.ship_to()), (None, None));
}

#[test]
fn the_own_line_of_a_journal_entry_is_the_first_on_its_form() {
    let company = support::books();
    let journal = forms::transaction(&company, 100);
    let lines: Vec<_> = journal.form_lines().collect();
    let [own, other] = lines[..] else {
        panic!("an entry of two lines: {lines:?}");
    };
    let own_line = journal.own_line.as_ref().unwrap();
    assert!(own.is_own && !other.is_own);
    assert_eq!((own.id, own.account), (own_line.id, own_line.account));
    assert_eq!(own.amount, own_line.amount);
    assert_eq!(own.description.as_deref(), Some("Opening balance"));
    // It posts by itself, and not once more as a line.
    assert!(own.posted().is_none() && other.posted().is_some());
    let sides = journal.lines.iter().filter(|line| line.is_own).count();
    assert_eq!(sides, 1);
    // No other form shows its own line.
    let invoice = forms::transaction(&company, 400);
    assert!(invoice.lines.iter().all(|line| !line.is_own));
}

#[test]
fn a_kind_that_never_posts_does_not_whatever_its_lines_say() {
    let lines = "abmc_estimate_lineitem";
    let books = (2001..=2007).fold(Changed::books(), |books, id| {
        books.set((lines, "target_id", id), "is_no_post_bool", Value::Null)
    });
    let header = ("abmc_estimate_header", "transaction_id", 2000);
    let books = books.set(header, "is_no_post_bool", Value::Null).company();
    let estimate = TransactionId::new(2000);
    let posted = books.ledger().unwrap().of_transaction(estimate).count();
    assert_eq!(posted, 0);
    let read = books.transaction(estimate).unwrap().unwrap();
    assert_eq!(read.posting, PostingState::NonPosting);
    let whole = support::books().profit_and_loss(Period::all()).unwrap();
    let changed = books.profit_and_loss(Period::all()).unwrap();
    assert_eq!(changed.income.total, whole.income.total);
}

#[test]
fn a_date_is_the_one_that_is_posted_on() {
    // The header says another day than the lines, which are in the books.
    let header = ("abmc_invoice_header", "transaction_id", 400);
    let books = Changed::books();
    let books = books.set(header, "transaction_date", date("2030-01-01"));
    let books = books.company();
    let today = day("2024-06-30");
    let ledger = books.ledger().unwrap();
    let mut posted = ledger.of_transaction(TransactionId::new(400));
    assert!(posted.all(|posting| posting.date == day("2023-11-15")));
    let open = books.open_items(Side::Receivable, today).unwrap();
    let whole = support::books()
        .open_items(Side::Receivable, today)
        .unwrap();
    assert_eq!(open.home_total().to_string(), "45.00");
    let dates =
        |open: &OpenItems| -> Vec<NaiveDate> { open.iter().map(|item| item.date).collect() };
    assert_eq!(dates(&open), dates(&whole));
}

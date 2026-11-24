//! Transactions that the books do not have.

use qbw::sa17::Decimal;
use qbw::transactions::{Details, LineRole, OpenState, PostingState, TransactionKind};
use qbw::{Company, CurrencyId, NameId, OtherSide, PaymentMethodId, Price, SiteId};

use crate::odd_forms::{Form, Row};
use crate::{forms, odd_forms};

/// The header of transaction 100, whose own line is 1.
fn header(own: &[(&'static str, &'static str)]) -> Row {
    let shared = [
        ("transaction_id", "100"),
        ("target_id", "1"),
        ("transaction_date", "2024-05-01"),
    ];
    Row::new(&[&shared, own].concat())
}

/// A line of transaction 100: its id, the next one and its account, where 0 is
/// none, and its amount.
fn line(
    (id, next, account): (&'static str, &'static str, &'static str),
    amount: &'static str,
    own: &[(&'static str, &'static str)],
) -> Row {
    let shared = [
        ("transaction_id", "100"),
        ("target_id", id),
        ("next_target_id", next),
        ("account_id", account),
        ("amount_amt", amount),
        ("foreign_amount_amt", amount),
        ("is_source_bool", if id == "1" { "yes" } else { "no" }),
    ];
    Row::new(&[&shared, own].concat())
}

/// A line without an amount.
fn blank(
    (id, next, account): (&'static str, &'static str, &'static str),
    own: &[(&'static str, &'static str)],
) -> Row {
    let shared = [
        ("transaction_id", "100"),
        ("target_id", id),
        ("next_target_id", next),
        ("account_id", account),
        ("is_source_bool", if id == "1" { "yes" } else { "no" }),
    ];
    Row::new(&[&shared, own].concat())
}

/// A company with one transaction of two lines: 50 on the account `own`, where
/// a debit is positive, and as much on account 12.
fn two_lines(
    tables: &'static str,
    code: &'static str,
    own: &'static str,
    amount: &'static str,
) -> Company {
    let other = match amount.strip_prefix('-') {
        Some(_) => "50.00",
        None => "-50.00",
    };
    odd_forms::with_one(
        tables,
        header(&[("transaction_view_type", code)]),
        vec![
            line(("1", "2", own), amount, &[]),
            line(("2", "0", "12"), other, &[]),
        ],
    )
}

#[test]
fn kinds_that_no_book_had_read_as_the_others() {
    let expected = |company: &Company, kind, total: &str| {
        let read = forms::transaction(company, 100);
        assert_eq!(read.kind, kind);
        assert_eq!(forms::money(read.total), total, "{kind:?}");
        assert_eq!(forms::lines(&read), ["Account 12 - - - 50.00"], "{kind:?}");
        assert_eq!(read.posting, PostingState::Posted);
        assert_eq!(read.open(), None, "{kind:?}");
        read.details
    };
    let receipt = two_lines("abmc_sales_receipt", "0", "1", "50.00");
    let details = expected(&receipt, TransactionKind::SalesReceipt, "50.00 (0)");
    assert!(matches!(details, Details::Sale(_)));
    let refund = two_lines("abmc_customer_refund", "0", "1", "-50.00");
    let details = expected(&refund, TransactionKind::CustomerRefund, "50.00 (0)");
    assert!(matches!(details, Details::Refund(refund) if refund.payment_method.is_none()));
    let paycheck = two_lines("es_paycheck", "20", "1", "-50.00");
    let details = expected(&paycheck, TransactionKind::Paycheck, "50.00 (0)");
    assert!(matches!(details, Details::Check(_)));
    let liabilities = two_lines("es_liability_payment", "21", "1", "-50.00");
    let kind = TransactionKind::PayrollLiabilityCheck;
    assert!(matches!(
        expected(&liabilities, kind, "50.00 (0)"),
        Details::Check(_)
    ));
    // A kind without a name shows its lines as they are posted.
    let unknown = two_lines("abmc_check", "40", "1", "-50.00");
    let details = expected(&unknown, TransactionKind::Other(40), "-");
    assert_eq!(details, Details::Other);
    let without_code = two_lines("abmc_check", "0", "1", "-50.00");
    expected(&without_code, TransactionKind::Other(0), "-");
}

#[test]
fn the_table_decides_the_kind_where_it_holds_one() {
    // The code on a payment of bills by card is not to be read.
    let payment = odd_forms::with_one(
        "abmc_bill_ccard",
        header(&[("transaction_view_type", "3")]),
        vec![
            line(("1", "2", "1"), "-50.00", &[]),
            line(("2", "0", "7"), "50.00", &[("open_balance_amt", "20.00")]),
        ],
    );
    let read = forms::transaction(&payment, 100);
    assert_eq!(read.kind, TransactionKind::BillPaymentCreditCard);
    assert!(matches!(read.details, Details::BillPayment(_)));
    assert_eq!(forms::money(read.total), "50.00 (0)");
    assert_eq!(forms::lines(&read), ["Account 7 - - - 50.00"]);
    // What is left of a payment is on the line of what was owed.
    assert_eq!(forms::money(read.open()), "20.00 (0)");
}

#[test]
fn a_company_of_one_currency_has_no_other() {
    let company = two_lines("abmc_statement_charge", "25", "3", "50.00");
    let charge = forms::transaction(&company, 100);
    assert_eq!(charge.currency, CurrencyId::COMPANY);
    assert_eq!(charge.total, charge.home_total);
    assert_eq!(forms::money(charge.total), "50.00 (0)");
    assert_eq!(forms::lines(&charge), ["Account 12 - - - 50.00"]);
    assert_eq!(forms::money(charge.open()), "50.00 (0)");
    assert!(charge.sale().is_some());
    assert_eq!(forms::posted(&charge), ["12 -50.00 -50.00"]);
}

#[test]
fn a_group_of_taxes_repeats_their_sum() {
    let tax = |id, next, item, amount, percent| {
        let own = [
            ("item_id", item),
            ("is_trans_tax_bool", "yes"),
            ("unit_price_pct", percent),
        ];
        line((id, next, "9"), amount, &own)
    };
    let company = odd_forms::with_one(
        "abmc_invoice",
        header(&[("customer_id", "101")]),
        vec![
            line(("1", "2", "3"), "108.00", &[]),
            line(("2", "3", "12"), "-100.00", &[("item_id", "501")]),
            tax("3", "4", "505", "-8.00", "8"),
            tax("4", "5", "503", "-6.00", "6"),
            tax("5", "0", "504", "-2.00", "2"),
        ],
    );
    let invoice = forms::transaction(&company, 100);
    assert_eq!(
        forms::lines(&invoice),
        [
            "Item 12 501 - - 100.00",
            "SalesTaxGroup 9 505 - 8% 8.00",
            "SalesTax 9 503 - 6% 6.00",
            "SalesTax 9 504 - 2% 2.00",
        ]
    );
    assert_eq!(forms::money(invoice.tax_total()), "8.00 (0)");
    assert_eq!(forms::money(invoice.subtotal()), "100.00 (0)");
    assert_eq!(invoice.tax_item().unwrap().number(), 505);
    let percent = invoice.tax_percent().unwrap();
    assert_eq!(percent.to_string(), "8");
}

#[test]
fn cash_back_is_a_line_of_its_own() {
    let cash_back = ("is_less_cashback_bool", "yes");
    let company = odd_forms::with_one(
        "abmc_deposit",
        header(&[("transaction_name_type", "105")]),
        vec![
            line(("1", "2", "1"), "80.00", &[]),
            line(("2", "3", "3"), "-100.00", &[]),
            line(("3", "4", "9"), "20.00", &[cash_back]),
            // Set on a line that only says something, too.
            blank(("4", "0", "0"), &[cash_back, ("memo", "Till")]),
        ],
    );
    let deposit = forms::transaction(&company, 100);
    assert_eq!(
        forms::lines(&deposit),
        [
            "Account 3 - - - 100.00",
            "CashBack 9 - - - -20.00",
            "Text - - - - -",
        ]
    );
    assert_eq!(forms::money(deposit.total), "80.00 (0)");
    assert_eq!(deposit.details, Details::Deposit);
}

#[test]
fn stock_moves_from_a_site_to_another() {
    let moved = |id, next, site, quantity| {
        let own = [
            ("item_id", "501"),
            ("site_id", site),
            ("quantity_qnty", quantity),
        ];
        blank((id, next, "0"), &own)
    };
    let company = odd_forms::with_one(
        "abmc_inventory_transfer",
        header(&[]),
        vec![
            blank(("1", "2", "0"), &[]),
            moved("2", "3", "5", "-4.00"),
            moved("3", "0", "6", "4.00"),
        ],
    );
    let transfer = forms::transaction(&company, 100);
    assert_eq!(transfer.details, Details::InventoryTransfer);
    assert_eq!((transfer.total, transfer.account()), (None, None));
    let own = transfer.own_line.as_ref().unwrap();
    assert_eq!((own.amount, own.other_side), (None, OtherSide::None));
    let sites: Vec<Option<SiteId>> = transfer.lines.iter().map(|line| line.site).collect();
    assert_eq!(sites, [Some(SiteId::new(5)), Some(SiteId::new(6))]);
    let quantities = transfer.lines.iter().map(|line| line.quantity.unwrap());
    let quantities: Vec<String> = quantities.map(|quantity| quantity.to_string()).collect();
    assert_eq!(quantities, ["-4", "4"]);
    assert_eq!(forms::posted(&transfer), [""; 0]);
}

/// A purchase order of one line, with what the line and the own line have.
fn order(
    own: &[(&'static str, &'static str)],
    of_line: &[(&'static str, &'static str)],
) -> Company {
    let of_line = [&[("item_id", "501")], of_line].concat();
    odd_forms::with_one(
        "abmc_purchase_order",
        header(&[]),
        vec![
            line(("1", "2", "7"), "-50.00", own),
            line(("2", "0", "12"), "50.00", &of_line),
        ],
    )
}

/// How far the line of an order is filled: `3 open, 2 filled`, and whether it
/// is done and whether somebody closed it.
fn progress(company: &Company) -> String {
    let read = forms::transaction(company, 100);
    let order = read.lines[0].order.unwrap();
    let written = |quantity: Option<Decimal>| {
        let quantity = quantity.map(|quantity| quantity.to_string());
        quantity.unwrap_or_else(|| "none".to_owned())
    };
    let mut progress = format!(
        "{} open, {} filled",
        written(order.open),
        written(order.filled)
    );
    if order.is_done {
        progress.push_str(", done");
    }
    if order.is_closed {
        progress.push_str(", closed");
    }
    progress
}

#[test]
fn an_order_can_be_filled_with_more_or_be_closed() {
    let received = ("is_received_bool", "yes");
    let closed = ("is_manually_closed_bool", "yes");
    let quantity = ("quantity_qnty", "5.00");
    let all = order(&[received], &[quantity, ("open_qnty", "0.00"), received]);
    assert_eq!(progress(&all), "0 open, 5 filled, done");
    let read = forms::transaction(&all, 100);
    assert!(read.is_fully_received() && !read.is_closed());
    let more = order(&[received], &[quantity, ("open_qnty", "-2.00"), received]);
    assert_eq!(progress(&more), "-2 open, 7 filled, done");
    let given_up = order(&[received, closed], &[quantity, received, closed]);
    assert_eq!(progress(&given_up), "5 open, 0 filled, done, closed");
    assert!(forms::transaction(&given_up, 100).is_closed());
    // A line that does not say how many is of one.
    let one = order(&[], &[("open_qnty", "0.00"), received]);
    assert_eq!(progress(&one), "0 open, 1 filled, done");
    let read = forms::transaction(&one, 100);
    assert!(!read.is_fully_received());
    let whole = read.own_line.and_then(|own| own.order).unwrap();
    assert_eq!((whole.open, whole.filled), (None, None));
    assert_eq!(progress(&order(&[], &[])), "1 open, 0 filled");
}

#[test]
fn a_price_is_where_the_file_has_it() {
    let rate = |own: &[(&'static str, &'static str)]| {
        let company = order(&[], own);
        forms::transaction(&company, 100).lines[0].rate
    };
    let each = |rate| match rate {
        Some(Price::Each(price)) => price.to_string(),
        rate => panic!("it is a price of one: {rate:?}"),
    };
    assert_eq!(each(rate(&[("unit_price_amt", "10.00")])), "10.00");
    assert_eq!(each(rate(&[("unit_price_qnty", "0.12345")])), "0.12345");
    let percent = rate(&[("unit_price_pct", "-10.00"), ("gross_unit_price_amt", "9")]);
    assert!(matches!(percent, Some(Price::Percent(percent)) if percent.is_negative()));
    assert_eq!(each(rate(&[("gross_unit_price_amt", "11.00")])), "11.00");
    assert_eq!(each(rate(&[("gross_unit_price_qnty", "1.125")])), "1.125");
    assert_eq!(rate(&[]), None);
}

#[test]
fn a_line_is_marked_up_by_an_amount_or_a_percentage() {
    let markup = |own| {
        let company = order(&[], &[own]);
        forms::transaction(&company, 100).lines[0].markup
    };
    let amount = markup(("markup_amt", "2.50"));
    assert!(matches!(amount, Some(Price::Each(amount)) if amount.to_string() == "2.50"));
    assert!(matches!(
        markup(("markup_amt_pct", "15")),
        Some(Price::Percent(_))
    ));
}

#[test]
fn a_line_on_an_account_needs_an_amount_or_words() {
    let company = odd_forms::with_one(
        "abmc_bill",
        header(&[("transaction_view_type", "9")]),
        vec![
            line(("1", "2", "7"), "-50.00", &[]),
            line(("2", "3", "12"), "50.00", &[]),
            blank(("3", "4", "12"), &[("memo", "Nothing yet")]),
            blank(("4", "5", "12"), &[]),
            blank(("5", "0", "0"), &[]),
        ],
    );
    let bill = forms::transaction(&company, 100);
    let roles: Vec<LineRole> = bill.lines.iter().map(|line| line.role).collect();
    let (account, empty) = (LineRole::Account, LineRole::Empty);
    assert_eq!(roles, [account, account, empty, empty]);
    assert_eq!(bill.form_lines().count(), 2);
    assert_eq!(forms::posted(&bill), ["12 50.00 50.00"]);
}

#[test]
fn a_line_has_a_name_of_any_list() {
    let named = |column, code, id| {
        let company = odd_forms::with_one(
            "abmc_check",
            header(&[("transaction_view_type", "3"), ("employee_id", "301")]),
            vec![
                line(("1", "2", "1"), "-50.00", &[]),
                line(
                    ("2", "0", "12"),
                    "50.00",
                    &[("lineitem_name_type", code), (column, id)],
                ),
            ],
        );
        let check = forms::transaction(&company, 100);
        assert_eq!(check.name, Some(NameId::new(301)));
        assert_eq!((check.customer(), check.vendor()), (None, None));
        check.lines[0].name.map(NameId::number)
    };
    assert_eq!(named("customer_id", "100", "101"), Some(101));
    assert_eq!(named("vendor_id", "101", "201"), Some(201));
    assert_eq!(named("employee_id", "102", "301"), Some(301));
    assert_eq!(named("other_id", "103", "401"), Some(401));
    // The columns repeat the transaction's name.
    assert_eq!(named("employee_id", "105", "301"), None);
    assert_eq!(named("employee_id", "104", "301"), None);
}

#[test]
fn an_own_line_without_an_amount_is_a_total_of_zero() {
    let company = odd_forms::with_one(
        "abmc_check",
        header(&[("transaction_view_type", "3")]),
        vec![
            blank(("1", "2", "1"), &[("due_date", "2024-01-01")]),
            blank(("2", "0", "12"), &[]),
        ],
    );
    let check = forms::transaction(&company, 100);
    assert_eq!(forms::money(check.total), "0.00 (0)");
    assert_eq!(check.own_line.as_ref().unwrap().amount, None);
    // What the day on a check means is not known.
    assert_eq!(check.due_date, None);
    assert!(check.left_out().is_nothing());
}

#[test]
fn an_estimate_can_be_given_up() {
    let estimate = |flag| {
        let company = odd_forms::with_one(
            "abmc_estimate",
            header(&[("customer_id", "101")]),
            vec![line(("1", "0", "3"), "0.00", &[("is_inactive_bool", flag)])],
        );
        forms::transaction(&company, 100).sale().unwrap().is_active
    };
    assert!(estimate("no") && !estimate("yes"));
}

#[test]
fn a_journal_entry_on_the_receivable_can_be_open() {
    let journal = |account, own: &[(&'static str, &'static str)]| {
        let company = odd_forms::with_one(
            "abmc_general_journal",
            header(&[]),
            vec![
                line(("1", "2", account), "-50.00", own),
                line(("2", "0", "12"), "50.00", &[]),
            ],
        );
        forms::transaction(&company, 100).open_state
    };
    let open = |state| match state {
        OpenState::Open(open) => open.to_string(),
        state => panic!("something is left: {state:?}"),
    };
    // It is a credit, and the entry shows it as one.
    assert_eq!(open(journal("3", &[])), "-50.00");
    let part = [("open_balance_amt", "-20.00")];
    assert_eq!(open(journal("7", &part)), "-20.00");
    assert_eq!(journal("3", &[("is_paid_bool", "yes")]), OpenState::Paid);
    let of_the_bank = journal("1", &[("is_paid_bool", "yes")]);
    assert_eq!(of_the_bank, OpenState::NotTracked);
}

#[test]
fn a_journal_entry_is_open_when_one_of_its_lines_is() {
    let journal = |own: &[(&'static str, &'static str)], other| {
        let company = odd_forms::with_one(
            "abmc_general_journal",
            header(&[]),
            vec![
                line(("1", "2", "3"), "-50.00", own),
                line(("2", "0", "3"), "50.00", other),
            ],
        );
        let read = forms::transaction(&company, 100);
        let summary = company.transaction_summary(read.id).unwrap().unwrap();
        assert_eq!(read.open_state, summary.open);
        read
    };
    let paid: &[(&'static str, &'static str)] = &[("is_paid_bool", "yes")];
    assert_eq!(forms::money(journal(paid, &[]).open()), "50.00 (0)");
    assert_eq!(forms::money(journal(&[], paid).open()), "-50.00 (0)");
    // What is left of the two comes to nothing, and neither is paid.
    assert_eq!(forms::money(journal(&[], &[]).open()), "0.00 (0)");
    assert_eq!(journal(paid, paid).open_state, OpenState::Paid);
}

#[test]
fn a_receipt_keeps_how_it_was_paid() {
    let company = odd_forms::with_one(
        "abmc_sales_receipt",
        header(&[("payment_method_id", "2")]),
        vec![line(("1", "0", "1"), "0.00", &[])],
    );
    let receipt = forms::transaction(&company, 100);
    let method = receipt.sale().unwrap().payment_method;
    assert_eq!(method, Some(PaymentMethodId::new(2)));
}

#[test]
fn the_latest_of_several_with_one_number_is_found() {
    let invoice = |id, day, number, memorized| {
        Row::new(&[
            ("transaction_id", id),
            ("target_id", id),
            ("transaction_date", day),
            ("doc_num", number),
            ("is_memorized_transaction_bool", memorized),
        ])
    };
    let company = odd_forms::company(&[Form {
        tables: "abmc_invoice",
        headers: vec![
            invoice("11", "2024-05-01", "A-77", "no"),
            invoice("12", "2024-03-01", "a-77", "no"),
            invoice("13", "2024-05-01", "A-77", "no"),
            invoice("14", "2024-06-01", "A-77", "yes"),
            invoice("15", "2024-06-01", "A-78", "no"),
        ],
        lines: Vec::new(),
    }]);
    let found = company.find(TransactionKind::Invoice, "A-77").unwrap();
    assert_eq!(found.unwrap().number(), 13);
    let all = company.find_all(TransactionKind::Invoice, "A-77").unwrap();
    let all: Vec<i32> = all.iter().map(|id| id.number()).collect();
    assert_eq!(all, [13, 11, 12]);
}

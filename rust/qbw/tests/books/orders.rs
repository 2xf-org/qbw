//! Orders that are not filled yet.

use qbw::orders::OpenOrderLine;
use qbw::sa17::Value;
use qbw::transactions::TransactionKind;
use qbw::{Company, ItemId, LineId, NameId, Side, TransactionId};

use crate::changed::{Changed, whole};
use crate::odd_forms::Row;
use crate::{counted, odd_forms, support};

/// What is ordered, filled and left of a line, and the part of its amount.
fn figures(line: &OpenOrderLine) -> [String; 4] {
    let typed = [line.ordered, line.filled, line.open];
    let [ordered, filled, open] = typed.map(|quantity| quantity.to_string());
    let amount = line.open_amount.map(|amount| amount.to_string());
    [ordered, filled, open, amount.unwrap_or_default()]
}

fn lines(company: &Company, side: Side) -> Vec<OpenOrderLine> {
    let orders = company.open_orders(side).unwrap();
    assert_eq!(orders.side, side);
    assert!(orders.left_out.is_nothing());
    orders.lines
}

#[test]
fn a_purchase_order_is_open_by_what_did_not_come() {
    let company = support::books();
    let lines = lines(&company, Side::Payable);
    let [line] = lines.as_slice() else {
        panic!("one line is open: {lines:?}");
    };
    assert_eq!(line.transaction, TransactionId::new(1400));
    assert_eq!(line.number.as_deref(), Some("P-12"));
    assert_eq!(line.date, "2024-04-05".parse().unwrap());
    assert_eq!(line.name, Some(NameId::new(201)));
    assert!(!line.is_order_done);
    assert_eq!(line.line.id, LineId::new(1402));
    assert_eq!(line.line.item, Some(ItemId::new(502)));
    // Three of five at 20.00.
    assert_eq!(figures(line), ["5", "2", "3", "60.00"]);
}

#[test]
fn a_sales_order_that_nothing_went_out_of_is_open_as_a_whole() {
    let company = support::books();
    let lines = lines(&company, Side::Receivable);
    let [line] = lines.as_slice() else {
        panic!("one line is open: {lines:?}");
    };
    assert_eq!(line.transaction, TransactionId::new(1600));
    assert_eq!(line.name, Some(NameId::new(103)));
    assert_eq!(figures(line), ["4", "0", "4", "200.00"]);
    let stock = company.stock().unwrap();
    let valve = stock.get(ItemId::new(502)).unwrap();
    assert_eq!(valve.on_sales_orders, line.open);
}

/// The header of a sales order.
fn header() -> Row {
    Row::new(&[
        ("transaction_id", "100"),
        ("target_id", "101"),
        ("transaction_date", "2024-03-01"),
        ("is_no_post_bool", "yes"),
    ])
}

/// A line of it, with `values` besides what every line has.
fn line(values: &[(&'static str, &'static str)]) -> Row {
    let shared = [
        ("transaction_id", "100"),
        ("transaction_view_type", "27"),
        ("is_no_post_bool", "yes"),
    ];
    Row::new(&[&shared, values].concat())
}

/// The own line of the order, and `lines` after it.
fn order(own: &[(&'static str, &'static str)], lines: Vec<Row>) -> Company {
    let of_own = [
        ("target_id", "101"),
        ("is_source_bool", "yes"),
        ("next_target_id", "102"),
        ("account_id", "3"),
    ];
    let mut rows = vec![line(&[&of_own, own].concat())];
    rows.extend(lines);
    odd_forms::with_one("abmc_sales_order", header(), rows)
}

#[test]
fn a_line_is_open_until_it_is_invoiced() {
    let company = order(
        &[],
        vec![
            line(&[
                ("target_id", "102"),
                ("next_target_id", "103"),
                ("item_id", "501"),
                ("quantity_qnty", "-4"),
                ("open_qnty", "-1.5"),
                ("unit_price_amt", "3.33"),
                ("amount_amt", "-13.32"),
            ]),
            line(&[
                ("target_id", "103"),
                ("next_target_id", "104"),
                ("item_id", "501"),
                ("quantity_qnty", "-2"),
                ("is_invoiced_bool", "yes"),
            ]),
            // No quantity, which is one.
            line(&[
                ("target_id", "104"),
                ("next_target_id", "105"),
                ("item_id", "501"),
                ("amount_amt", "-25"),
            ]),
            // A tax, and words.
            line(&[
                ("target_id", "105"),
                ("next_target_id", "106"),
                ("item_id", "503"),
                ("is_trans_tax_bool", "yes"),
                ("amount_amt", "-2"),
            ]),
            line(&[("target_id", "106"), ("memo", "Call before noon")]),
        ],
    );
    let lines = lines(&company, Side::Receivable);
    let open: Vec<(i32, [String; 4])> = lines
        .iter()
        .map(|line| (line.line.id.number(), figures(line)))
        .collect();
    let open_line = |id, figures: [&str; 4]| (id, figures.map(str::to_owned));
    let expected = [
        // 1.5 at 3.33 is 4.995.
        open_line(102, ["4", "2.5", "1.5", "5.00"]),
        open_line(104, ["1", "0", "1", "25.00"]),
    ];
    assert_eq!(open, expected);
}

#[test]
fn a_discount_that_does_not_say_how_many_is_of_none() {
    let company = order(
        &[],
        vec![
            line(&[
                ("target_id", "102"),
                ("next_target_id", "103"),
                ("item_id", "506"),
                ("amount_amt", "5"),
            ]),
            line(&[
                ("target_id", "103"),
                ("next_target_id", "104"),
                ("item_id", "501"),
                ("amount_amt", "-25"),
            ]),
            line(&[
                ("target_id", "104"),
                ("item_id", "506"),
                ("quantity_qnty", "-2"),
                ("amount_amt", "5"),
            ]),
        ],
    );
    let read = company.transaction(TransactionId::new(100));
    let read = read.unwrap().unwrap();
    let progress = read.lines.iter().map(|line| {
        let order = line.order.unwrap();
        let [open, filled] = [order.open, order.filled];
        (
            open.map(|open| open.to_string()),
            filled.map(|filled| filled.to_string()),
        )
    });
    let one = |open: &str| (Some(open.to_owned()), Some("0".to_owned()));
    let progress: Vec<_> = progress.collect();
    assert_eq!(progress, [(None, None), one("1"), one("2")]);
}

#[test]
fn a_line_can_be_open_in_an_order_that_is_done() {
    let open = || {
        line(&[
            ("target_id", "102"),
            ("item_id", "501"),
            ("quantity_qnty", "-4"),
        ])
    };
    for own in [
        ("is_invoiced_bool", "yes"),
        ("is_manually_closed_bool", "yes"),
    ] {
        let company = order(&[own], vec![open()]);
        let lines = lines(&company, Side::Receivable);
        assert!(
            matches!(lines.as_slice(), [line] if line.is_order_done),
            "{own:?}"
        );
    }
    let company = order(&[], vec![open()]);
    assert!(!lines(&company, Side::Receivable)[0].is_order_done);
}

#[test]
fn an_order_that_cannot_be_read_is_counted_once() {
    let open = |id| {
        line(&[
            ("target_id", id),
            ("item_id", "501"),
            ("quantity_qnty", "-4"),
        ])
    };
    let rows = vec![open("102"), open("103")];
    let company = odd_forms::with_one("abmc_sales_order", header().unreadable(), rows);
    let orders = company.open_orders(Side::Receivable).unwrap();
    assert_eq!((orders.lines.len(), orders.left_out.rows), (0, 1));
}

#[test]
fn an_order_whose_form_cannot_be_read_is_counted() {
    // A number is no memo.
    let order = ("abmc_purchase_order_header", "transaction_id", 1400);
    let company = Changed::books().set(order, "memo", whole(5)).company();
    let orders = company.open_orders(Side::Payable).unwrap();
    assert_eq!((orders.lines.len(), orders.left_out.rows), (0, 1));
    assert!(!company.has_open_orders(Side::Payable).unwrap());
}

#[test]
fn a_line_that_is_damaged_past_its_quantity_is_counted() {
    let values = [
        ("target_id", "102"),
        ("item_id", "501"),
        ("quantity_qnty", "-4"),
        // More digits than an amount has.
        ("amount_amt", "0.000001"),
    ];
    let company = order(&[], vec![line(&values)]);
    let orders = company.open_orders(Side::Receivable).unwrap();
    assert_eq!((orders.lines.len(), orders.left_out.rows), (0, 1));
    // The order itself can be read.
    assert!(
        company
            .transaction(TransactionId::new(100))
            .unwrap()
            .is_some()
    );
}

#[test]
fn a_kind_of_order_says_whose_it_is() {
    let sides = [TransactionKind::SalesOrder, TransactionKind::PurchaseOrder]
        .map(TransactionKind::order_side);
    assert_eq!(sides, [Some(Side::Receivable), Some(Side::Payable)]);
    assert_eq!(TransactionKind::Estimate.order_side(), None);
}

#[test]
fn whether_any_line_is_open_is_known_from_the_orders_alone() {
    let company = counted::books();
    // What it takes to read an order.
    for order in [1400, 1600] {
        company.transaction(TransactionId::new(order)).unwrap();
    }
    let scanned = company.source().scanned();
    for side in [Side::Receivable, Side::Payable] {
        assert!(company.has_open_orders(side).unwrap());
    }
    // The lines of the two kinds of orders.
    assert_eq!(company.source().scanned() - scanned, 2);
    let scanned = company.source().scanned();
    for side in [Side::Receivable, Side::Payable] {
        assert!(company.has_open_orders(side).unwrap());
    }
    assert_eq!(company.source().scanned(), scanned);
    // It is the same once everything was read.
    let stock = support::books();
    stock.stock().unwrap();
    assert!(stock.has_open_orders(Side::Payable).unwrap());
}

#[test]
fn no_line_is_open_of_orders_that_are_filled() {
    let filled = |line, flag| {
        let books = Changed::books().set(line, flag, Value::Boolean(true));
        let company = books.company();
        [Side::Receivable, Side::Payable].map(|side| {
            let open = company.open_orders(side).unwrap();
            let has_open = company.has_open_orders(side).unwrap();
            assert_eq!(has_open, !open.lines.is_empty());
            has_open
        })
    };
    let bought = ("abmc_purchase_order_lineitem", "target_id", 1402);
    let sold = ("abmc_sales_order_lineitem", "target_id", 1602);
    assert_eq!(filled(bought, "is_received_bool"), [true, false]);
    assert_eq!(filled(sold, "is_invoiced_bool"), [false, true]);
    // A tax is no line of an order, though nothing of it was invoiced.
    let tax = [
        ("target_id", "102"),
        ("item_id", "503"),
        ("is_trans_tax_bool", "yes"),
        ("amount_amt", "-2"),
    ];
    let tax = order(&[], vec![line(&tax)]);
    assert!(tax.open_orders(Side::Receivable).unwrap().lines.is_empty());
    assert!(!tax.has_open_orders(Side::Receivable).unwrap());
    // A company that has only words on its orders.
    let lost = order(
        &[],
        vec![line(&[("target_id", "102"), ("memo", "Call before noon")])],
    );
    assert!(!lost.has_open_orders(Side::Receivable).unwrap());
    // An order that cannot be read counts for nothing.
    let open = line(&[
        ("target_id", "102"),
        ("item_id", "501"),
        ("quantity_qnty", "-4"),
    ]);
    let damaged = odd_forms::with_one("abmc_sales_order", header().unreadable(), vec![open]);
    assert!(!damaged.has_open_orders(Side::Receivable).unwrap());
}

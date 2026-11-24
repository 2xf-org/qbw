//! What there is in stock, and what happened to it.

use qbw::chrono::NaiveDate;
use qbw::inventory::{StockMovement, StockRow, Valuation};
use qbw::sa17::Decimal;
use qbw::transactions::TransactionKind;
use qbw::{AsOf, Company, CurrencyId, Error, ItemId, LineId, NameId, TransactionId};

use crate::small_stock::{
    ASSEMBLY, BILL, BUILD, Counts, INVOICE, Line, Own, PART, PURCHASE_ORDER, RACK, SALES_ORDER,
    SERVICE, SHELF, SHOP, SITE_TRANSFER, VENDOR_CREDIT, YARD,
};
use crate::{small_stock, support};

const VALVE: ItemId = ItemId::new(502);
const TWIN_VALVE: ItemId = ItemId::new(507);

fn day(text: &str) -> NaiveDate {
    text.parse().unwrap()
}

/// A quantity as it is typed into the figures below.
fn typed(quantity: Decimal) -> String {
    quantity.to_string()
}

fn row(company: &Company, item: ItemId) -> StockRow {
    company.stock().unwrap().get(item).unwrap().clone()
}

#[test]
fn the_stock_has_every_item_that_is_kept_in_it() {
    let company = support::books();
    let stock = company.stock().unwrap();
    let items: Vec<ItemId> = stock.rows.iter().map(|row| row.item).collect();
    assert_eq!(items, [VALVE, TWIN_VALVE]);
    assert_eq!(stock.value.to_string(), "200.00");
    let home = company.preferences().unwrap().home_currency;
    assert_eq!(stock.value.currency(), home);
    assert!(stock.left_out.is_nothing());
    assert!(stock.get(ItemId::new(501)).is_none());
}

#[test]
fn an_item_has_what_there_is_and_what_is_ordered() {
    let valve = row(&support::books(), VALVE);
    assert_eq!(typed(valve.on_hand), "8");
    assert_eq!(valve.value.to_string(), "160.00");
    assert_eq!(valve.average_cost.unwrap().to_string(), "20.00");
    assert_eq!(typed(valve.counted), "8");
    assert_eq!(typed(valve.on_order), "3");
    assert_eq!(typed(valve.on_sales_orders), "4");
    assert_eq!(typed(valve.to_be_built), "0");
    assert_eq!(typed(valve.required_for_builds), "0");
    // Four of eight are promised.
    assert_eq!(typed(valve.available), "4");
    assert_eq!(valve.reorder_point.map(typed).as_deref(), Some("12"));
    assert!(valve.needs_reorder() && valve.sites.is_empty());
    let stored = valve.stored.unwrap();
    assert_eq!(stored.on_hand, valve.on_hand);
    assert_eq!(stored.value, Some(valve.value));

    let twin = row(&support::books(), TWIN_VALVE);
    assert_eq!(
        (typed(twin.on_hand), typed(twin.counted)),
        ("1".into(), "1".into())
    );
    assert_eq!(twin.value.to_string(), "40.00");
    assert!(!twin.needs_reorder() && twin.reorder_point.is_none());
}

/// What a valuation has of an item: the quantity, the value and the average.
fn valued(valuation: &Valuation, item: ItemId) -> (String, String, Option<String>) {
    let row = valuation.get(item).unwrap();
    let average = row.average_cost.map(|cost| cost.to_string());
    (typed(row.on_hand), row.value.to_string(), average)
}

#[test]
fn a_day_has_what_there_was_at_its_end() {
    let company = support::books();
    let on = |text: &str| company.valuation(day(text)).unwrap();
    let figures = |quantity: &str, value: &str, average: &str| {
        (
            quantity.to_owned(),
            value.to_owned(),
            Some(average.to_owned()),
        )
    };
    let year_end = on("2023-12-31");
    assert_eq!(valued(&year_end, VALVE), figures("7", "140.00", "20.00"));
    assert_eq!(valued(&year_end, TWIN_VALVE), figures("0", "0.00", "0.00"));
    assert_eq!(year_end.value.to_string(), "140.00");
    assert_eq!(year_end.as_of, AsOf::Date("2023-12-31".parse().unwrap()));
    assert!(year_end.left_out.is_nothing());

    // What happened on the day counts.
    assert_eq!(
        valued(&on("2023-05-09"), VALVE),
        figures("0", "0.00", "0.00")
    );
    assert_eq!(
        valued(&on("2023-05-10"), VALVE),
        figures("10", "200.00", "20.00")
    );
    let last = on("2024-05-10");
    assert_eq!(valued(&last, VALVE), figures("8", "160.00", "20.00"));
    assert_eq!(valued(&last, TWIN_VALVE), figures("1", "40.00", "40.00"));
    assert_eq!(last.value, company.stock().unwrap().value);
    // Before the history starts, nothing is known of a cost.
    let nothing = ("0".to_owned(), "0.00".to_owned(), None);
    assert_eq!(valued(&on("2022-12-31"), VALVE), nothing);
}

/// A movement as it is typed below: the date, the transaction and the line,
/// what changed and what there was after it.
fn typed_movement(movement: &StockMovement) -> (String, i32, i32, [String; 4]) {
    let figures = [
        typed(movement.quantity),
        movement.value.to_string(),
        typed(movement.on_hand),
        movement.value_on_hand.to_string(),
    ];
    (
        movement.date.to_string(),
        movement.transaction.number(),
        movement.line.number(),
        figures,
    )
}

#[test]
fn movements_are_the_lines_that_changed_something() {
    let company = support::books();
    let movements = company.movements(VALVE).unwrap().unwrap();
    assert_eq!(
        (movements.item, movements.left_out.is_nothing()),
        (VALVE, true)
    );
    let rows: Vec<_> = movements.rows.iter().map(typed_movement).collect();
    let movement = |date: &str, transaction, line, figures: [&str; 4]| {
        (
            date.to_owned(),
            transaction,
            line,
            figures.map(str::to_owned),
        )
    };
    // The purchase order and the sales order changed nothing.
    let expected = [
        movement("2023-05-10", 200, 202, ["10", "200.00", "10", "200.00"]),
        movement("2023-11-15", 400, 402, ["-3", "-60.00", "7", "140.00"]),
        movement("2024-02-01", 700, 702, ["1", "20.00", "8", "160.00"]),
        movement("2024-03-25", 1200, 1202, ["-1", "-20.00", "7", "140.00"]),
        movement("2024-04-10", 1500, 1502, ["2", "40.00", "9", "180.00"]),
        movement("2024-05-05", 2100, 2102, ["1", "20.00", "10", "200.00"]),
        movement("2024-05-10", 2200, 2202, ["-2", "-40.00", "8", "160.00"]),
    ];
    assert_eq!(rows, expected);
    // An item that is not kept in stock, and one that is not there.
    for item in [501, 999] {
        let movements = company.movements(ItemId::new(item)).unwrap();
        assert!(movements.is_none(), "item {item}");
    }
}

#[test]
fn a_movement_says_what_it_is_and_leads_to_its_row() {
    let company = support::books();
    let movements = company.movements(VALVE).unwrap().unwrap();
    let invoice = &movements.rows[1];
    assert_eq!(invoice.kind, Some(TransactionKind::Invoice));
    assert_eq!(invoice.number.as_deref(), Some("1041"));
    assert_eq!(invoice.name, Some(NameId::new(101)));
    assert_eq!(invoice.average_cost.to_string(), "20.00");
    let row = company.stored_row(invoice.origin).unwrap().unwrap();
    assert_eq!(row.get("target_id").unwrap().as_i64(), Some(402));
    let transaction = company.transaction(invoice.transaction).unwrap().unwrap();
    let line = transaction
        .lines
        .iter()
        .find(|line| line.id == invoice.line);
    assert_eq!(line.unwrap().item, Some(VALVE));
}

/// A part of which ten were bought, with `lines` besides, and what there is of
/// it according to them.
fn counted(lines: &[Line]) -> String {
    let history = small_stock::history(&[]);
    let mut all = vec![Line::new(BILL, PART, Some("10"))];
    all.extend_from_slice(lines);
    let company = small_stock::company(&[], None, vec![history, small_stock::lines(&all)]);
    typed(row(&company, ItemId::new(PART)).counted)
}

#[test]
fn a_line_without_a_quantity_is_of_one_in_its_direction() {
    assert_eq!(counted(&[]), "10");
    assert_eq!(counted(&[Line::new(INVOICE, PART, Some("-2.5"))]), "7.5");
    assert_eq!(counted(&[Line::new(INVOICE, PART, None)]), "9");
    assert_eq!(counted(&[Line::new(BILL, PART, None)]), "11");
    assert_eq!(counted(&[Line::new(VENDOR_CREDIT, PART, None)]), "9");
}

#[test]
fn what_does_not_post_and_other_items_do_not_count() {
    let pending = Line {
        counts: Counts::Pending,
        ..Line::new(INVOICE, PART, Some("-2"))
    };
    let memorized = Line {
        counts: Counts::Memorized,
        ..Line::new(BILL, PART, Some("4"))
    };
    let service = Line::new(INVOICE, SERVICE, Some("-3"));
    assert_eq!(counted(&[pending, memorized, service]), "10");
}

#[test]
fn a_pending_build_adds_the_assembly_and_takes_its_parts() {
    let pending = |item, quantity, own| Line {
        counts: Counts::Pending,
        own,
        ..Line::new(BUILD, item, Some(quantity))
    };
    let lines = small_stock::lines(&[
        pending(ASSEMBLY, "2", Own::Side),
        pending(PART, "-6", Own::Line),
    ]);
    let history = small_stock::history(&[
        (PART, 0, "2024-01-01", "0", "0", "0"),
        (PART, 1, "2024-01-02", "10", "50", "5"),
    ]);
    let values = [("INCLUDEBUILDSINQTYAVAIL", "1")];
    let company = small_stock::company(&values, None, vec![lines, history]);
    let gate = row(&company, ItemId::new(ASSEMBLY));
    assert_eq!(
        (typed(gate.to_be_built), typed(gate.available)),
        ("2".into(), "0".into())
    );
    let hinge = row(&company, ItemId::new(PART));
    assert_eq!(typed(hinge.required_for_builds), "6");
    assert_eq!(
        (typed(hinge.on_hand), typed(hinge.available)),
        ("10".into(), "4".into())
    );
    assert!(gate.average_cost.is_none() && gate.stored.is_none());
}

/// A part of which ten are on hand, two on order and three on sales orders.
fn ordered(values: &[(&str, &str)], reorder_point: Option<&str>) -> StockRow {
    let lines = small_stock::lines(&[
        Line {
            open: Some("2"),
            ..Line::new(PURCHASE_ORDER, PART, Some("5"))
        },
        Line::new(SALES_ORDER, PART, Some("-3")),
        Line::new(SALES_ORDER, SERVICE, None),
    ]);
    let history = small_stock::history(&[
        (PART, 0, "2024-01-01", "0", "0", "0"),
        (PART, 1, "2024-01-02", "10", "50", "5"),
    ]);
    let company = small_stock::company(values, reorder_point, vec![lines, history]);
    row(&company, ItemId::new(PART))
}

#[test]
fn the_preferences_say_what_is_available() {
    let nothing = ordered(&[], None);
    assert_eq!(typed(nothing.on_sales_orders), "3");
    assert_eq!(typed(nothing.available), "10");
    let less_orders = ordered(&[("M_AVAILISONHANDMINUSONSO", "1")], None);
    assert_eq!(typed(less_orders.available), "7");
    assert_eq!(less_orders.value.currency(), CurrencyId::COMPANY);
}

#[test]
fn an_item_at_its_reorder_point_is_to_be_ordered() {
    // Ten are there and two are coming.
    assert!(ordered(&[], Some("12")).needs_reorder());
    assert!(ordered(&[], Some("12.5")).needs_reorder());
    assert!(!ordered(&[], Some("11.99")).needs_reorder());
    assert!(!ordered(&[], None).needs_reorder());
}

#[test]
fn a_site_has_what_is_in_its_bins() {
    let into = |quantity, site, bin| Line::new(BILL, PART, Some(quantity)).at(site, bin);
    let moved = |quantity, site, bin| Line::new(SITE_TRANSFER, PART, Some(quantity)).at(site, bin);
    let lines = small_stock::lines(&[
        into("6", YARD, Some(SHELF)),
        into("4", YARD, Some(RACK)),
        moved("-3", YARD, Some(SHELF)),
        moved("3", SHOP, None),
    ]);
    let history = small_stock::history(&[]);
    let company = small_stock::company(&[], None, vec![lines, history]);
    let hinge = row(&company, ItemId::new(PART));
    let sites: Vec<(i32, String)> = hinge
        .sites
        .iter()
        .map(|at| (at.site.number(), typed(at.on_hand)))
        .collect();
    let at = |site, quantity: &str| (site, quantity.to_owned());
    let expected = [at(YARD, "7"), at(SHELF, "3"), at(RACK, "4"), at(SHOP, "3")];
    assert_eq!(sites, expected);
    assert_eq!(typed(hinge.counted), "10");
    assert!(hinge.sites.iter().all(|at| at.stored.is_none()));
    // Nothing of the assembly was anywhere.
    assert!(row(&company, ItemId::new(ASSEMBLY)).sites.is_empty());
}

#[test]
fn a_value_and_a_cost_can_change_without_the_quantity() {
    let history = small_stock::history(&[
        (PART, 0, "2024-01-01", "0", "0", "3"),
        (PART, 1, "2024-01-02", "10", "50", "5"),
        // An order, which changes nothing.
        (PART, 2, "2024-01-03", "10", "50", "5"),
        // Somebody changed the value, and a bill the cost.
        (PART, 3, "2024-01-04", "10", "45", "4.5"),
        (PART, 4, "2024-01-05", "10", "45", "4.6"),
    ]);
    let company = small_stock::company(&[], None, vec![history]);
    let movements = company.movements(ItemId::new(PART)).unwrap().unwrap();
    let rows: Vec<(i32, [String; 2], String)> = movements
        .rows
        .iter()
        .map(|movement| {
            let changes = [typed(movement.quantity), movement.value.to_string()];
            (
                movement.line.number(),
                changes,
                movement.average_cost.to_string(),
            )
        })
        .collect();
    let movement =
        |line, changes: [&str; 2], cost: &str| (line, changes.map(str::to_owned), cost.to_owned());
    let expected = [
        movement(1, ["10", "50.00"], "5.00"),
        movement(3, ["0", "-5.00"], "4.50"),
        movement(4, ["0", "0.00"], "4.60"),
    ];
    assert_eq!(rows, expected);
    // The transactions are not there.
    let first = &movements.rows[0];
    assert_eq!((first.kind, first.name, &first.number), (None, None, &None));
    assert_eq!(first.transaction, TransactionId::new(1001));
    assert_eq!(first.line, LineId::new(1));
    // The start of an item has a cost.
    let start = company.valuation(day("2024-01-01")).unwrap();
    let expected = ("0".to_owned(), "0.00".to_owned(), Some("3.00".to_owned()));
    assert_eq!(valued(&start, ItemId::new(PART)), expected);
}

#[test]
fn stock_can_be_below_zero() {
    let history = small_stock::history(&[
        (PART, 0, "2024-01-01", "0", "0", "0"),
        (PART, 1, "2024-01-02", "-2", "-10", "5"),
    ]);
    let company = small_stock::company(&[], Some("0"), vec![history]);
    let hinge = row(&company, ItemId::new(PART));
    assert_eq!(
        (typed(hinge.on_hand), hinge.value.to_string()),
        ("-2".into(), "-10.00".into())
    );
    assert!(hinge.needs_reorder());
}

#[test]
fn a_history_that_cannot_be_read_is_counted() {
    let history = small_stock::damaged_history(&[
        (PART, 0, "2024-01-01", "0", "0", "0"),
        (PART, 1, "2024-01-02", "10", "50", "5"),
        (PART, 2, "2024-01-03", "12", "60", "5"),
    ]);
    let company = small_stock::company(&[], None, vec![history]);
    let stock = company.stock().unwrap();
    assert_eq!(stock.left_out.rows, 1);
    assert_eq!(typed(stock.rows[0].on_hand), "10");
    let valuation = company.valuation(day("2024-01-05")).unwrap();
    assert_eq!(valuation.left_out.rows, 1);
    // The company has no headers, and the movement needs one.
    let movements = company.movements(ItemId::new(PART)).unwrap().unwrap();
    assert_eq!((movements.rows.len(), movements.left_out.rows), (1, 2));
}

#[test]
fn damaged_lines_are_counted() {
    let company = support::damaged_books();
    let stock = company.stock().unwrap();
    // A line of an invoice. The history is whole, and so are its figures.
    assert_eq!(stock.left_out.rows, 1);
    assert_eq!(typed(stock.get(VALVE).unwrap().on_hand), "8");
    assert!(
        company
            .movements(VALVE)
            .unwrap()
            .unwrap()
            .left_out
            .is_nothing()
    );
}

#[test]
fn a_file_without_a_history_has_no_stock() {
    let company = small_stock::company(&[], None, Vec::new());
    let error = company.stock().unwrap_err();
    assert!(matches!(
        error,
        Error::NoTable {
            table: "abmc_item_history"
        }
    ));
}

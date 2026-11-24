//! What settles what is owed, and when.

use qbw::chrono::NaiveDate;
use qbw::open_items::NameOwed;
use qbw::{AsOf, Company, CurrencyId, NameId, Side};

use crate::small_owing;
use crate::small_owing::{
    BUILDER, EURO, JOB, Line, Link, PAYABLE, RECEIVABLE, RECEIVABLE_EUR, SUPPLIER, day, item,
    opened,
};

/// What is open of one thing of 100 that another of 100 settles by a link of
/// `code`, on the days before and after.
fn settled(side: Side, code: i16, amounts: [&'static str; 2]) -> [Vec<(i32, i32, String)>; 3] {
    let (account, name) = match side {
        Side::Receivable => (RECEIVABLE, BUILDER),
        _ => (PAYABLE, SUPPLIER),
    };
    let lines = [
        Line::new(101, "2024-01-05", account, name, amounts[0]),
        Line::new(201, "2024-01-20", account, name, amounts[1]),
    ];
    let company = small_owing::company(&lines, &[(code, 201, 101, "-100.00")]);
    ["2024-01-19", "2024-01-20", "2024-01-31"].map(|date| {
        let open = company.open_items(side, day(date)).unwrap();
        opened(&open)
    })
}

#[test]
fn customers_settle_by_six_kinds_of_links() {
    let before = [item(1, BUILDER, "100.00")];
    // A payment, its discount, a credit, a journal entry.
    for code in [4, 5, 3, 22] {
        let [earlier, on_the_day, after] = settled(Side::Receivable, code, ["100.00", "-100.00"]);
        assert_eq!(earlier, before, "code {code}");
        assert!(on_the_day.is_empty() && after.is_empty(), "code {code}");
    }
    // Refunds of a credit and of a payment settle what was owed to the customer.
    let before = [item(1, BUILDER, "-100.00")];
    for code in [2, 10] {
        let [earlier, on_the_day, after] = settled(Side::Receivable, code, ["-100.00", "100.00"]);
        assert_eq!(earlier, before, "code {code}");
        assert!(on_the_day.is_empty() && after.is_empty(), "code {code}");
    }
}

#[test]
fn the_company_settles_by_four_kinds_of_links() {
    let before = [item(1, SUPPLIER, "100.00")];
    // A check, a card or a journal entry, a vendor's credit, a discount.
    for code in [6, 7, 8, 16] {
        let [earlier, on_the_day, after] = settled(Side::Payable, code, ["-100.00", "100.00"]);
        assert_eq!(earlier, before, "code {code}");
        assert!(on_the_day.is_empty() && after.is_empty(), "code {code}");
    }
}

#[test]
fn no_other_link_settles_anything() {
    let both = [item(1, BUILDER, "100.00"), item(2, BUILDER, "-100.00")];
    // Those of the other side, a deposit, goods received, orders invoiced.
    for code in [6, 7, 8, 16, 9, 15, 20, 21] {
        let [earlier, _, after] = settled(Side::Receivable, code, ["100.00", "-100.00"]);
        assert_eq!(
            (&earlier[..], &after[..]),
            (&both[..1], &both[..]),
            "{code}"
        );
    }
    let both = [item(1, SUPPLIER, "100.00"), item(2, SUPPLIER, "-100.00")];
    for code in [2, 3, 4, 5, 10, 22, 9, 15, 20, 21] {
        let [earlier, _, after] = settled(Side::Payable, code, ["-100.00", "100.00"]);
        assert_eq!(
            (&earlier[..], &after[..]),
            (&both[..1], &both[..]),
            "{code}"
        );
    }
}

#[test]
fn a_link_counts_from_the_later_of_its_two_days() {
    // The payment is dated before what it paid.
    let lines = [
        Line::new(101, "2024-01-20", RECEIVABLE, BUILDER, "100.00"),
        Line::new(201, "2024-01-05", RECEIVABLE, BUILDER, "-100.00"),
    ];
    let company = small_owing::company(&lines, &[(4, 201, 101, "-100.00")]);
    let open = |date| opened(&company.open_items(Side::Receivable, day(date)).unwrap());
    assert_eq!(open("2024-01-04"), []);
    assert_eq!(open("2024-01-05"), [item(2, BUILDER, "-100.00")]);
    assert_eq!(open("2024-01-19"), [item(2, BUILDER, "-100.00")]);
    assert_eq!(open("2024-01-20"), []);
}

#[test]
fn what_is_on_no_day_settles_nothing() {
    // A payment whose lines and header are lost.
    let invoice = Line::new(101, "2024-01-05", RECEIVABLE, BUILDER, "100.00");
    let company = small_owing::company(&[invoice], &[(4, 9901, 101, "-100.00")]);
    let open = company.open_items(Side::Receivable, day("2024-12-31"));
    assert_eq!(opened(&open.unwrap()), [item(1, BUILDER, "100.00")]);
}

/// A journal entry that debits a customer 100 and credits it 40, of which the
/// credit paid an invoice and a payment paid the debit.
const NETTED: &[Line] = &[
    Line::new(301, "2024-05-10", RECEIVABLE, BUILDER, "100.00").paid(),
    Line::new(302, "2024-05-10", RECEIVABLE, BUILDER, "-40.00").paid(),
    Line::new(401, "2024-05-10", RECEIVABLE, BUILDER, "40.00").paid(),
    Line::new(501, "2024-05-12", RECEIVABLE, BUILDER, "-100.00").paid(),
];
const NETTED_LINKS: &[Link] = &[(22, 302, 401, "-40.00"), (4, 501, 301, "-100.00")];

#[test]
fn a_journal_entry_that_debits_and_credits_one_customer_settles_line_by_line() {
    let company = small_owing::company(NETTED, NETTED_LINKS);
    let open = |as_of: AsOf| company.open_items(Side::Receivable, as_of).unwrap();
    assert_eq!(opened(&open(AsOf::Latest)), []);
    assert_eq!(opened(&open(AsOf::Date(day("2024-12-31")))), []);
    let between = open(AsOf::Date(day("2024-05-11")));
    assert_eq!(opened(&between), [item(3, BUILDER, "100.00")]);
    assert_eq!(between.items[0].total.to_string(), "100.00");
    // Without the links, each of its sides is an item.
    let alone = small_owing::company(NETTED, &[]);
    let alone = alone.open_items(Side::Receivable, day("2024-05-10"));
    let expected = [
        item(3, BUILDER, "100.00"),
        item(3, BUILDER, "-40.00"),
        item(4, BUILDER, "40.00"),
    ];
    assert_eq!(opened(&alone.unwrap()), expected);
    let balance = company.ledger().unwrap();
    let balance = balance.customer_balance(NameId::new(BUILDER), AsOf::Latest);
    assert!(balance.unwrap().home_amount.is_zero());
}

#[test]
fn a_link_to_a_line_settles_the_part_with_that_line() {
    // Two lines of one sign are one part, whichever of them is linked to.
    let lines = [
        Line::new(101, "2024-05-10", RECEIVABLE, BUILDER, "60.00"),
        Line::new(102, "2024-05-10", RECEIVABLE, BUILDER, "40.00"),
        Line::new(103, "2024-05-10", RECEIVABLE, JOB, "20.00"),
        Line::new(201, "2024-05-12", RECEIVABLE, BUILDER, "-40.00").paid(),
        Line::new(301, "2024-05-12", RECEIVABLE, JOB, "-5.00").paid(),
    ];
    let links = [(4, 201, 102, "-40.00"), (4, 301, 103, "-5.00")];
    let company = small_owing::company(&lines, &links);
    let open = company.open_items(Side::Receivable, day("2024-05-31"));
    let expected = [item(1, BUILDER, "60.00"), item(1, JOB, "15.00")];
    assert_eq!(opened(&open.unwrap()), expected);
}

/// What is open of an invoice of so many euros, in euros and in the company's
/// currency, when so many of them were paid.
fn euros(own: &'static str, home: &'static str, paid: Option<&'static str>) -> [String; 2] {
    let mut lines = vec![Line::new(101, "2024-03-01", RECEIVABLE_EUR, EURO, own).home(home)];
    let mut links = Vec::new();
    if let Some(paid) = paid {
        lines.push(Line::new(201, "2024-03-02", RECEIVABLE_EUR, EURO, paid).paid());
        links.push((4, 201, 101, paid));
    }
    let company = small_owing::company(&lines, &links);
    let open = company.open_items(Side::Receivable, day("2024-03-31"));
    let open = open.unwrap();
    let [item] = &open.items[..] else {
        panic!("one item is open: {open:?}");
    };
    assert_eq!(item.open.currency(), CurrencyId::new(2));
    [item.open.to_string(), item.home_open.to_string()]
}

#[test]
fn a_foreign_item_is_worth_what_its_line_posted() {
    // All of it is what was posted, though the rate of 1.1 says 110.00.
    assert_eq!(euros("100.00", "110.07", None), ["100.00", "110.07"]);
    // 33.33 are 36.663 at the rate, and the 10.15 that were paid are 11.165. What
    // is left is 36.66 less 11.17, and not 23.18 at the rate, 25.50.
    let part = euros("33.33", "36.66", Some("-10.15"));
    assert_eq!(part, ["23.18", "25.49"]);
    // A line that is not at the rate has what is left of it at the rate.
    let part = euros("100.00", "110.07", Some("-40.00"));
    assert_eq!(part, ["60.00", "66.00"]);
}

#[test]
fn adjustments_make_up_the_rest_of_a_balance() {
    let lines = [
        Line::new(101, "2024-03-01", RECEIVABLE_EUR, EURO, "100.00").home("110.00"),
        Line::new(201, "2024-03-31", RECEIVABLE_EUR, EURO, "0.00").home("5.00"),
        Line::new(301, "2024-04-30", RECEIVABLE_EUR, EURO, "0.00").home("-2.00"),
        Line::new(401, "2024-03-31", RECEIVABLE, BUILDER, "0.00"),
    ];
    let company = small_owing::company(&lines, &[]);
    let owed = |date: &str| {
        let owed = company.owed_by(NameId::new(EURO), day(date)).unwrap();
        let owed = owed.unwrap();
        let home = [owed.home_total, owed.adjustments];
        let balance = owed.home_total.checked_add(owed.adjustments).unwrap();
        let ledger = company.ledger().unwrap();
        let posted = ledger.customer_balance(NameId::new(EURO), day(date));
        assert_eq!(balance, posted.unwrap().home_amount);
        home.map(|amount| amount.to_string())
    };
    assert_eq!(owed("2024-03-30"), ["110.00", "0.00"]);
    assert_eq!(owed("2024-03-31"), ["110.00", "5.00"]);
    assert_eq!(owed("2024-04-30"), ["110.00", "3.00"]);
    let builder = company.owed_by(NameId::new(BUILDER), day("2024-04-30"));
    assert!(builder.unwrap().unwrap().adjustments.is_zero());
}

fn owed(company: &Company, side: Side, date: &str) -> Vec<(i32, [String; 5])> {
    let date: NaiveDate = day(date);
    let names = company.names_owed(side, date).unwrap();
    assert_eq!((names.side, names.day), (side, date));
    assert_eq!(names.as_of, AsOf::Date(date));
    let row = |row: &NameOwed| {
        let only = |totals: &qbw::Totals| {
            let amounts: Vec<String> = totals.iter().map(|total| total.to_string()).collect();
            amounts.join(" and ")
        };
        let figures = [
            only(&row.total),
            only(&row.overdue),
            row.home_total.to_string(),
            row.home_overdue.to_string(),
            row.adjustments.to_string(),
        ];
        (row.name.number(), figures)
    };
    names.rows.iter().map(row).collect()
}

#[test]
fn every_name_has_what_it_owes_with_its_jobs_in_one_pass() {
    let lines = [
        Line::new(101, "2024-01-05", RECEIVABLE, JOB, "70.00").due("2024-01-31"),
        Line::new(201, "2024-01-06", RECEIVABLE, BUILDER, "30.00").due("2024-02-10"),
        Line::new(301, "2024-01-20", RECEIVABLE, BUILDER, "-45.00").due("2024-01-20"),
        Line::new(401, "2024-01-05", RECEIVABLE_EUR, EURO, "10.00")
            .home("11.00")
            .due("2024-01-06"),
        Line::new(501, "2024-01-31", RECEIVABLE_EUR, EURO, "0.00").home("2.50"),
        Line::new(601, "2024-01-05", PAYABLE, SUPPLIER, "-80.00").due("2024-01-15"),
        Line::new(701, "2024-01-06", PAYABLE, SUPPLIER, "20.00").due("2024-01-06"),
    ];
    let company = small_owing::company(&lines, &[]);
    let row = |name, figures: [&str; 5]| (name, figures.map(str::to_owned));
    let expected = [
        row(BUILDER, ["55.00", "70.00", "55.00", "70.00", "0.00"]),
        row(JOB, ["70.00", "70.00", "70.00", "70.00", "0.00"]),
        row(EURO, ["10.00", "10.00", "11.00", "11.00", "2.50"]),
    ];
    assert_eq!(owed(&company, Side::Receivable, "2024-02-05"), expected);
    let expected = [row(SUPPLIER, ["60.00", "80.00", "60.00", "80.00", "0.00"])];
    assert_eq!(owed(&company, Side::Payable, "2024-02-05"), expected);
    // It is what each of them is asked for.
    let names = company
        .names_owed(Side::Receivable, day("2024-02-05"))
        .unwrap();
    for row in &names.rows {
        let alone = company
            .owed_by(row.name, day("2024-02-05"))
            .unwrap()
            .unwrap();
        assert_eq!((&row.total, &row.overdue), (&alone.total, &alone.overdue));
        assert_eq!(row.home_total, alone.home_total);
        assert_eq!(row.adjustments, alone.adjustments);
        assert_eq!(names.get(row.name), Some(row));
    }
    assert_eq!(names.get(NameId::new(SUPPLIER)), None);
}

#[test]
fn a_name_without_a_row_owes_nothing_in_its_own_currency() {
    let line = Line::new(101, "2024-01-05", RECEIVABLE, JOB, "70.00");
    let company = small_owing::company(&[line], &[]);
    let names = company.names_owed(Side::Receivable, AsOf::Latest).unwrap();
    assert_eq!(names.rows.len(), 2);
    let nothing = names.owed(NameId::new(EURO)).unwrap();
    let total = nothing.total.single().unwrap();
    assert!(total.is_zero() && nothing.overdue.single().unwrap().is_zero());
    assert_eq!(total.currency(), CurrencyId::new(2));
    let home = [
        nothing.home_total,
        nothing.home_overdue,
        nothing.adjustments,
    ];
    assert!(
        home.iter()
            .all(|home| home.is_zero() && home.currency() == CurrencyId::new(1))
    );
    // One with a row has that, and one of the other list has none.
    let job = NameId::new(JOB);
    assert_eq!(names.owed(job).as_ref(), names.get(job));
    assert_eq!(names.owed(NameId::new(SUPPLIER)), None);
    let vendors = company.names_owed(Side::Payable, AsOf::Latest).unwrap();
    assert!(
        vendors
            .owed(NameId::new(SUPPLIER))
            .unwrap()
            .home_total
            .is_zero()
    );
    assert_eq!(vendors.owed(job), None);
}

#[test]
fn what_takes_off_what_is_owed_is_never_overdue() {
    let lines = [
        Line::new(101, "2024-01-05", RECEIVABLE, BUILDER, "70.00").due("2024-01-31"),
        Line::new(201, "2024-01-06", RECEIVABLE, BUILDER, "-30.00").due("2024-01-15"),
    ];
    let company = small_owing::company(&lines, &[]);
    let open = company
        .open_items(Side::Receivable, day("2024-02-29"))
        .unwrap();
    let overdue = |date| opened(open.overdue(day(date)));
    assert_eq!(overdue("2024-01-31"), []);
    assert_eq!(overdue("2024-02-01"), [item(1, BUILDER, "70.00")]);
    let owed = company.owed_by(NameId::new(BUILDER), day("2024-02-29"));
    let owed = owed.unwrap().unwrap();
    assert_eq!(owed.home_total.to_string(), "40.00");
    assert_eq!(owed.home_overdue.to_string(), "70.00");
}

#[test]
fn the_latest_day_is_the_last_that_something_is_posted_on() {
    let lines = [
        Line::new(101, "2024-01-05", RECEIVABLE, BUILDER, "70.00").due("2024-01-31"),
        Line::new(201, "2024-02-20", PAYABLE, SUPPLIER, "-30.00"),
    ];
    let company = small_owing::company(&lines, &[]);
    let owed = company.owed_by(NameId::new(BUILDER), AsOf::Latest);
    let owed = owed.unwrap().unwrap();
    assert_eq!((owed.as_of, owed.day), (AsOf::Latest, day("2024-02-20")));
    assert_eq!(owed.home_overdue.to_string(), "70.00");
}

//! How late what is owed is.

use qbw::aging::{Aged, Aging, AgingBy, AgingOptions};
use qbw::chrono::NaiveDate;
use qbw::settings::AgingBasis;
use qbw::{AsOf, CurrencyId, NameId, Side};

use crate::small_owing::{BUILDER, Line, RECEIVABLE};
use crate::{small_owing, support};

fn day(text: &str) -> NaiveDate {
    text.parse().unwrap()
}

fn aging(side: Side, options: &AgingOptions) -> Aging {
    support::books().aging(side, options).unwrap()
}

fn texts(aged: &Aged) -> Vec<String> {
    aged.amounts.iter().map(ToString::to_string).collect()
}

#[test]
fn what_is_open_is_sorted_by_how_late_it_is() {
    let options = AgingOptions::new(day("2024-02-15"));
    let aging = aging(Side::Receivable, &options);
    let [ada, bell] = aging.rows.as_slice() else {
        panic!("two customers owe: {aging:?}");
    };
    // Due on the 15th of December, 62 days before.
    assert_eq!(ada.name, Some(NameId::new(101)));
    assert_eq!(
        texts(&ada.home_aged),
        ["0.00", "0.00", "0.00", "46.00", "0.00"]
    );
    assert_eq!(ada.home_aged.total.to_string(), "46.00");
    // Due in March, in euros.
    assert_eq!(
        texts(&bell.aged),
        ["100.00", "0.00", "0.00", "0.00", "0.00"]
    );
    assert_eq!(bell.aged.total.currency(), CurrencyId::new(2));
    assert_eq!(bell.home_aged.total.to_string(), "110.00");
    assert_eq!(
        texts(&aging.totals),
        ["110.00", "0.00", "0.00", "46.00", "0.00"]
    );
    assert_eq!(aging.totals.total.to_string(), "156.00");
    assert_eq!((aging.basis, aging.buckets.len()), (AgingBasis::DueDate, 5));
    assert!(aging.adjustments.is_zero() && aging.left_out.is_nothing());
}

#[test]
fn an_aging_can_count_from_the_transaction_in_other_buckets() {
    let options = AgingOptions::new(day("2024-02-15"))
        .basis(AgingBasis::TransactionDate)
        .try_buckets(2, 60)
        .unwrap();
    let aging = aging(Side::Receivable, &options);
    let ends: Vec<_> = aging
        .buckets
        .iter()
        .map(|bucket| (bucket.first, bucket.last))
        .collect();
    let expected = [
        (None, Some(0)),
        (Some(1), Some(60)),
        (Some(61), Some(120)),
        (Some(121), None),
    ];
    assert_eq!(ends, expected);
    // Invoiced 92 and 5 days before.
    assert_eq!(texts(&aging.totals), ["0.00", "110.00", "46.00", "0.00"]);
}

#[test]
fn an_aging_can_have_a_row_for_each_item() {
    let options = AgingOptions::new(day("2024-06-30")).by(AgingBy::Items);
    let aging = aging(Side::Payable, &options);
    let rows: Vec<(i32, i64, String)> = aging
        .rows
        .iter()
        .map(|row| {
            let aged = row.item.as_ref().unwrap();
            (
                aged.item.transaction.number(),
                aged.age,
                row.home_aged.total.to_string(),
            )
        })
        .collect();
    // The receipt is 51 days late. The credit ages from its own date.
    let expected = [(1500, 51, "40.00"), (1200, 97, "-20.00")];
    assert_eq!(
        rows,
        expected.map(|(id, age, amount)| (id, age, amount.to_owned()))
    );
    assert_eq!(
        texts(&aging.totals),
        ["0.00", "0.00", "40.00", "0.00", "-20.00"]
    );
}

/// An invoice that is due late, one that says no day, and two credits, of which
/// one was due before it was given.
const LINES: &[Line] = &[
    Line::new(101, "2024-01-01", RECEIVABLE, BUILDER, "100.00").due("2024-03-31"),
    Line::new(201, "2024-01-01", RECEIVABLE, BUILDER, "10.00"),
    Line::new(301, "2024-03-20", RECEIVABLE, BUILDER, "-1.00").due("2024-01-01"),
    Line::new(401, "2024-01-01", RECEIVABLE, BUILDER, "-0.10").due("2024-03-31"),
];

#[test]
fn a_credit_ages_from_its_own_day_and_so_does_what_says_no_day() {
    let company = small_owing::company(LINES, &[]);
    let options = AgingOptions::new(day("2024-04-10"));
    let aging = company.aging(Side::Receivable, &options).unwrap();
    assert_eq!(aging.basis, AgingBasis::DueDate);
    // The invoice is ten days late. The credits are 21 and 100 days old, and so
    // is what says no day.
    assert_eq!(
        texts(&aging.totals),
        ["0.00", "99.00", "0.00", "0.00", "9.90"]
    );
    let mut items = options.by(AgingBy::Items);
    items = items.try_buckets(0, 1).unwrap();
    let aging = company.aging(Side::Receivable, &items).unwrap();
    let ages = aging.rows.iter().map(|row| {
        let item = row.item.as_ref().unwrap();
        (item.item.transaction.number(), item.age)
    });
    let mut ages: Vec<(i32, i64)> = ages.collect();
    ages.sort_unstable();
    assert_eq!(ages, [(1, 10), (2, 100), (3, 21), (4, 100)]);
}

#[test]
fn a_company_can_prefer_to_age_from_the_transaction() {
    let preferred = |value| {
        let preferences = [("AGEFROMTXDATE", value)];
        small_owing::with_preferences(&preferences, LINES, &[])
    };
    let options = AgingOptions::new(day("2024-04-10"));
    let company = preferred("1");
    let aging = company.aging(Side::Receivable, &options).unwrap();
    assert_eq!(aging.basis, AgingBasis::TransactionDate);
    assert_eq!(
        texts(&aging.totals),
        ["0.00", "-1.00", "0.00", "0.00", "109.90"]
    );
    // Whoever asks can say otherwise.
    let due = options.basis(AgingBasis::DueDate);
    let aging = company.aging(Side::Receivable, &due).unwrap();
    assert_eq!(
        texts(&aging.totals),
        ["0.00", "99.00", "0.00", "0.00", "9.90"]
    );
    let company = preferred("0");
    let aging = company.aging(Side::Receivable, &options).unwrap();
    assert_eq!(aging.basis, AgingBasis::DueDate);
}

#[test]
fn an_aging_has_so_many_buckets_of_some_days() {
    let options = AgingOptions::new(AsOf::Latest);
    let most = AgingOptions::MOST_BUCKETS;
    assert!(options.try_buckets(most, 1).is_ok());
    let error = options.try_buckets(most + 1, 30).unwrap_err();
    let says = "an aging has at most 366 buckets of at least one day, not 367 of 30";
    assert_eq!(error.to_string(), says);
    let error = options.try_buckets(3, 0).unwrap_err();
    assert_eq!((error.count, error.days), (3, 0));
}

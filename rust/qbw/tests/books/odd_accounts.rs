//! Statements of books whose accounts are not as they should be. Nothing that
//! is posted is lost.

use qbw::chrono::NaiveDate;
use qbw::reports::{EarlierProfit, SectionAccount};
use qbw::{AsOf, Money, Period};

use crate::small_books;
use crate::small_books::BANK;

fn day(text: &str) -> NaiveDate {
    text.parse().unwrap()
}

fn texts<const N: usize>(amounts: [Money; N]) -> [String; N] {
    amounts.map(|amount| amount.to_string())
}

/// The accounts of a section and the ones below each, as numbers.
fn nested(accounts: &[SectionAccount]) -> String {
    let accounts = accounts.iter().map(|account| {
        let below = nested(&account.children);
        format!("{}[{below}]", account.account)
    });
    accounts.collect()
}

#[test]
fn a_ring_of_parents_is_in_its_section_once() {
    let accounts = [
        BANK,
        (2, 10, 3, 0),
        (3, 10, 2, 0),
        (4, 10, 2, 0),
        (5, 10, 0, 0),
    ];
    let entries = [
        ("2024-01-01", 2, "1.00"),
        ("2024-01-01", 3, "10.00"),
        ("2024-01-01", 4, "100.00"),
        ("2024-01-01", 5, "1000.00"),
    ];
    let company = small_books::company(&accounts, &entries);
    let statement = company
        .profit_and_loss(day("2024-01-01")..=day("2024-12-31"))
        .unwrap();
    assert_eq!(statement.income.total.to_string(), "1111.00");
    assert_eq!(nested(&statement.income.accounts), "5[]2[3[]4[]]");
    let listing = company.account_listing(AsOf::Latest).unwrap();
    assert_eq!(
        listing.rows[0].total.map(|total| total.to_string()),
        Some("1111.00".to_owned())
    );
}

#[test]
fn an_account_nested_too_deep_stands_at_the_top() {
    let mut accounts = vec![BANK];
    accounts.extend((2..=14).map(|id| (id, 10, if id == 2 { 0 } else { id - 1 }, 0)));
    let company = small_books::company(&accounts, &[("2024-01-01", 14, "7.00")]);
    let income = company
        .profit_and_loss(day("2024-01-01")..=day("2024-12-31"))
        .unwrap()
        .income;
    assert_eq!(income.total.to_string(), "7.00");
    assert_eq!(nested(&income.accounts), "11[12[13[14[]]]]");
}

#[test]
fn a_parent_of_another_type_is_not_in_the_section() {
    let accounts = [BANK, (2, 12, 0, 0), (3, 10, 2, 0)];
    let company = small_books::company(&accounts, &[("2024-01-01", 3, "5.00")]);
    let statement = company
        .profit_and_loss(day("2024-01-01")..=day("2024-12-31"))
        .unwrap();
    assert_eq!(nested(&statement.income.accounts), "3[]");
    assert_eq!(nested(&statement.expense.accounts), "");
}

#[test]
fn the_balance_sheet_counts_balances_of_accounts_below() {
    let accounts = [BANK, (2, 9, 0, 0), (3, 9, 2, 0)];
    let company = small_books::company(
        &accounts,
        &[("2024-01-01", 2, "1.00"), ("2024-01-01", 3, "10.00")],
    );
    let listing = company.account_listing(AsOf::Latest).unwrap();
    let totals = listing
        .rows
        .iter()
        .map(|row| row.total.unwrap().to_string());
    assert_eq!(
        totals.collect::<Vec<String>>(),
        ["11.00", "-11.00", "-10.00"]
    );
    let sheet = company.balance_sheet(day("2024-12-31")).unwrap();
    assert_eq!(nested(&sheet.equity.accounts.accounts), "2[3[]]");
    assert_eq!(
        texts([sheet.equity.total, sheet.assets.total]),
        ["11.00", "11.00"]
    );
}

#[test]
fn earlier_profit_stands_alone_without_an_account_for_it() {
    let accounts = [BANK, (2, 10, 0, 0)];
    let entries = [("2023-06-01", 2, "30.00"), ("2024-06-01", 2, "4.00")];
    let company = small_books::company(&accounts, &entries);
    let balance = company.trial_balance(day("2024-12-31")).unwrap();
    let rows: Vec<_> = balance
        .rows
        .iter()
        .map(|row| {
            (
                row.account.map(|account| account.number()),
                row.balance.to_string(),
            )
        })
        .collect();
    let expected = [(Some(1), "34.00"), (Some(2), "-4.00"), (None, "-30.00")];
    assert_eq!(
        rows,
        expected.map(|(account, balance)| (account, balance.to_owned()))
    );
    let equity = company.balance_sheet(day("2024-12-31")).unwrap().equity;
    assert_eq!(nested(&equity.accounts.accounts), "");
    assert!(matches!(equity.earlier_profit, EarlierProfit::Alone(_)));
    let earlier_profit = equity.earlier_profit.amount();
    let figures = texts([earlier_profit, equity.net_income, equity.total]);
    assert_eq!(figures, ["30.00", "4.00", "34.00"]);
}

#[test]
fn an_account_that_is_not_equity_does_not_take_the_profit() {
    // The code of Retained Earnings on an expense.
    let accounts = [BANK, (2, 10, 0, 0), (3, 12, 0, 8)];
    let company = small_books::company(&accounts, &[("2023-06-01", 2, "30.00")]);
    let sheet = company.balance_sheet(day("2024-12-31")).unwrap();
    assert_eq!(
        texts([sheet.assets.total, sheet.liabilities_and_equity]),
        ["30.00", "30.00"]
    );
    let balance = company.trial_balance(day("2024-12-31")).unwrap();
    assert_eq!(texts([balance.debits, balance.credits]), ["30.00", "30.00"]);
}

#[test]
fn what_is_posted_to_no_known_account_is_unplaced() {
    // An account of a type that no release has, and one that is not listed.
    let accounts = [BANK, (2, 99, 0, 0)];
    let entries = [("2024-01-01", 2, "5.00"), ("2024-01-01", 7, "2.00")];
    let company = small_books::company(&accounts, &entries);
    let sheet = company.balance_sheet(day("2024-12-31")).unwrap();
    assert_eq!(
        texts([sheet.assets.total, sheet.liabilities_and_equity]),
        ["7.00", "0.00"]
    );
    assert_eq!(sheet.unplaced.to_string(), "-7.00");
    let statement = company.profit_and_loss(Period::all()).unwrap();
    assert_eq!(
        texts([statement.net_income, statement.unplaced]),
        ["0.00", "-7.00"]
    );
    let balance = company.trial_balance(day("2024-12-31")).unwrap();
    assert_eq!(texts([balance.debits, balance.credits]), ["7.00", "7.00"]);
    let ledger = company
        .general_ledger(day("2024-01-01")..=day("2024-12-31"))
        .unwrap();
    let last = ledger.accounts.last().unwrap();
    assert_eq!(
        (last.account.number(), last.closing.to_string()),
        (7, "-2.00".to_owned())
    );
}

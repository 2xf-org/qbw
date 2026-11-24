//! What is posted on the first and on the last day of a fiscal year.

use qbw::chrono::NaiveDate;
use qbw::{AccountId, Company, Page, Period};

use crate::small_books;
use crate::small_books::{Account, BANK};

/// Equity that QuickBooks uses for the profit of earlier years, and income.
const ACCOUNTS: &[Account] = &[BANK, (RETAINED, 9, 0, 8), (INCOME, 10, 0, 0)];
const RETAINED: i32 = 2;
const INCOME: i32 = 3;

/// Income of years that start in January, each amount of a day of its own.
const ENTRIES: &[(&str, i32, &str)] = &[
    ("2023-12-31", INCOME, "1.00"),
    ("2024-01-01", INCOME, "10.00"),
    ("2024-06-30", INCOME, "100.00"),
    ("2024-12-31", INCOME, "1000.00"),
    ("2025-01-01", INCOME, "10000.00"),
];

fn books() -> Company {
    small_books::company(ACCOUNTS, ENTRIES)
}

fn day(text: &str) -> NaiveDate {
    text.parse().unwrap()
}

/// Net income and retained earnings at the end of a day.
fn equity(company: &Company, date: &str) -> [String; 2] {
    let sheet = company.balance_sheet(day(date)).unwrap();
    assert_eq!(sheet.assets.total, sheet.liabilities_and_equity);
    let retained = sheet.equity.accounts.accounts.iter();
    let mut retained = retained.filter(|account| account.account == AccountId::new(RETAINED));
    let retained = retained.next().map(|account| account.amount.to_string());
    [
        sheet.equity.net_income.to_string(),
        retained.unwrap_or_default(),
    ]
}

#[test]
fn the_first_day_of_a_year_is_part_of_it() {
    let company = books();
    assert_eq!(equity(&company, "2023-12-31"), ["1.00", ""]);
    assert_eq!(equity(&company, "2024-01-01"), ["10.00", "1.00"]);
    assert_eq!(equity(&company, "2024-06-30"), ["110.00", "1.00"]);
    assert_eq!(equity(&company, "2024-12-31"), ["1110.00", "1.00"]);
    assert_eq!(equity(&company, "2025-01-01"), ["10000.00", "1111.00"]);
    let earned = |date| {
        let year = company.fiscal_year(day(date)).unwrap();
        company
            .profit_and_loss(year)
            .unwrap()
            .net_income
            .to_string()
    };
    assert_eq!(earned("2024-07-01"), "1110.00");
    assert_eq!(earned("2025-07-01"), "10000.00");
}

#[test]
fn a_year_can_start_with_another_month() {
    let entries = [
        ("2024-06-30", INCOME, "1.00"),
        ("2024-07-01", INCOME, "10.00"),
        ("2025-06-30", INCOME, "100.00"),
        ("2025-07-01", INCOME, "1000.00"),
    ];
    let company = small_books::with_years("7", ACCOUNTS, &entries);
    assert_eq!(equity(&company, "2024-07-01"), ["10.00", "1.00"]);
    assert_eq!(equity(&company, "2025-06-30"), ["110.00", "1.00"]);
    assert_eq!(equity(&company, "2025-07-01"), ["1000.00", "111.00"]);
}

#[test]
fn a_trial_balance_moves_the_profit_with_the_first_day() {
    let company = books();
    let rows = |date| -> Vec<String> {
        let balance = company.trial_balance(day(date)).unwrap();
        assert_eq!(balance.debits, balance.credits);
        let rows = balance.rows.iter().map(|row| row.balance.to_string());
        rows.collect()
    };
    assert_eq!(rows("2024-12-31"), ["1111.00", "-1.00", "-1110.00"]);
    assert_eq!(rows("2025-01-01"), ["11111.00", "-1111.00", "-10000.00"]);
}

#[test]
fn a_register_of_income_opens_as_the_general_ledger_does() {
    let company = books();
    let opened = |account, first: &str| {
        let period = day(first)..=day("2025-12-31");
        let ledger = company.general_ledger(period.clone()).unwrap();
        let mut accounts = ledger.accounts.iter();
        let account = AccountId::new(account);
        let in_ledger = accounts.find(|found| found.account == account);
        let in_ledger = in_ledger.map(|found| [found.opening, found.closing]);
        let register = company.register(account, period, Page::all()).unwrap();
        let register = register.unwrap();
        let in_register = [register.opening.home_amount, register.closing.home_amount];
        assert_eq!(in_ledger, Some(in_register), "{account} from {first}");
        in_register.map(|amount| amount.to_string())
    };
    assert_eq!(opened(INCOME, "2024-01-01"), ["0.00", "-11110.00"]);
    assert_eq!(opened(INCOME, "2024-01-02"), ["-10.00", "-11110.00"]);
    assert_eq!(opened(INCOME, "2025-01-01"), ["0.00", "-10000.00"]);
    assert_eq!(opened(RETAINED, "2024-06-30"), ["-1.00", "-1.00"]);
    assert_eq!(opened(RETAINED, "2025-01-01"), ["-1111.00", "-1111.00"]);
    assert_eq!(opened(1, "2025-01-01"), ["1111.00", "11111.00"]);
    // The whole register is the account's balance, as ever.
    let whole = company.register(AccountId::new(INCOME), Period::all(), Page::all());
    let whole = whole.unwrap().unwrap();
    assert_eq!(whole.opening.home_amount.to_string(), "0.00");
    assert_eq!(whole.closing.home_amount.to_string(), "-11111.00");
    assert_eq!(whole.period, Period::all());
}

#[test]
fn a_statement_is_made_month_by_month() {
    let company = books();
    let year = company.fiscal_year(day("2024-03-03")).unwrap();
    let months = year.months().map(|month| {
        let statement = company.profit_and_loss(month).unwrap();
        statement.net_income.to_string()
    });
    let months: Vec<String> = months.collect();
    assert_eq!(months.len(), 12);
    let earned = [&months[0], &months[5], &months[11], &months[1]];
    assert_eq!(earned, ["10.00", "100.00", "1000.00", "0.00"]);
}

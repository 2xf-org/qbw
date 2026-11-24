//! The examples, run on the synthetic company.

// Every function here runs inside a test, where a failed unwrap is a failed test.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

/// What `example` does with the synthetic company and the arguments.
fn run<'argument>(example: &str, arguments: impl IntoIterator<Item = &'argument str>) -> Output {
    let file = format!("{example}.qbw");
    let file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(file);
    fs::write(&file, qbw_synthetic::company::books().build()).unwrap();
    let cargo = ["run", "--quiet", "--package", "qbw"];
    Command::new(env!("CARGO"))
        .args(cargo)
        .args(["--example", example, "--"])
        .arg(file)
        .args(arguments)
        .output()
        .unwrap()
}

/// What `example` prints for the synthetic company and the arguments.
fn printed<'argument>(
    example: &str,
    arguments: impl IntoIterator<Item = &'argument str>,
) -> String {
    let output = run(example, arguments);
    let errors = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{errors}");
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn accounts_prints_the_active_accounts() {
    let printed = printed("accounts", None);
    let lines: Vec<&str> = printed.lines().collect();
    assert_eq!(lines.len(), 21);
    assert_eq!(lines[0], format!("{:<28} {:>12}", "Checking", "837.00"));
    assert_eq!(
        lines[12],
        format!("{:<28} {:>12}", "Sales:Parts", "-100.00")
    );
}

#[test]
fn customers_prints_balances_and_contacts() {
    let printed = printed("customers", None);
    let lines: Vec<&str> = printed.lines().collect();
    let ada = [
        "    555-0101",
        "    ada@ada-stores.example",
        "    Ada Stores",
        "    7 Mill Lane",
        "    Northport, NP 10002",
    ];
    assert_eq!(lines[0], format!("{:<28} {:>12}", "Ada Stores", "45.00"));
    assert_eq!(lines[1..6], ada);
    let annex = format!("{:<28} {:>12}", "Ada Stores:Annex", "0.00");
    assert_eq!(lines[6], annex);
}

#[test]
fn reorder_prints_what_is_to_be_ordered() {
    let printed = printed("reorder", None);
    let valve = format!("{:<28} {:>8} on hand {:>8} on order\n", "Valve", 8, 3);
    assert_eq!(printed, valve);
}

#[test]
fn stored_row_prints_every_column() {
    let printed = printed("stored_row", Some("Sales:Parts"));
    assert!(printed.starts_with("table abmc_account_user, page "));
    let sublevel = format!("{:<28} 1", "sublevel");
    assert!(printed.lines().any(|line| line == sublevel));
    assert_eq!(printed.lines().count(), 15);
}

#[test]
fn register_prints_the_rows_and_the_closing_balance() {
    let printed = printed("register", Some("Checking"));
    let lines: Vec<&str> = printed.lines().collect();
    assert_eq!(lines[1], "2023-06-05      501      -200.00       800.00");
    assert_eq!(lines.last(), Some(&"6 rows, 837.00"));
}

#[test]
fn balance_sheet_prints_both_sides() {
    let printed = printed("balance_sheet", Some("2024-06-30"));
    let has = |line: &str| printed.lines().any(|printed| printed == line);
    assert!(has(&format!(
        "{:<34} {:>12}",
        "  Retained Earnings", "130.00"
    )));
    assert!(has(&format!("{:<34} {:>12}", "Total assets", "1262.00")));
    let sides = format!("{:<34} {:>12}", "Total liabilities and equity", "1262.00");
    assert!(has(&sides));
}

#[test]
fn invoice_prints_the_form_and_what_is_left() {
    let printed = printed("invoice", Some("1041"));
    let lines: Vec<&str> = printed.lines().map(str::trim_end).collect();
    assert_eq!(
        lines,
        [
            "2023-11-15 Ada Stores",
            "Valve                        150.00",
            "Installation                  40.00",
            "Delivered to dock 1",
            "State tax                     12.00",
            "Total                        202.00",
            "Still to pay                  45.00",
        ]
    );
}

#[test]
fn find_prints_the_newest_first() {
    let printed = printed("find", Some("valve"));
    let credit = "2024-02-01 Credit Memo          CM-3        54.00";
    let lines: Vec<&str> = printed.lines().collect();
    assert_eq!(lines, [credit, "1 of 1"]);
}

#[test]
fn overview_prints_the_company_and_a_customer() {
    let printed = printed("overview", ["2024-06-30", "Ada Stores"]);
    let lines: Vec<&str> = printed.lines().collect();
    assert_eq!(lines[0], format!("{:<28} {:>12}", "Checking", "837.00"));
    assert_eq!(lines[2], format!("{:<28} {:>12}", "Customers owe", "45.00"));
    assert_eq!(lines[7], "Ada Stores");
    assert_eq!(
        lines[11],
        format!(
            "{:<12} {:>6} 2024-04-25 {:<20} {:>9}",
            "Invoice", "1043", "Accounts Receivable", "50.00"
        )
    );
    assert_eq!(lines.last(), Some(&"5 of 7"));
}

#[test]
fn check_prints_what_agrees_and_why_the_rest_does_not() {
    let output = run("check", None);
    // Somebody has to look at the balance, which may be out of date.
    assert_eq!(output.status.code(), Some(2));
    let printed = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = printed.lines().collect();
    let balances = "accounts with the balance QuickBooks keeps";
    assert_eq!(lines[5], format!("{balances:<48} {:>7} of {:>7}", 21, 22));
    let finding = [
        "",
        "accounts with the balance QuickBooks keeps: Company Card",
        "    -25.00 instead of -30.00",
        "    maybe QuickBooks' total is out of date",
    ];
    assert_eq!(lines[23..], finding);
}

#[test]
fn timings_prints_a_line_for_each_question() {
    let printed = printed("timings", None);
    assert!(printed.starts_with("the names "));
    assert_eq!(printed.lines().count(), 18);
}

#[test]
fn owed_prints_what_is_open_and_what_paid_it() {
    let printed = printed("owed", ["Ada Stores", "2024-06-30"]);
    let lines: Vec<&str> = printed.lines().collect();
    let expected = [
        "owes 45.00, of which 45.00 is overdue",
        "2023-11-15 Invoice 1041: 45.00 of 202.00",
        "    2024-01-20 paid by 100.00",
        "    2024-02-01 credited by 54.00",
        "    2024-01-20 discount from 2.00",
        "    2024-06-01 discount from 1.00",
    ];
    assert_eq!(lines, expected);
}

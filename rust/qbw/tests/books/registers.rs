//! Registers.

use qbw::chrono::NaiveDate;
use qbw::registers::{Register, RegisterRow};
use qbw::transactions::Cleared;
use qbw::{AccountId, NameId, OtherSide, Page, Period, Side, TransactionId};

use crate::{counted, support};

const CHECKING: AccountId = AccountId::new(1);

fn register(account: i32) -> Register {
    let company = support::books();
    company
        .register(AccountId::new(account), Period::all(), Page::all())
        .unwrap()
        .unwrap()
}

/// The line, the number, the amount and the balance of each row.
fn rows(register: &Register) -> Vec<(i32, Option<&str>, String, String)> {
    let row = |row: &RegisterRow| {
        (
            row.line.number(),
            row.amount.to_string(),
            row.balance.to_string(),
        )
    };
    let rows = register.rows.iter().map(|found| (found, row(found)));
    rows.map(|(found, (line, amount, balance))| (line, found.number.as_deref(), amount, balance))
        .collect()
}

fn row(
    line: i32,
    number: Option<&'static str>,
    amount: &str,
    balance: &str,
) -> (i32, Option<&'static str>, String, String) {
    (line, number, amount.to_owned(), balance.to_owned())
}

#[test]
fn a_register_runs_through_the_postings_of_an_account() {
    let register = register(1);
    assert_eq!(
        rows(&register),
        [
            row(101, Some("1"), "1000.00", "1000.00"),
            row(301, Some("501"), "-200.00", "800.00"),
            row(601, None, "100.00", "900.00"),
            row(1301, None, "-55.00", "845.00"),
            row(1701, Some("502"), "0.00", "845.00"),
            // The check waits to be printed and has no number.
            row(2301, None, "-8.00", "837.00"),
        ]
    );
    assert!(register.opening.amount.is_zero() && register.left_out.is_nothing());
    assert_eq!(
        (register.closing.amount.to_string(), register.row_count),
        ("837.00".to_owned(), 6)
    );
}

#[test]
fn a_row_says_who_and_what_and_on_which_other_side() {
    let register = register(1);
    let names: Vec<Option<i32>> = register
        .rows
        .iter()
        .map(|row| row.name.map(NameId::number))
        .collect();
    assert_eq!(names, [None, Some(201), None, None, Some(401), Some(202)]);
    let sides: Vec<OtherSide> = register.rows.iter().map(|row| row.other_side).collect();
    let sides_expected =
        [10, 7, 5, 2, 15, 9].map(|account| OtherSide::Account(AccountId::new(account)));
    assert_eq!(sides, sides_expected);
    let memos: Vec<Option<&str>> = register
        .rows
        .iter()
        .map(|row| row.memo.as_deref())
        .collect();
    let funds = Some("Funds transfer");
    assert_eq!(
        memos,
        [
            Some("Opening balance"),
            None,
            Some("Deposit"),
            funds,
            Some("VOID: Fuel"),
            None
        ]
    );
    let cleared: Vec<Cleared> = register.rows.iter().map(|row| row.cleared).collect();
    let reconciled = Cleared::Reconciled;
    assert_eq!(
        cleared,
        [
            reconciled,
            reconciled,
            reconciled,
            reconciled,
            reconciled,
            Cleared::Ticked
        ]
    );
    let reconciled_on: Option<NaiveDate> = register.rows[0].reconciled_on;
    assert_eq!(reconciled_on, "2024-05-31".parse().ok());
}

#[test]
fn a_row_on_another_line_has_the_number_of_its_transaction() {
    assert_eq!(
        rows(&register(3)),
        [
            row(401, Some("1041"), "202.00", "202.00"),
            row(502, Some("88"), "-100.00", "102.00"),
            row(503, Some("88"), "-2.00", "100.00"),
            row(701, Some("CM-3"), "-54.00", "46.00"),
            row(2401, None, "-1.00", "45.00"),
            row(2402, None, "0.00", "45.00"),
        ]
    );
}

#[test]
fn what_a_line_posts_besides_has_the_line_s_memo() {
    let register = register(14);
    let shown: Vec<_> = register
        .rows
        .iter()
        .map(|row| {
            (
                row.line.number(),
                row.number.as_deref(),
                row.memo.as_deref(),
            )
        })
        .collect();
    let valve = Some("Brass valve");
    assert_eq!(
        shown,
        [(402, Some("1041"), valve), (702, Some("CM-3"), valve)]
    );
}

#[test]
fn a_register_is_kept_in_the_account_s_currency() {
    let register = register(2);
    assert_eq!(
        rows(&register),
        [
            row(901, Some("W-9"), "100.00", "100.00"),
            row(1302, None, "50.00", "150.00"),
            // The rate changed, and the euros did not.
            row(2501, Some("2"), "0.00", "150.00"),
        ]
    );
    assert_eq!(register.closing.home_amount.to_string(), "180.00");
}

#[test]
fn a_page_of_a_period_goes_on_from_the_balance_before_it() {
    let company = support::books();
    let year = company.fiscal_year("2024-05-01".parse().unwrap()).unwrap();
    let register = company
        .register(CHECKING, year, Page::new(1, 2))
        .unwrap()
        .unwrap();
    assert_eq!(
        rows(&register),
        [
            row(1301, None, "-55.00", "845.00"),
            row(1701, Some("502"), "0.00", "845.00"),
        ]
    );
    assert_eq!(
        (register.opening.amount.to_string(), register.row_count),
        ("800.00".to_owned(), 4)
    );
    assert_eq!(register.closing.amount.to_string(), "837.00");

    let past_the_end = Page::new(9, 2);
    let register = company
        .register(CHECKING, year, past_the_end)
        .unwrap()
        .unwrap();
    assert_eq!((register.rows.len(), register.row_count), (0, 4));
    let before = Period::until("2023-02-28".parse().unwrap());
    let register = company
        .register(CHECKING, before, Page::all())
        .unwrap()
        .unwrap();
    assert!(register.rows.is_empty() && register.closing.amount.is_zero());
}

#[test]
fn only_the_rows_of_the_page_are_read() {
    let company = counted::books();
    company.ledger().unwrap();
    let fetched = || company.source().fetched();
    assert_eq!(fetched(), 0);
    let (always, page) = (Period::all(), Page::new(0, 2));
    company.register(CHECKING, always, page).unwrap().unwrap();
    assert_eq!(fetched(), 2);
    company
        .register(AccountId::new(3), always, page)
        .unwrap()
        .unwrap();
    assert_eq!(fetched(), 4);
}

#[test]
fn a_register_of_a_card_is_read_as_the_card_is_kept() {
    let company = support::books();
    let card = company.accounts().unwrap().named("Company Card").unwrap();
    let register = register(card.id.number());
    let columns: Vec<[String; 3]> = register
        .rows
        .iter()
        .map(|row| {
            let columns = [
                card.account_type.increase(row.amount),
                card.account_type.decrease(row.amount),
                Some(card.account_type.as_kept(row.balance)),
            ];
            columns.map(|amount| amount.map(|amount| amount.to_string()).unwrap_or_default())
        })
        .collect();
    let row = |columns: [&str; 3]| columns.map(str::to_owned);
    // A charge, and what was brought back.
    let expected = [row(["30.00", "", "30.00"]), row(["", "5.00", "25.00"])];
    assert_eq!(columns, expected);
    let closing = card.account_type.as_kept(register.closing);
    assert_eq!(closing.amount.to_string(), "25.00");
    assert!(closing.left_out.is_nothing());
}

#[test]
fn postings_have_the_texts_of_their_registers() {
    let company = counted::books();
    let register = company.register(CHECKING, Period::all(), Page::all());
    let register = register.unwrap().unwrap();
    let ledger = company.ledger().unwrap();
    let postings: Vec<_> = ledger.of_account(CHECKING).collect();
    let before = company.source().fetched();
    let texts = company.posting_texts(&postings[..3]).unwrap();
    assert_eq!(company.source().fetched() - before, 3);
    assert!(texts.left_out.is_nothing() && texts.rows.len() == 3);
    for (text, row) in texts.rows.iter().zip(&register.rows[..3]) {
        assert_eq!((&text.number, &text.memo), (&row.number, &row.memo));
        assert_eq!(text.reconciled_on, row.reconciled_on);
    }
    assert_eq!(texts.rows[0].reconciled_on, "2024-05-31".parse().ok());
    assert!(company.posting_texts(&[]).unwrap().rows.is_empty());
}

#[test]
fn the_texts_of_a_name_s_postings_are_those_of_their_transactions() {
    let company = support::books();
    let ada = NameId::new(101);
    let ledger = company.name_ledger(Side::Receivable, ada, Period::all());
    let ledger = ledger.unwrap().unwrap();
    let postings: Vec<_> = ledger.rows.iter().map(|row| row.posting).collect();
    let texts = company.posting_texts(&postings).unwrap();
    let invoice = &texts.rows[0];
    assert_eq!(invoice.number.as_deref(), Some("1041"));
    assert_eq!(invoice.memo.as_deref(), Some("Spring order"));
}

#[test]
fn a_posting_that_was_lost_since_has_no_texts() {
    let company = support::damaged_books();
    let card = company
        .ledger()
        .unwrap()
        .of_transaction(TransactionId::new(1000));
    let card: Vec<_> = card.collect();
    let texts = company.posting_texts(&card).unwrap();
    // The header cannot be read.
    assert!(texts.rows.iter().all(|text| text.number.is_none()));
    assert!(!texts.left_out.is_nothing());
}

#[test]
fn an_account_that_nothing_is_posted_to_has_no_rows() {
    for account in [11, 19] {
        let register = register(account);
        assert!(register.rows.is_empty(), "account {account}");
        assert!(register.closing.home_amount.is_zero());
    }
    // An account that was deleted, and one that never was.
    for account in [22, 99] {
        let company = support::books();
        let register = company.register(AccountId::new(account), Period::all(), Page::all());
        assert_eq!(register.unwrap(), None, "account {account}");
    }
}

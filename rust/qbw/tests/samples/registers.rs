//! Registers against the ledger and the headers.

use std::collections::HashMap;

use qbw::chrono::NaiveDate;
use qbw::registers::Register;
use qbw::sa17::Columns;
use qbw::{AccountId, Company, Page, Period};

use crate::agreement::Agreement;
use crate::support;

const PAGE: usize = 50;

/// The first, a middle and the last page of every account's register.
fn pages(company: &Company) -> Vec<(Page, Register)> {
    let accounts = company.accounts().unwrap().iter();
    let pages = accounts.flat_map(|account| {
        let rows = company.ledger().unwrap().of_account(account.id).len();
        let mut starts = vec![0, rows / 2, rows.saturating_sub(PAGE)];
        starts.dedup();
        starts
            .into_iter()
            .map(move |start| (account.id, Page::new(start, PAGE)))
    });
    let read = |(account, page): (AccountId, Page)| {
        let register = company.register(account, Period::all(), page);
        (page, register.unwrap().unwrap())
    };
    pages.map(read).collect()
}

/// The numbers of all transactions, from their headers.
fn numbers(company: &Company) -> HashMap<i32, Option<String>> {
    let mut numbers = HashMap::new();
    let tables = company.source().tables();
    for table in tables.filter(|table| table.name().ends_with("_header")) {
        let columns = ["transaction_id", "doc_num", "target_id"];
        let Ok(columns) = Columns::new(table.definition(), columns) else {
            continue;
        };
        for row in table.scan(&columns) {
            let row = row.unwrap();
            let number = row.read::<Option<String>>(1).unwrap();
            let number = number.filter(|number| !number.is_empty());
            numbers.insert(row.read(0).unwrap(), number);
        }
    }
    numbers
}

/// The rows after which the day ends, each with its day and its balance.
fn day_ends(company: &Company, page: Page, register: &Register) -> Vec<(NaiveDate, bool)> {
    let ledger = company.ledger().unwrap();
    let is_last = page.start() + register.rows.len() == register.row_count;
    let next = register.rows.iter().skip(1).map(Some);
    let next = next.chain(is_last.then_some(None));
    let rows = register.rows.iter().zip(next);
    let ends = rows.filter(|(row, next)| next.is_none_or(|next| next.date != row.date));
    ends.map(|(row, _)| {
        let balance = ledger.balance(register.account, row.date).unwrap().amount;
        (row.date, balance == row.balance)
    })
    .collect()
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn running_balances_are_the_ledger_s_at_the_end_of_each_day() {
    for company in support::samples() {
        let mut agreement = Agreement::default();
        for (page, register) in pages(&company) {
            for (_, agrees) in day_ends(&company, page, &register) {
                agreement.add(agrees);
            }
        }
        agreement.check("day ends in registers with the ledger's balance");
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn rows_have_the_numbers_of_the_headers() {
    for company in support::samples() {
        let numbers = numbers(&company);
        let mut agreement = Agreement::default();
        for (_, register) in pages(&company) {
            for row in &register.rows {
                agreement.add(numbers.get(&row.transaction.number()) == Some(&row.number));
            }
        }
        agreement.check("rows with the number of the header");
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn a_page_of_the_largest_register_is_complete() {
    for company in support::samples() {
        let ledger = company.ledger().unwrap();
        let accounts = company.accounts().unwrap().iter().map(|account| account.id);
        let largest = accounts.max_by_key(|account| ledger.of_account(*account).len());
        let (largest, rows) = (largest.unwrap(), ledger.of_account(largest.unwrap()).len());
        for start in [0, rows / 2, rows - 100] {
            let page = Page::new(start, 100);
            let register = company
                .register(largest, Period::all(), page)
                .unwrap()
                .unwrap();
            assert_eq!((register.rows.len(), register.row_count), (100, rows));
            assert!(register.left_out.is_nothing());
        }
    }
}

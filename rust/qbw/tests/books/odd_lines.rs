//! Ledgers of books whose lines are not as they should be.

use qbw::sa17::ValueProblem;
use qbw::{AccountId, AsOf, Company, CurrencyId, Money, NameId, Problem};
use qbw_synthetic::{Cell, SyntheticColumn, SyntheticRow, SyntheticTable};

use crate::small_books;
use crate::small_books::BANK;
use crate::support;

const ENTRIES: &[(&str, i32, &str)] = &[("2024-01-01", 2, "5.00")];

/// The company of [`ENTRIES`] with another table.
fn company_with(table: SyntheticTable) -> Company {
    let database = small_books::database(&[BANK, (2, 10, 0, 0)], ENTRIES);
    support::company(database.table(table))
}

fn bank_balance(company: &Company) -> String {
    let ledger = company.ledger().unwrap();
    ledger
        .balance(AccountId::new(1), AsOf::Latest)
        .unwrap()
        .home_amount
        .to_string()
}

#[test]
fn what_a_lost_line_posts_besides_is_left_out() {
    let stock = SyntheticTable::new("abmc_general_journal_inventory_lineitem")
        .column(SyntheticColumn::new("target_id", 2))
        .column(SyntheticColumn::new("target_type", 2))
        .column(SyntheticColumn::new("account_id", 2))
        .column(SyntheticColumn::new("amount_amt", 3).width(20).scale(5));
    let row = |line| {
        let amount = Cell::decimal("3.00");
        SyntheticRow::new([
            Cell::integer(line),
            Cell::integer(1),
            Cell::integer(1),
            amount,
        ])
    };
    // Of line 10, which is there, and of line 99.
    let company = company_with(stock.row(row(10)).row(row(99)));
    assert_eq!(bank_balance(&company), "8.00");
    let ledger = company.ledger().unwrap();
    assert_eq!((ledger.postings().len(), ledger.left_out().rows), (3, 1));
    let problems: Vec<&Problem> = ledger
        .damage()
        .iter()
        .map(|damage| &damage.problem)
        .collect();
    assert_eq!(problems, [&Problem::NoLine]);
}

#[test]
fn an_amount_that_the_ledger_cannot_keep_is_left_out() {
    let lines = small_books::lines("abmc_check_lineitem")
        .row(small_books::line(90, 9, "2024-01-02", 1, 91, "0.000001"))
        .row(small_books::line(91, 9, "2024-01-02", 1, 0, "4.00"));
    let company = company_with(lines);
    assert_eq!(bank_balance(&company), "9.00");
    let [damage] = company.ledger().unwrap().damage() else {
        panic!("one line is left out");
    };
    let Problem::Column { column, found } = &damage.problem else {
        panic!("the amount is the problem: {damage}");
    };
    assert_eq!(*column, "amount_amt");
    assert!(matches!(found, ValueProblem::Range { .. }));
}

#[test]
fn a_table_of_lines_that_no_release_had_is_counted() {
    let lines = small_books::lines("abmc_rebate_lineitem");
    let with_rows = lines.row(small_books::line(90, 9, "2024-01-02", 1, 0, "4.00"));
    let company = company_with(with_rows);
    let left_out = company.ledger().unwrap().left_out();
    assert_eq!((left_out.rows, left_out.unread_tables), (0, 1));
    // One without rows posts nothing.
    let empty = company_with(small_books::lines("abmc_rebate_lineitem"));
    assert!(empty.ledger().unwrap().left_out().is_nothing());
}

#[test]
fn a_vendor_in_another_currency_is_owed_sales_tax_in_it() {
    const TAX: i32 = 2;
    let vendors = SyntheticTable::new("abmc_vendor_user")
        .column(SyntheticColumn::new("vendor_id", 2))
        .column(SyntheticColumn::new("currency_id", 2))
        .row(SyntheticRow::new([Cell::integer(7), Cell::integer(2)]));
    let mut line = Vec::from(small_books::cells(90, 9, "2024-01-02", TAX, 0, "-12.00"));
    line.pop();
    line.extend([Cell::decimal("-10.00"), Cell::integer(7)]);
    let lines = small_books::lines("abmc_bill_lineitem")
        .column(SyntheticColumn::new("vendor_id", 2))
        .row(SyntheticRow::new(line));
    // The account is QuickBooks' own for sales tax, kept in the company's currency.
    let database = small_books::database(&[BANK, (TAX, 7, 0, 4)], &[]);
    let database = database.table(vendors).table(lines);
    let company = support::company(database);
    let ledger = company.ledger().unwrap();
    let owed = ledger.vendor_balance(NameId::new(7), AsOf::Latest).unwrap();
    let expected = (
        Money::new("-10".parse().unwrap(), CurrencyId::new(2)),
        "-12.00".to_owned(),
    );
    assert_eq!((owed.amount, owed.home_amount.to_string()), expected);
    assert_eq!(
        ledger.vendor_balances(AsOf::Latest).get(&NameId::new(7)),
        Some(&owed)
    );
    let account = ledger.balance(AccountId::new(TAX), AsOf::Latest).unwrap();
    assert_eq!(account.amount, account.home_amount);
}

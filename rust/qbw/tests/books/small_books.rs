//! Small companies, for what the books of the others do not have.

use qbw::Company;
use qbw_synthetic::{Cell, SyntheticColumn, SyntheticDatabase, SyntheticRow, SyntheticTable};

use crate::support;

/// An account: its id, its type, the account above it and what QuickBooks uses
/// it for.
pub(crate) type Account = (i32, i16, i32, i16);

/// A bank, of type 0.
pub(crate) const BANK: Account = (1, 0, 0, 0);

/// A company with `accounts` and a journal entry of two lines for each of
/// `entries`: the day, the account of the credit and the amount, which is a
/// debit of the bank.
pub(crate) fn company(accounts: &[Account], entries: &[(&str, i32, &str)]) -> Company {
    support::company(database(accounts, entries))
}

/// The company, whose years start with the month `start`.
pub(crate) fn with_years(
    start: &str,
    accounts: &[Account],
    entries: &[(&str, i32, &str)],
) -> Company {
    support::company(with_preferences(
        &[("FISCALBEGIN", start)],
        accounts,
        entries,
    ))
}

/// The database of [`company`].
pub(crate) fn database(accounts: &[Account], entries: &[(&str, i32, &str)]) -> SyntheticDatabase {
    with_preferences(&[], accounts, entries)
}

fn with_preferences(
    preferences: &[(&str, &str)],
    accounts: &[Account],
    entries: &[(&str, i32, &str)],
) -> SyntheticDatabase {
    let tables = [chart(accounts), totals(accounts), journal_lines(entries)];
    let preferences = support::preferences(preferences);
    tables
        .into_iter()
        .fold(preferences, SyntheticDatabase::table)
}

fn chart(accounts: &[Account]) -> SyntheticTable {
    let mut table = SyntheticTable::new("abmc_account_user")
        .column(SyntheticColumn::new("account_id", 2))
        .column(SyntheticColumn::new("name", 9).width(32))
        .column(SyntheticColumn::new("parent_id", 2))
        .column(SyntheticColumn::new("type", 1).width(2))
        .column(SyntheticColumn::new("is_hidden_bool", 24).width(1))
        .column(SyntheticColumn::new("is_deleted_bool", 24).width(1));
    for &(id, kind, parent, _) in accounts {
        let (id, parent) = (Cell::integer(id), Cell::integer(parent));
        let off = Cell::bit(false);
        let row = [
            id,
            Cell::text("Account"),
            parent,
            Cell::small_int(kind),
            off.clone(),
            off,
        ];
        table = table.row(SyntheticRow::new(row));
    }
    table
}

fn totals(accounts: &[Account]) -> SyntheticTable {
    let mut table = SyntheticTable::new("abmc_account_internal")
        .column(SyntheticColumn::new("account_id", 2))
        .column(SyntheticColumn::new("special_type", 1).width(2))
        .column(SyntheticColumn::new("display_index_next", 2));
    for &(id, _, _, special) in accounts {
        let row = [
            Cell::integer(id),
            Cell::small_int(special),
            Cell::integer(0),
        ];
        table = table.row(SyntheticRow::new(row));
    }
    table
}

fn journal_lines(entries: &[(&str, i32, &str)]) -> SyntheticTable {
    let mut table = lines("abmc_general_journal_lineitem");
    for (number, &(day, account, amount)) in (1..).zip(entries) {
        let (own, other) = (number * 10, number * 10 + 1);
        let credit = format!("-{amount}");
        table = table.row(line(own, number, day, 1, other, amount));
        table = table.row(line(other, number, day, account, 0, &credit));
    }
    table
}

/// A table of lines, with the columns that every one has.
pub(crate) fn lines(name: &str) -> SyntheticTable {
    SyntheticTable::new(name)
        .column(SyntheticColumn::new("target_id", 2))
        .column(SyntheticColumn::new("transaction_id", 2))
        .column(SyntheticColumn::new("transaction_date", 6))
        .column(SyntheticColumn::new("transaction_view_type", 1).width(2))
        .column(SyntheticColumn::new("account_id", 2))
        .column(SyntheticColumn::new("is_source_bool", 24).width(1))
        .column(SyntheticColumn::new("sibling_account_id", 2))
        .column(SyntheticColumn::new("next_target_id", 2))
        .column(SyntheticColumn::new("amount_amt", 3).width(20).scale(5))
        .column(
            SyntheticColumn::new("foreign_amount_amt", 3)
                .width(20)
                .scale(5),
        )
}

/// A line of a journal entry, which is its own line where another comes after it.
pub(crate) fn line(
    id: i32,
    transaction: i32,
    day: &str,
    account: i32,
    next: i32,
    amount: &str,
) -> SyntheticRow {
    SyntheticRow::new(cells(id, transaction, day, account, next, amount))
}

/// The fields of [`line`], of which the last is the amount in the other currency.
pub(crate) fn cells(
    id: i32,
    transaction: i32,
    day: &str,
    account: i32,
    next: i32,
    amount: &str,
) -> [Cell; 10] {
    let amount = Cell::decimal(amount);
    [
        Cell::integer(id),
        Cell::integer(transaction),
        Cell::day(day),
        Cell::small_int(8),
        Cell::integer(account),
        Cell::bit(next != 0),
        Cell::integer(0),
        Cell::integer(next),
        amount.clone(),
        amount,
    ]
}

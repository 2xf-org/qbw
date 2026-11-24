//! A company laid out as QuickBooks Desktop lays one out.
//!
//! The tables have the names, the types and the keys that they have in a
//! company file, and the columns that qbw reads. Every value is typed by
//! hand: amounts, signs, the accounts on the other side, the order of the
//! lines, links and balances. Nothing is worked out, so that a reader is
//! tested against the values and not against another reader.
//!
//! Amounts are debits where positive. The company keeps its books in currency
//! 1 and trades in currency 2 as well. The transaction numbered 4 has the key
//! 400, and its lines 401 and up.

mod accounts;
mod banking;
mod columns;
mod inventory;
mod items;
mod journals;
mod links;
mod lists;
mod literal;
mod names;
mod one_currency;
mod purchases;
mod sales;
mod settings;
mod table;
mod unused_forms;

use crate::company::books::table::Table;
use crate::synthetic_database::SyntheticDatabase;

/// The user that owns a company's tables.
const OWNER: &str = "QBAdminGroup";

/// A company's books: 25 transactions over two years, in two currencies, with
/// nested accounts, a job, stock, a partial payment with a discount, credits,
/// orders, a pending, a memorized and a voided transaction.
#[must_use]
pub fn books() -> SyntheticDatabase {
    write(tables())
}

/// [`books`] with three things damaged: the line of invoice 1041 for the
/// installation and the header of the card charge cannot be read, and the
/// chain of the bill's lines leads to a line that is not there.
#[must_use]
pub fn damaged_books() -> SyntheticDatabase {
    let tables = tables().into_iter().map(|mut table| {
        match table.name() {
            "abmc_invoice_lineitem" => table.make_unreadable("target_id", 403),
            "abmc_credit_cardcharge_header" => table.make_unreadable("transaction_id", 1000),
            "abmc_bill_lineitem" => table.set("target_id", 201, "next_target_id", 999),
            _ => {}
        }
        table
    });
    write(tables.collect())
}

/// A small company that keeps its books in one currency and uses no other: a
/// customer and a vendor, an invoice and a bill that are paid in part, and what
/// the bank opened with. What [`books`] has, too, has the same key and the same
/// name or number: `Ada Stores`, `Checking`, invoice 1041, bill 7781.
#[must_use]
pub fn one_currency_books() -> SyntheticDatabase {
    write(one_currency::tables(tables()))
}

fn tables() -> Vec<Table> {
    let subjects = [
        settings::tables(),
        accounts::tables(),
        names::tables(),
        items::tables(),
        lists::tables(),
        sales::tables(),
        purchases::tables(),
        banking::tables(),
        journals::tables(),
        inventory::tables(),
        links::tables(),
        unused_forms::tables(),
    ];
    subjects.into_iter().flatten().collect()
}

fn write(tables: Vec<Table>) -> SyntheticDatabase {
    let database = SyntheticDatabase::new().owner(OWNER);
    let tables = tables.iter().map(Table::write);
    tables.fold(database, SyntheticDatabase::table)
}

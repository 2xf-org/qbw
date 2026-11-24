//! The chart of accounts.

use crate::company::books::literal::{Literal, row};
use crate::company::books::table::Table;

pub(super) fn tables() -> Vec<Table> {
    vec![accounts(), totals()]
}

/// The keys of the accounts are in the order they were made in. The list has
/// them in the order that `totals` chains them.
fn accounts() -> Table {
    Table::new(
        "abmc_account_user",
        &["
            account_id integer key
            name varchar(32) null
            parent_id integer null = 0
            sublevel smallint null = 0
            type smallint null
            account_num varchar(8) null
            description varchar(201) null
            currency_id integer null -> abmc_currency = 1
            class_id integer null -> abmc_class
            tax_line_id_num integer null = 0
            is_hidden_bool bit
            is_deleted_bool bit
            timestamp_tms integer null = 0
            modtimestamp_tms integer null = 0
        "],
    )
    .row(row! {
        "account_id" => 1, "name" => "Checking", "type" => 0, "account_num" => "1000",
        "description" => "Everyday account",
        "timestamp_tms" => 1672565400, "modtimestamp_tms" => 1672569000,
    })
    .row(row! {
        "account_id" => 2, "name" => "Euro bank", "type" => 0, "account_num" => "1010",
        "currency_id" => 2,
    })
    .row(row! { "account_id" => 3, "name" => "Accounts Receivable", "type" => 1 })
    .row(row! {
        "account_id" => 4, "name" => "Accounts Receivable - EUR", "type" => 1, "currency_id" => 2,
    })
    .row(row! { "account_id" => 5, "name" => "Undeposited Funds", "type" => 2 })
    .row(row! { "account_id" => 6, "name" => "Inventory Asset", "type" => 2 })
    .row(row! { "account_id" => 7, "name" => "Accounts Payable", "type" => 5 })
    .row(row! { "account_id" => 8, "name" => "Company Card", "type" => 6 })
    .row(row! { "account_id" => 9, "name" => "Sales Tax Payable", "type" => 7 })
    .row(row! { "account_id" => 10, "name" => "Opening Balance Equity", "type" => 9 })
    .row(row! { "account_id" => 11, "name" => "Retained Earnings", "type" => 9 })
    .row(row! {
        "account_id" => 12, "name" => "Sales", "type" => 10, "account_num" => "4000",
        "tax_line_id_num" => 1261,
    })
    .row(row! {
        "account_id" => 13, "name" => "Parts", "parent_id" => 12, "sublevel" => 1, "type" => 10,
    })
    .row(row! { "account_id" => 14, "name" => "Cost of Goods Sold", "type" => 11 })
    .row(row! { "account_id" => 15, "name" => "Office", "type" => 12 })
    .row(row! {
        "account_id" => 16, "name" => "Supplies", "parent_id" => 15, "sublevel" => 1, "type" => 12,
    })
    .row(row! { "account_id" => 17, "name" => "Exchange Gain or Loss", "type" => 14 })
    .row(row! { "account_id" => 18, "name" => "Discounts Given", "type" => 12 })
    .row(row! { "account_id" => 19, "name" => "Purchase Orders", "type" => 15 })
    .row(row! { "account_id" => 20, "name" => "Sales Orders", "type" => 15 })
    .row(row! { "account_id" => 21, "name" => "Estimates", "type" => 15 })
    // A deleted account keeps its key and loses everything else.
    .row(row! {
        "account_id" => 22, "type" => 0, "is_deleted_bool" => true, "currency_id" => Literal::Null,
    })
    .row(
        row! { "account_id" => 23, "name" => "Old Savings", "type" => 0, "is_hidden_bool" => true },
    )
}

fn totals() -> Table {
    Table::new(
        "abmc_account_internal",
        &["
            account_id integer key -> abmc_account_user
            special_type smallint null = 0
            ending_balance_amt numeric(20,5) null
            ending_balance_home_amt numeric(20,5) null
            reconcile_statement_date date null
            reconcile_statement_amt numeric(20,5) null
            display_index_next integer
        "],
    )
    .row(row! {
        "account_id" => 1, "display_index_next" => -2,
        "ending_balance_amt" => "837.00", "ending_balance_home_amt" => "837.00",
        "reconcile_statement_date" => "2024-05-31", "reconcile_statement_amt" => "845.00",
    })
    .row(row! {
        "account_id" => 2, "display_index_next" => 23,
        "ending_balance_amt" => "150.00", "ending_balance_home_amt" => "180.00",
    })
    .row(row! {
        "account_id" => 3, "special_type" => 2, "display_index_next" => 4,
        "ending_balance_amt" => "45.00", "ending_balance_home_amt" => "45.00",
    })
    .row(row! {
        "account_id" => 4, "special_type" => 2, "display_index_next" => 5,
        "ending_balance_amt" => "0.00", "ending_balance_home_amt" => "0.00",
    })
    .row(row! {
        "account_id" => 5, "special_type" => 5, "display_index_next" => 6,
        "ending_balance_amt" => "0.00", "ending_balance_home_amt" => "0.00",
    })
    .row(row! {
        "account_id" => 6, "special_type" => 15, "display_index_next" => 7,
        "ending_balance_amt" => "200.00", "ending_balance_home_amt" => "200.00",
    })
    .row(row! {
        "account_id" => 7, "special_type" => 3, "display_index_next" => 8,
        "ending_balance_amt" => "-20.00", "ending_balance_home_amt" => "-20.00",
    })
    // Out of date: the card owes 25.
    .row(row! {
        "account_id" => 8, "display_index_next" => 9,
        "ending_balance_amt" => "-30.00", "ending_balance_home_amt" => "-30.00",
    })
    .row(row! {
        "account_id" => 9, "special_type" => 4, "display_index_next" => 10,
        "ending_balance_amt" => "0.00", "ending_balance_home_amt" => "0.00",
    })
    .row(row! {
        "account_id" => 10, "special_type" => 1, "display_index_next" => 11,
        "ending_balance_amt" => "-1000.00", "ending_balance_home_amt" => "-1000.00",
    })
    .row(row! {
        "account_id" => 11, "special_type" => 8, "display_index_next" => 12,
        "ending_balance_amt" => "0.00",
    })
    .row(row! {
        "account_id" => 12, "display_index_next" => 13,
        "ending_balance_amt" => "-150.00", "ending_balance_home_amt" => "-150.00",
    })
    .row(row! {
        "account_id" => 13, "display_index_next" => 14,
        "ending_balance_amt" => "-100.00", "ending_balance_home_amt" => "-100.00",
    })
    .row(row! {
        "account_id" => 14, "special_type" => 16, "display_index_next" => 15,
        "ending_balance_amt" => "40.00", "ending_balance_home_amt" => "40.00",
    })
    .row(row! {
        "account_id" => 15, "display_index_next" => 16,
        "ending_balance_amt" => "-20.00", "ending_balance_home_amt" => "-20.00",
    })
    .row(row! {
        "account_id" => 16, "display_index_next" => 18,
        "ending_balance_amt" => "25.00", "ending_balance_home_amt" => "25.00",
    })
    .row(row! {
        "account_id" => 17, "special_type" => 24, "display_index_next" => 19,
        "ending_balance_amt" => "-15.00", "ending_balance_home_amt" => "-15.00",
    })
    .row(row! {
        "account_id" => 18, "display_index_next" => 17,
        "ending_balance_amt" => "3.00", "ending_balance_home_amt" => "3.00",
    })
    .row(row! {
        "account_id" => 19, "special_type" => 10, "display_index_next" => 20,
        "ending_balance_amt" => "0.00",
    })
    .row(row! {
        "account_id" => 20, "special_type" => 20, "display_index_next" => 21,
        "ending_balance_amt" => "0.00",
    })
    .row(row! {
        "account_id" => 21, "special_type" => 14, "display_index_next" => 22,
        "ending_balance_amt" => "0.00",
    })
    // A deleted account can keep what it was for.
    .row(row! { "account_id" => 22, "special_type" => 8, "display_index_next" => 1 })
    .row(row! {
        "account_id" => 23, "display_index_next" => 3,
        "ending_balance_amt" => "0.00", "ending_balance_home_amt" => "0.00",
    })
}

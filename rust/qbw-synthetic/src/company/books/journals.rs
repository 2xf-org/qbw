//! Journal entries.

use crate::company::books::columns::{
    self, CLASS, CLEARING, CUSTOMER, DISCOUNT, EMPLOYEE_AND_OTHER, HEADER_NAME_TYPE, IS_SPLIT,
    ITEM, LINE_NAME_TYPE, OPEN_BALANCE, PRICE, VENDOR,
};
use crate::company::books::literal::row;
use crate::company::books::table::Table;

pub(super) fn tables() -> Vec<Table> {
    vec![
        journals(),
        journal_lines(),
        columns::inventory_lines("abmc_general_journal"),
    ]
}

fn journals() -> Table {
    let own = "
        is_home_currency_adjustment_bool bit
        is_shown_in_home_currency_bool bit
    ";
    columns::headers(
        "abmc_general_journal",
        &[CUSTOMER, VENDOR, EMPLOYEE_AND_OTHER, HEADER_NAME_TYPE, own],
    )
    .row(row! {
        "transaction_id" => 100, "target_id" => 101, "transaction_date" => "2023-03-01",
        "doc_num" => "1", "memo" => "Opening balance", "trans_num" => 1,
        "timestamp_tms" => 1677672000, "modtimestamp_tms" => 1677672000,
        "abm_id" => "TXN0000000000100",
    })
    // The euro account is worth 5 more at the day's rate. It holds no more euros.
    .row(row! {
        "transaction_id" => 2500, "target_id" => 2501, "transaction_date" => "2024-06-30",
        "doc_num" => "2", "memo" => "Rate of June", "is_home_currency_adjustment_bool" => true,
        "is_shown_in_home_currency_bool" => true, "trans_num" => 25,
        "abm_id" => "TXN0000000002500",
    })
}

fn journal_lines() -> Table {
    let own = "
        is_adjustment_bool bit
    ";
    columns::lines(
        "abmc_general_journal",
        &[
            IS_SPLIT,
            ITEM,
            CUSTOMER,
            VENDOR,
            EMPLOYEE_AND_OTHER,
            LINE_NAME_TYPE,
            CLASS,
            PRICE,
            DISCOUNT,
            OPEN_BALANCE,
            CLEARING,
            own,
        ],
    )
    .rows(
        row! {
            "transaction_id" => 100, "transaction_date" => "2023-03-01",
            "transaction_view_type" => 8, "doc_num" => "1", "memo" => "Opening balance",
        },
        [
            row! {
                "target_id" => 101, "is_source_bool" => true, "next_target_id" => 102,
                "account_id" => 1, "sibling_account_id" => 10, "amount_amt" => "1000.00",
                "foreign_amount_amt" => "1000.00", "cleared_type" => 2,
                "reconcile_date" => "2024-05-31",
            },
            row! {
                "target_id" => 102, "next_target_id" => 0, "account_id" => 10,
                "sibling_account_id" => 1, "amount_amt" => "-1000.00",
                "foreign_amount_amt" => "-1000.00",
            },
        ],
    )
    .rows(
        row! {
            "transaction_id" => 2500, "transaction_date" => "2024-06-30",
            "transaction_view_type" => 8, "doc_num" => "2", "memo" => "Rate of June",
        },
        [
            row! {
                "target_id" => 2501, "is_source_bool" => true, "next_target_id" => 2502,
                "account_id" => 2, "sibling_account_id" => 17, "amount_amt" => "5.00",
                "foreign_amount_amt" => "0.00", "is_adjustment_bool" => true,
            },
            row! {
                "target_id" => 2502, "next_target_id" => 0, "account_id" => 17,
                "sibling_account_id" => 2, "amount_amt" => "-5.00",
                "foreign_amount_amt" => "0.00", "is_adjustment_bool" => true,
            },
        ],
    )
}

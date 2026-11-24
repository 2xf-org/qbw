//! Stock that is counted and built, and the history of it.

use crate::company::books::columns::{
    self, BIN, CLASS, CLEARING, CUSTOMER, IS_ARAP, IS_SPLIT, ITEM, LINE_NAME_TYPE, PART_NUMBER,
    PRICE, PRINT_LATER,
};
use crate::company::books::literal::row;
use crate::company::books::table::Table;

pub(super) fn tables() -> Vec<Table> {
    vec![
        adjustments(),
        adjustment_lines(),
        columns::inventory_lines("abmc_inventory_adjust"),
        builds(),
        build_lines(),
        columns::inventory_lines("abmc_item_assembly"),
        history(),
    ]
}

fn adjustments() -> Table {
    let own = "
        site_id integer null -> abmc_site
    ";
    columns::headers("abmc_inventory_adjust", &[own]).row(row! {
        "transaction_id" => 2100, "target_id" => 2101, "transaction_date" => "2024-05-05",
        "doc_num" => "A-1", "memo" => "Found on the shelf", "trans_num" => 21,
        "abm_id" => "TXN0000000002100",
    })
}

fn adjustment_lines() -> Table {
    let own = "
        open_qnty numeric(20,5) null
        is_value_adjust_bool bit
    ";
    columns::lines(
        "abmc_inventory_adjust",
        &[
            IS_SPLIT,
            ITEM,
            BIN,
            CUSTOMER,
            LINE_NAME_TYPE,
            CLASS,
            PRICE,
            CLEARING,
            own,
        ],
    )
    .rows(
        row! {
            "transaction_id" => 2100, "transaction_date" => "2024-05-05",
            "transaction_view_type" => 18,
        },
        [
            row! {
                "target_id" => 2101, "is_source_bool" => true, "next_target_id" => 2102,
                "account_id" => 15, "sibling_account_id" => 6, "amount_amt" => "-20.00",
                "foreign_amount_amt" => "-20.00", "doc_num" => "A-1",
                "memo" => "Found on the shelf",
            },
            row! {
                "target_id" => 2102, "next_target_id" => 0, "account_id" => 6,
                "sibling_account_id" => 15, "amount_amt" => "20.00",
                "foreign_amount_amt" => "20.00", "item_id" => 502, "quantity_qnty" => "1.00",
            },
        ],
    )
}

fn builds() -> Table {
    let own = "
        custom_style_id integer null -> abmc_custom_style
        item_id integer null
    ";
    columns::headers("abmc_item_assembly", &[own, PRINT_LATER, IS_ARAP]).row(row! {
        "transaction_id" => 2200, "target_id" => 2201, "transaction_date" => "2024-05-10",
        "doc_num" => "B-1", "item_id" => 507, "trans_num" => 22, "abm_id" => "TXN0000000002200",
    })
}

fn build_lines() -> Table {
    let clearing = "
        cleared_type smallint null = 0
        reimbursed_type smallint null = 0
    ";
    columns::lines(
        "abmc_item_assembly",
        &[IS_SPLIT, ITEM, BIN, PART_NUMBER, clearing],
    )
    .rows(
        row! {
            "transaction_id" => 2200, "transaction_date" => "2024-05-10",
            "transaction_view_type" => 29,
        },
        [
            row! {
                "target_id" => 2201, "is_source_bool" => true, "next_target_id" => 2202,
                "account_id" => 6, "sibling_account_id" => 6, "amount_amt" => "40.00",
                "foreign_amount_amt" => "40.00", "doc_num" => "B-1", "item_id" => 507,
                "quantity_qnty" => "1.00",
            },
            row! {
                "target_id" => 2202, "next_target_id" => 0, "account_id" => 6,
                "sibling_account_id" => 6, "amount_amt" => "-40.00",
                "foreign_amount_amt" => "-40.00", "memo" => "Brass valve", "item_id" => 502,
                "quantity_qnty" => "-2.00",
            },
        ],
    )
}

/// What every item in stock had on hand after each of its lines. The first row
/// of an item is its start, and has no line.
fn history() -> Table {
    Table::new(
        "abmc_item_history",
        &["
            item_id integer key -> abmc_item_user
            sequence_number integer key
            transaction_date date null
            transaction_id integer
            target_id integer
            quantity_on_hand numeric(20,5) null
            value_on_hand numeric(20,5) null
            average_cost numeric(20,5) null
        "],
    )
    .rows(
        row! { "item_id" => 502, "average_cost" => "20.00" },
        [
            row! {
                "sequence_number" => 0, "transaction_date" => "2023-01-01",
                "quantity_on_hand" => "0.00", "value_on_hand" => "0.00", "average_cost" => "0.00",
            },
            row! {
                "sequence_number" => 1, "transaction_date" => "2023-05-10",
                "transaction_id" => 200, "target_id" => 202, "quantity_on_hand" => "10.00",
                "value_on_hand" => "200.00",
            },
            row! {
                "sequence_number" => 2, "transaction_date" => "2023-11-15",
                "transaction_id" => 400, "target_id" => 402, "quantity_on_hand" => "7.00",
                "value_on_hand" => "140.00",
            },
            row! {
                "sequence_number" => 3, "transaction_date" => "2024-02-01",
                "transaction_id" => 700, "target_id" => 702, "quantity_on_hand" => "8.00",
                "value_on_hand" => "160.00",
            },
            row! {
                "sequence_number" => 4, "transaction_date" => "2024-03-25",
                "transaction_id" => 1200, "target_id" => 1202, "quantity_on_hand" => "7.00",
                "value_on_hand" => "140.00",
            },
            row! {
                "sequence_number" => 5, "transaction_date" => "2024-04-05",
                "transaction_id" => 1400, "target_id" => 1402, "quantity_on_hand" => "7.00",
                "value_on_hand" => "140.00",
            },
            row! {
                "sequence_number" => 6, "transaction_date" => "2024-04-10",
                "transaction_id" => 1500, "target_id" => 1502, "quantity_on_hand" => "9.00",
                "value_on_hand" => "180.00",
            },
            row! {
                "sequence_number" => 7, "transaction_date" => "2024-04-12",
                "transaction_id" => 1600, "target_id" => 1602, "quantity_on_hand" => "9.00",
                "value_on_hand" => "180.00",
            },
            row! {
                "sequence_number" => 8, "transaction_date" => "2024-05-05",
                "transaction_id" => 2100, "target_id" => 2102, "quantity_on_hand" => "10.00",
                "value_on_hand" => "200.00",
            },
            row! {
                "sequence_number" => 9, "transaction_date" => "2024-05-10",
                "transaction_id" => 2200, "target_id" => 2202, "quantity_on_hand" => "8.00",
                "value_on_hand" => "160.00",
            },
        ],
    )
    .rows(
        row! { "item_id" => 507 },
        [
            row! {
                "sequence_number" => 0, "transaction_date" => "2023-01-01",
                "quantity_on_hand" => "0.00", "value_on_hand" => "0.00", "average_cost" => "0.00",
            },
            row! {
                "sequence_number" => 1, "transaction_date" => "2024-05-10",
                "transaction_id" => 2200, "target_id" => 2201, "quantity_on_hand" => "1.00",
                "value_on_hand" => "40.00", "average_cost" => "40.00",
            },
        ],
    )
}

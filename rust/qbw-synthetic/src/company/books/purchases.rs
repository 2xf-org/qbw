//! What vendors send: bills, credits, receipts and purchase orders.

use crate::company::books::columns::{
    self, CONTACT, DUE_DATE, IS_ARAP, IS_PAID, PRINT_LATER, PURCHASE_LINES, SHIP_VIA, TERMS,
    VENDOR, VIEW_TYPE,
};
use crate::company::books::literal::row;
use crate::company::books::table::Table;

pub(super) fn tables() -> Vec<Table> {
    vec![
        bills(),
        bill_lines(),
        // Stock that is bought posts by the lines of the bill themselves.
        columns::inventory_lines("abmc_bill"),
        purchase_orders(),
        purchase_order_lines(),
    ]
}

fn bills() -> Table {
    let own = "
        is_item_receipt_bool bit
    ";
    columns::headers(
        "abmc_bill",
        &[
            VIEW_TYPE, VENDOR, CONTACT, TERMS, DUE_DATE, IS_PAID, IS_ARAP, own,
        ],
    )
    .row(row! {
        "transaction_id" => 200, "target_id" => 201, "transaction_date" => "2023-05-10",
        "transaction_view_type" => 9, "doc_num" => "7781", "vendor_id" => 201,
        "contact_id" => "CONTACT000000201", "terms_id" => 1, "due_date" => "2023-06-09",
        "is_paid_bool" => true, "trans_num" => 2, "abm_id" => "TXN0000000000200",
    })
    .row(row! {
        "transaction_id" => 1200, "target_id" => 1201, "transaction_date" => "2024-03-25",
        "transaction_view_type" => 12, "doc_num" => "VC-1", "vendor_id" => 201,
        "contact_id" => "CONTACT000000201", "trans_num" => 12, "abm_id" => "TXN0000000001200",
    })
    .row(row! {
        "transaction_id" => 1500, "target_id" => 1501, "transaction_date" => "2024-04-10",
        "transaction_view_type" => 19, "doc_num" => "R-9", "vendor_id" => 201,
        "contact_id" => "CONTACT000000201", "due_date" => "2024-05-10",
        "is_item_receipt_bool" => true, "trans_num" => 15, "abm_id" => "TXN0000000001500",
    })
}

fn bill_lines() -> Table {
    columns::lines("abmc_bill", PURCHASE_LINES)
        .rows(
            row! {
                "transaction_id" => 200, "transaction_date" => "2023-05-10",
                "transaction_view_type" => 9, "vendor_id" => 201, "is_paid_bool" => true,
                "due_date" => "2023-06-09",
            },
            [
                row! {
                    "target_id" => 201, "is_source_bool" => true, "next_target_id" => 202,
                    "account_id" => 7, "sibling_account_id" => 6, "amount_amt" => "-200.00",
                    "foreign_amount_amt" => "-200.00", "doc_num" => "7781",
                    "lineitem_name_type" => 101,
                },
                row! {
                    "target_id" => 202, "next_target_id" => 0, "account_id" => 6,
                    "sibling_account_id" => 7, "amount_amt" => "200.00",
                    "foreign_amount_amt" => "200.00", "memo" => "Valve, brass", "item_id" => 502,
                    "quantity_qnty" => "10.00", "unit_price_amt" => "20.00",
                    "manufacturer_part_num" => "CV-20",
                },
            ],
        )
        .rows(
            row! {
                "transaction_id" => 1200, "transaction_date" => "2024-03-25",
                "transaction_view_type" => 12, "vendor_id" => 201,
            },
            [
                row! {
                    "target_id" => 1201, "is_source_bool" => true, "next_target_id" => 1202,
                    "account_id" => 7, "sibling_account_id" => 6, "amount_amt" => "20.00",
                    "foreign_amount_amt" => "20.00", "doc_num" => "VC-1",
                    "lineitem_name_type" => 101,
                },
                row! {
                    "target_id" => 1202, "next_target_id" => 0, "account_id" => 6,
                    "sibling_account_id" => 7, "amount_amt" => "-20.00",
                    "foreign_amount_amt" => "-20.00", "memo" => "Valve, brass", "item_id" => 502,
                    "quantity_qnty" => "-1.00", "unit_price_amt" => "20.00",
                },
            ],
        )
        .rows(
            row! {
                "transaction_id" => 1500, "transaction_date" => "2024-04-10",
                "transaction_view_type" => 19, "vendor_id" => 201, "due_date" => "2024-05-10",
            },
            [
                row! {
                    "target_id" => 1501, "is_source_bool" => true, "next_target_id" => 1502,
                    "account_id" => 7, "sibling_account_id" => 6, "amount_amt" => "-40.00",
                    "foreign_amount_amt" => "-40.00", "doc_num" => "R-9",
                    "lineitem_name_type" => 101,
                },
                row! {
                    "target_id" => 1502, "next_target_id" => 0, "account_id" => 6,
                    "sibling_account_id" => 7, "amount_amt" => "40.00",
                    "foreign_amount_amt" => "40.00", "memo" => "Valve, brass", "item_id" => 502,
                    "quantity_qnty" => "2.00", "unit_price_amt" => "20.00",
                },
            ],
        )
}

fn purchase_orders() -> Table {
    let own = "
        ship_to_id varchar(17) null
        ship_date date null
        po_num varchar(26) null
        fob varchar(14) null
        other varchar(30) null
        custom_style_id integer null -> abmc_custom_style
        is_to_be_sent_bool bit
        is_received_bool bit
        ship_to_name_id integer null
        vendor_message varchar(101) null
        expected_date date null
        site_id integer null -> abmc_site
    ";
    columns::headers(
        "abmc_purchase_order",
        &[
            VENDOR,
            CONTACT,
            TERMS,
            DUE_DATE,
            SHIP_VIA,
            IS_PAID,
            PRINT_LATER,
            own,
        ],
    )
    .row(row! {
        "transaction_id" => 1400, "target_id" => 1401, "transaction_date" => "2024-04-05",
        "doc_num" => "P-12", "vendor_id" => 201, "contact_id" => "CONTACT000000201",
        "terms_id" => 1, "due_date" => "2024-05-05", "ship_date" => "2024-04-19",
        "vendor_message" => "Deliver in the morning.", "custom_style_id" => 1,
        "is_no_post_bool" => true, "trans_num" => 14, "abm_id" => "TXN0000000001400",
    })
}

fn purchase_order_lines() -> Table {
    columns::lines("abmc_purchase_order", &columns::sales_lines()).rows(
        row! {
            "transaction_id" => 1400, "transaction_date" => "2024-04-05",
            "transaction_view_type" => 17, "vendor_id" => 201, "is_no_post_bool" => true,
            "due_date" => "2024-05-05",
        },
        [
            row! {
                "target_id" => 1401, "is_source_bool" => true, "next_target_id" => 1402,
                "account_id" => 19, "sibling_account_id" => 6, "amount_amt" => "-100.00",
                "foreign_amount_amt" => "-100.00", "doc_num" => "P-12",
                "lineitem_name_type" => 101,
            },
            row! {
                "target_id" => 1402, "next_target_id" => 0, "account_id" => 6,
                "sibling_account_id" => 19, "amount_amt" => "100.00",
                "foreign_amount_amt" => "100.00", "memo" => "Valve, brass", "item_id" => 502,
                "quantity_qnty" => "5.00", "unit_price_amt" => "20.00", "open_qnty" => "3.00",
                "manufacturer_part_num" => "CV-20",
            },
        ],
    )
}

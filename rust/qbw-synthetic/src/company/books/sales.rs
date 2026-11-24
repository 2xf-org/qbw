//! What customers get: invoices, credit memos, estimates and sales orders.

use crate::company::books::columns::{
    self, CONTACT, CREDIT_LINES, CUSTOMER, DUE_DATE, IS_ARAP, IS_PAID, PRINT_LATER, SALES_FORM,
    SHIP_VIA, TERMS,
};
use crate::company::books::literal::{Literal, row};
use crate::company::books::table::Table;

pub(super) fn tables() -> Vec<Table> {
    vec![
        invoices(),
        invoice_lines(),
        invoice_stock(),
        credit_memos(),
        credit_memo_lines(),
        credit_memo_stock(),
        estimates(),
        estimate_lines(),
        sales_orders(),
        sales_order_lines(),
    ]
}

fn invoices() -> Table {
    let own = "
        is_finance_charge_bool bit
        sales_order_num varchar(26) null
    ";
    columns::headers(
        "abmc_invoice",
        &[
            CUSTOMER,
            CONTACT,
            TERMS,
            DUE_DATE,
            SALES_FORM,
            SHIP_VIA,
            IS_PAID,
            PRINT_LATER,
            IS_ARAP,
            own,
        ],
    )
    .row(row! {
        "transaction_id" => 400, "target_id" => 401, "transaction_date" => "2023-11-15",
        "doc_num" => "1041", "customer_id" => 101, "memo" => "Spring order",
        "contact_id" => "CONTACT000000100", "terms_id" => 1, "due_date" => "2023-12-15",
        "ship_to_id" => "SHIPTO0000000400", "ship_date" => "2023-11-15", "ship_via_id" => 1,
        "sales_rep_id" => 1, "po_num" => "PO-77", "customer_message_id" => 1,
        "sales_tax_code_id" => 1, "custom_style_id" => 1, "is_to_be_sent_bool" => true,
        "sales_order_num" => "S-4", "trans_num" => 4, "timestamp_tms" => 1700056800,
        "modtimestamp_tms" => 1700060400, "abm_id" => "TXN0000000000400",
    })
    .row(row! {
        "transaction_id" => 800, "target_id" => 801, "transaction_date" => "2024-02-10",
        "doc_num" => "1042", "customer_id" => 102, "contact_id" => "CONTACT000000102",
        "terms_id" => 1, "due_date" => "2024-03-11", "ship_date" => "2024-02-10",
        "sales_tax_code_id" => 2, "custom_style_id" => 1, "currency_id" => 2,
        "exchange_rate" => "1.10", "is_paid_bool" => true, "trans_num" => 8,
        "abm_id" => "TXN0000000000800",
    })
    // Pending: it has amounts, and posts none of them.
    .row(row! {
        "transaction_id" => 1800, "target_id" => 1801, "transaction_date" => "2024-04-25",
        "doc_num" => "1043", "customer_id" => 103, "contact_id" => "CONTACT000000103",
        "due_date" => "2024-05-25", "ship_date" => "2024-04-25", "custom_style_id" => 1,
        "is_to_be_printed_bool" => true, "is_no_post_bool" => true, "trans_num" => 18,
        "abm_id" => "TXN0000000001800",
    })
}

fn invoice_lines() -> Table {
    columns::lines("abmc_invoice", &columns::sales_lines())
        .rows(
            row! {
                "transaction_id" => 400, "transaction_date" => "2023-11-15",
                "transaction_view_type" => 4, "customer_id" => 101, "is_split_bool" => true,
                "due_date" => "2023-12-15",
            },
            [
                row! {
                    "target_id" => 401, "is_source_bool" => true, "next_target_id" => 402,
                    "account_id" => 3, "sibling_account_id" => 13, "amount_amt" => "202.00",
                    "foreign_amount_amt" => "202.00", "memo" => "Spring order",
                    "doc_num" => "1041", "lineitem_name_type" => 100, "open_balance_amt" => "45.00",
                },
                row! {
                    "target_id" => 402, "next_target_id" => 403, "account_id" => 13,
                    "sibling_account_id" => 3, "amount_amt" => "-150.00",
                    "foreign_amount_amt" => "-150.00", "memo" => "Brass valve", "item_id" => 502,
                    "quantity_qnty" => "-3.00", "unit_price_amt" => "50.00",
                    "sales_tax_code_id" => 1, "manufacturer_part_num" => "CV-20",
                    "service_date" => "2023-11-14", "other1" => "Shelf 4",
                    "abm_id" => "LINE000000000402",
                },
                row! {
                    "target_id" => 403, "next_target_id" => 404, "account_id" => 12,
                    "sibling_account_id" => 3, "amount_amt" => "-40.00",
                    "foreign_amount_amt" => "-40.00", "memo" => "Installation, per valve",
                    "item_id" => 501, "quantity_qnty" => "-1.00", "unit_price_amt" => "40.00",
                    "sales_tax_code_id" => 2,
                },
                row! {
                    "target_id" => 404, "next_target_id" => 405, "sibling_account_id" => 3,
                    "memo" => "Delivered to dock 1",
                },
                row! {
                    "target_id" => 405, "next_target_id" => 0, "account_id" => 9,
                    "sibling_account_id" => 3, "amount_amt" => "-12.00",
                    "foreign_amount_amt" => "-12.00", "memo" => "State tax", "item_id" => 503,
                    "unit_price_pct" => "8.00", "is_trans_tax_bool" => true, "vendor_id" => 202,
                    "lineitem_name_type" => 101,
                },
            ],
        )
        .rows(
            row! {
                "transaction_id" => 800, "transaction_date" => "2024-02-10",
                "transaction_view_type" => 4, "customer_id" => 102, "currency_id" => 2,
                "exchange_rate" => "1.10", "is_paid_bool" => true, "due_date" => "2024-03-11",
            },
            [
                row! {
                    "target_id" => 801, "is_source_bool" => true, "next_target_id" => 802,
                    "account_id" => 4, "sibling_account_id" => 12, "amount_amt" => "110.00",
                    "foreign_amount_amt" => "100.00", "doc_num" => "1042",
                    "lineitem_name_type" => 100,
                },
                row! {
                    "target_id" => 802, "next_target_id" => 0, "account_id" => 12,
                    "sibling_account_id" => 4, "amount_amt" => "-110.00",
                    "foreign_amount_amt" => "-100.00", "memo" => "Installation", "item_id" => 501,
                    "quantity_qnty" => "-1.00", "unit_price_amt" => "100.00",
                    "sales_tax_code_id" => 2,
                },
            ],
        )
        .rows(
            row! {
                "transaction_id" => 1800, "transaction_date" => "2024-04-25",
                "transaction_view_type" => 4, "customer_id" => 103, "is_no_post_bool" => true,
                "due_date" => "2024-05-25",
            },
            [
                row! {
                    "target_id" => 1801, "is_source_bool" => true, "next_target_id" => 1802,
                    "account_id" => 3, "sibling_account_id" => 13, "amount_amt" => "50.00",
                    "foreign_amount_amt" => "50.00", "doc_num" => "1043",
                    "lineitem_name_type" => 100,
                },
                row! {
                    "target_id" => 1802, "next_target_id" => 0, "account_id" => 13,
                    "sibling_account_id" => 3, "amount_amt" => "-50.00",
                    "foreign_amount_amt" => "-50.00", "memo" => "Brass valve", "item_id" => 502,
                    "quantity_qnty" => "-1.00", "unit_price_amt" => "50.00",
                    "sales_tax_code_id" => 1,
                },
            ],
        )
}

fn invoice_stock() -> Table {
    columns::inventory_lines("abmc_invoice").rows(
        row! { "target_id" => 402 },
        [
            row! {
                "inventory_record_id" => 1, "target_type" => 1, "account_id" => 6,
                "amount_amt" => "-60.00", "foreign_amount_amt" => "-60.00",
                "quantity_qnty" => "-3.00",
            },
            row! {
                "inventory_record_id" => 2, "target_type" => 2, "account_id" => 14,
                "amount_amt" => "60.00", "foreign_amount_amt" => "60.00",
                "quantity_qnty" => "3.00",
            },
        ],
    )
}

fn credit_memos() -> Table {
    columns::headers(
        "abmc_credit_memo",
        &[
            CUSTOMER,
            CONTACT,
            TERMS,
            DUE_DATE,
            SALES_FORM,
            SHIP_VIA,
            IS_PAID,
            PRINT_LATER,
            IS_ARAP,
        ],
    )
    .row(row! {
        "transaction_id" => 700, "target_id" => 701, "transaction_date" => "2024-02-01",
        "doc_num" => "CM-3", "customer_id" => 101, "memo" => "One valve came back",
        "contact_id" => "CONTACT000000101", "due_date" => "2024-02-01",
        "ship_date" => "2024-02-01", "sales_tax_code_id" => 1, "custom_style_id" => 1,
        "is_paid_bool" => true, "trans_num" => 7, "abm_id" => "TXN0000000000700",
    })
}

fn credit_memo_lines() -> Table {
    columns::lines("abmc_credit_memo", CREDIT_LINES).rows(
        row! {
            "transaction_id" => 700, "transaction_date" => "2024-02-01",
            "transaction_view_type" => 6, "customer_id" => 101, "is_split_bool" => true,
            "is_paid_bool" => true, "due_date" => "2024-02-01",
        },
        [
            row! {
                "target_id" => 701, "is_source_bool" => true, "next_target_id" => 702,
                "account_id" => 3, "sibling_account_id" => 13, "amount_amt" => "-54.00",
                "foreign_amount_amt" => "-54.00", "memo" => "One valve came back",
                "doc_num" => "CM-3", "lineitem_name_type" => 100,
            },
            row! {
                "target_id" => 702, "next_target_id" => 703, "account_id" => 13,
                "sibling_account_id" => 3, "amount_amt" => "50.00", "foreign_amount_amt" => "50.00",
                "memo" => "Brass valve", "item_id" => 502, "quantity_qnty" => "1.00",
                "unit_price_amt" => "50.00", "sales_tax_code_id" => 1,
            },
            row! {
                "target_id" => 703, "next_target_id" => 0, "account_id" => 9,
                "sibling_account_id" => 3, "amount_amt" => "4.00", "foreign_amount_amt" => "4.00",
                "memo" => "State tax", "item_id" => 503, "unit_price_pct" => "8.00",
                "is_trans_tax_bool" => true, "vendor_id" => 202, "lineitem_name_type" => 101,
            },
        ],
    )
}

fn credit_memo_stock() -> Table {
    columns::inventory_lines("abmc_credit_memo").rows(
        row! { "target_id" => 702 },
        [
            row! {
                "inventory_record_id" => 3, "target_type" => 1, "account_id" => 6,
                "amount_amt" => "20.00", "foreign_amount_amt" => "20.00",
                "quantity_qnty" => "1.00",
            },
            row! {
                "inventory_record_id" => 4, "target_type" => 2, "account_id" => 14,
                "amount_amt" => "-20.00", "foreign_amount_amt" => "-20.00",
                "quantity_qnty" => "-1.00",
            },
        ],
    )
}

fn estimates() -> Table {
    let own = "
        is_finance_charge_bool bit
    ";
    columns::headers(
        "abmc_estimate",
        &[
            CUSTOMER,
            CONTACT,
            TERMS,
            DUE_DATE,
            SALES_FORM,
            PRINT_LATER,
            IS_ARAP,
            own,
        ],
    )
    .row(row! {
        "transaction_id" => 2000, "target_id" => 2001, "transaction_date" => "2024-05-01",
        "doc_num" => "E-1", "customer_id" => 101, "contact_id" => "CONTACT000000101",
        "due_date" => "2024-05-31", "ship_date" => "2024-05-01", "sales_tax_code_id" => 1,
        "custom_style_id" => 1, "is_no_post_bool" => true, "trans_num" => 20,
        "abm_id" => "TXN0000000002000",
    })
}

/// A group of two items, with its start and its end, a subtotal of it, and a
/// discount on that.
fn estimate_lines() -> Table {
    columns::lines("abmc_estimate", &columns::sales_lines()).rows(
        row! {
            "transaction_id" => 2000, "transaction_date" => "2024-05-01",
            "transaction_view_type" => 24, "customer_id" => 101, "is_no_post_bool" => true,
            "is_split_bool" => true, "due_date" => "2024-05-31",
        },
        [
            row! {
                "target_id" => 2001, "is_source_bool" => true, "next_target_id" => 2002,
                "account_id" => 21, "amount_amt" => "129.00", "foreign_amount_amt" => "129.00",
                "doc_num" => "E-1", "lineitem_name_type" => 100,
            },
            row! {
                "target_id" => 2002, "next_target_id" => 2003, "sibling_account_id" => 21,
                "memo" => "Valve kit", "item_id" => 505, "quantity_qnty" => "-1.00",
            },
            row! {
                "target_id" => 2003, "next_target_id" => 2004, "account_id" => 13,
                "sibling_account_id" => 21, "amount_amt" => "-100.00",
                "foreign_amount_amt" => "-100.00", "memo" => "Brass valve", "item_id" => 502,
                "quantity_qnty" => "-2.00", "unit_price_amt" => "50.00", "sales_tax_code_id" => 1,
            },
            row! {
                "target_id" => 2004, "next_target_id" => 2005, "account_id" => 12,
                "sibling_account_id" => 21, "amount_amt" => "-40.00",
                "foreign_amount_amt" => "-40.00", "memo" => "Installation, per valve",
                "item_id" => 501, "quantity_qnty" => "-1.00", "unit_price_amt" => "40.00",
                "sales_tax_code_id" => 2,
            },
            row! {
                "target_id" => 2005, "next_target_id" => 2006, "sibling_account_id" => 21,
                "amount_amt" => "-140.00", "foreign_amount_amt" => "-140.00", "item_id" => 505,
                "is_groupend_row_bool" => true,
            },
            row! {
                "target_id" => 2006, "next_target_id" => 2007, "sibling_account_id" => 21,
                "amount_amt" => "-140.00", "foreign_amount_amt" => "-140.00",
                "memo" => "Kit in all", "item_id" => 504,
            },
            row! {
                "target_id" => 2007, "next_target_id" => 0, "account_id" => 18,
                "sibling_account_id" => 21, "amount_amt" => "11.00",
                "foreign_amount_amt" => "11.00", "memo" => "Kit discount", "item_id" => 506,
                "unit_price_amt" => "-11.00", "sales_tax_code_id" => 2,
            },
        ],
    )
}

fn sales_orders() -> Table {
    columns::headers(
        "abmc_sales_order",
        &[
            CUSTOMER,
            CONTACT,
            TERMS,
            SALES_FORM,
            SHIP_VIA,
            IS_PAID,
            PRINT_LATER,
            IS_ARAP,
        ],
    )
    .row(row! {
        "transaction_id" => 1600, "target_id" => 1601, "transaction_date" => "2024-04-12",
        "doc_num" => "S-5", "customer_id" => 103, "contact_id" => "CONTACT000000103",
        "terms_id" => 1, "ship_date" => "2024-06-01", "po_num" => "ANNEX-1",
        "sales_tax_code_id" => 1, "custom_style_id" => 1, "is_no_post_bool" => true,
        "trans_num" => 16, "abm_id" => "TXN0000000001600",
    })
}

fn sales_order_lines() -> Table {
    columns::lines("abmc_sales_order", &columns::sales_lines()).rows(
        row! {
            "transaction_id" => 1600, "transaction_date" => "2024-04-12",
            "transaction_view_type" => 27, "customer_id" => 103, "is_no_post_bool" => true,
            "due_date" => "2024-06-01",
        },
        [
            row! {
                "target_id" => 1601, "is_source_bool" => true, "next_target_id" => 1602,
                "account_id" => 20, "sibling_account_id" => 13, "amount_amt" => "200.00",
                "foreign_amount_amt" => "200.00", "doc_num" => "S-5", "lineitem_name_type" => 100,
            },
            // Nothing in `open_qnty` is the whole quantity.
            row! {
                "target_id" => 1602, "next_target_id" => 0, "account_id" => 13,
                "sibling_account_id" => 20, "amount_amt" => "-200.00",
                "foreign_amount_amt" => "-200.00", "memo" => "Brass valve", "item_id" => 502,
                "quantity_qnty" => "-4.00", "unit_price_amt" => "50.00", "sales_tax_code_id" => 1,
                "open_qnty" => Literal::Null,
            },
        ],
    )
}

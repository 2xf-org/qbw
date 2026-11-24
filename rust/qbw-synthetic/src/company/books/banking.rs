//! Money that moves: payments, deposits, checks, card charges and transfers.

use crate::company::books::columns::{
    self, CLASS, CLEARING, CONTACT, CUSTOMER, DUE_DATE, EMPLOYEE_AND_OTHER, HEADER_NAME_TYPE,
    IS_ARAP, IS_SPLIT, LINE_NAME_TYPE, OPEN_BALANCE, PAYMENT_LINES, PAYMENT_METHOD, PRINT_LATER,
    PURCHASE_LINES, VENDOR, VIEW_TYPE,
};
use crate::company::books::literal::{Literal, row};
use crate::company::books::table::Table;

pub(super) fn tables() -> Vec<Table> {
    vec![
        payments(),
        payment_lines(),
        bill_payments(),
        bill_payment_lines(),
        deposits(),
        deposit_lines(),
        checks(),
        check_lines(),
        columns::inventory_lines("abmc_check"),
        card_charges(),
        card_charge_lines(),
        columns::inventory_lines("abmc_credit_cardcharge"),
        transfers(),
        transfer_lines(),
    ]
}

fn payments() -> Table {
    let own = "
        is_deposited_bool bit
        is_bounced_bool bit
    ";
    columns::headers(
        "abmc_received_payment",
        &[CUSTOMER, PAYMENT_METHOD, IS_ARAP, own],
    )
    .row(row! {
        "transaction_id" => 500, "target_id" => 501, "transaction_date" => "2024-01-20",
        "doc_num" => "88", "customer_id" => 101, "payment_method_id" => 1,
        "is_deposited_bool" => true, "trans_num" => 5, "abm_id" => "TXN0000000000500",
    })
    .row(row! {
        "transaction_id" => 900, "target_id" => 901, "transaction_date" => "2024-03-10",
        "doc_num" => "W-9", "customer_id" => 102, "payment_method_id" => 2,
        "currency_id" => 2, "exchange_rate" => "1.20", "is_deposited_bool" => true,
        "trans_num" => 9, "abm_id" => "TXN0000000000900",
    })
    // It moves no money: the discount is all of it.
    .row(row! {
        "transaction_id" => 2400, "target_id" => 2401, "transaction_date" => "2024-06-01",
        "customer_id" => 101, "is_arap_bool" => true, "is_deposited_bool" => true,
        "trans_num" => 24, "abm_id" => "TXN0000000002400",
    })
}

fn payment_lines() -> Table {
    columns::lines("abmc_received_payment", PAYMENT_LINES)
        .rows(
            row! {
                "transaction_id" => 500, "transaction_date" => "2024-01-20",
                "transaction_view_type" => 7, "customer_id" => 101, "is_paid_bool" => true,
            },
            [
                row! {
                    "target_id" => 501, "is_source_bool" => true, "next_target_id" => 502,
                    "account_id" => 5, "sibling_account_id" => 3, "amount_amt" => "100.00",
                    "foreign_amount_amt" => "100.00", "doc_num" => "88",
                    "lineitem_name_type" => 100, "payment_method_id" => 1,
                },
                row! {
                    "target_id" => 502, "next_target_id" => 503, "account_id" => 3,
                    "sibling_account_id" => 5, "amount_amt" => "-100.00",
                    "foreign_amount_amt" => "-100.00",
                },
                row! {
                    "target_id" => 503, "next_target_id" => 504, "account_id" => 3,
                    "sibling_account_id" => 5, "amount_amt" => "-2.00",
                    "foreign_amount_amt" => "-2.00", "is_discount_bool" => true,
                },
                row! {
                    "target_id" => 504, "next_target_id" => 0, "account_id" => 18,
                    "sibling_account_id" => 5, "amount_amt" => "2.00",
                    "foreign_amount_amt" => "2.00", "is_applied_discount_bool" => true,
                },
            ],
        )
        .rows(
            row! {
                "transaction_id" => 900, "transaction_date" => "2024-03-10",
                "transaction_view_type" => 7, "customer_id" => 102, "currency_id" => 2,
                "exchange_rate" => "1.20", "is_paid_bool" => true,
            },
            [
                row! {
                    "target_id" => 901, "is_source_bool" => true, "next_target_id" => 902,
                    "account_id" => 2, "sibling_account_id" => 4, "amount_amt" => "120.00",
                    "foreign_amount_amt" => "100.00", "doc_num" => "W-9",
                    "lineitem_name_type" => 100, "payment_method_id" => 2,
                },
                row! {
                    "target_id" => 902, "next_target_id" => 903, "account_id" => 4,
                    "sibling_account_id" => 2, "amount_amt" => "-110.00",
                    "foreign_amount_amt" => "-100.00",
                },
                // The exchange difference has no amount in the other currency.
                row! {
                    "target_id" => 903, "next_target_id" => 0, "account_id" => 17,
                    "sibling_account_id" => 2, "amount_amt" => "-10.00",
                    "foreign_amount_amt" => Literal::Null, "is_gain_loss_target_bool" => true,
                },
            ],
        )
        .rows(
            row! {
                "transaction_id" => 2400, "transaction_date" => "2024-06-01",
                "transaction_view_type" => 7, "customer_id" => 101, "is_paid_bool" => true,
            },
            [
                row! {
                    "target_id" => 2401, "is_source_bool" => true, "next_target_id" => 2402,
                    "account_id" => 3, "sibling_account_id" => 3, "amount_amt" => "-1.00",
                    "foreign_amount_amt" => "-1.00", "lineitem_name_type" => 100,
                    "is_discount_bool" => true,
                },
                row! {
                    "target_id" => 2402, "next_target_id" => 2403, "account_id" => 3,
                    "sibling_account_id" => 3, "amount_amt" => "0.00",
                    "foreign_amount_amt" => "0.00",
                },
                row! {
                    "target_id" => 2403, "next_target_id" => 0, "account_id" => 18,
                    "sibling_account_id" => 3, "amount_amt" => "1.00",
                    "foreign_amount_amt" => "1.00", "is_applied_discount_bool" => true,
                },
            ],
        )
}

fn bill_payments() -> Table {
    columns::headers(
        "abmc_bill_check",
        &[VENDOR, CONTACT, DUE_DATE, PRINT_LATER, IS_ARAP],
    )
    .row(row! {
        "transaction_id" => 300, "target_id" => 301, "transaction_date" => "2023-06-05",
        "doc_num" => "501", "vendor_id" => 201, "contact_id" => "CONTACT000000201",
        "trans_num" => 3, "abm_id" => "TXN0000000000300",
    })
}

fn bill_payment_lines() -> Table {
    columns::lines("abmc_bill_check", PAYMENT_LINES).rows(
        row! {
            "transaction_id" => 300, "transaction_date" => "2023-06-05",
            "transaction_view_type" => 14, "vendor_id" => 201, "is_paid_bool" => true,
        },
        [
            row! {
                "target_id" => 301, "is_source_bool" => true, "next_target_id" => 302,
                "account_id" => 1, "sibling_account_id" => 7, "amount_amt" => "-200.00",
                "foreign_amount_amt" => "-200.00", "doc_num" => "501",
                "lineitem_name_type" => 101, "cleared_type" => 2,
            },
            row! {
                "target_id" => 302, "next_target_id" => 0, "account_id" => 7,
                "sibling_account_id" => 1, "amount_amt" => "200.00",
                "foreign_amount_amt" => "200.00",
            },
        ],
    )
}

fn deposits() -> Table {
    columns::headers(
        "abmc_deposit",
        &[
            CUSTOMER,
            VENDOR,
            EMPLOYEE_AND_OTHER,
            HEADER_NAME_TYPE,
            IS_ARAP,
        ],
    )
    .row(row! {
        "transaction_id" => 600, "target_id" => 601, "transaction_date" => "2024-01-25",
        "memo" => "Deposit", "trans_num" => 6, "abm_id" => "TXN0000000000600",
    })
}

fn deposit_lines() -> Table {
    let own = "
        is_less_cashback_bool bit
    ";
    columns::lines(
        "abmc_deposit",
        &[
            IS_SPLIT,
            CUSTOMER,
            VENDOR,
            EMPLOYEE_AND_OTHER,
            LINE_NAME_TYPE,
            CLASS,
            OPEN_BALANCE,
            CLEARING,
            PAYMENT_METHOD,
            own,
        ],
    )
    .rows(
        row! {
            "transaction_id" => 600, "transaction_date" => "2024-01-25",
            "transaction_view_type" => 2,
        },
        [
            row! {
                "target_id" => 601, "is_source_bool" => true, "next_target_id" => 602,
                "account_id" => 1, "sibling_account_id" => 5, "amount_amt" => "100.00",
                "foreign_amount_amt" => "100.00", "memo" => "Deposit", "cleared_type" => 2,
            },
            row! {
                "target_id" => 602, "next_target_id" => 0, "account_id" => 5,
                "sibling_account_id" => 1, "amount_amt" => "-100.00",
                "foreign_amount_amt" => "-100.00", "doc_num" => "88", "customer_id" => 101,
                "lineitem_name_type" => 100, "payment_method_id" => 1, "is_paid_bool" => true,
            },
        ],
    )
}

fn checks() -> Table {
    columns::headers(
        "abmc_check",
        &[
            VIEW_TYPE,
            CUSTOMER,
            VENDOR,
            EMPLOYEE_AND_OTHER,
            HEADER_NAME_TYPE,
            CONTACT,
            PRINT_LATER,
            IS_ARAP,
        ],
    )
    // Voided: its amounts became zero.
    .row(row! {
        "transaction_id" => 1700, "target_id" => 1701, "transaction_date" => "2024-04-20",
        "transaction_view_type" => 3, "doc_num" => "502", "other_id" => 401,
        "transaction_name_type" => 103, "contact_id" => "CONTACT000000401",
        "memo" => "VOID: Fuel", "last_auditable_action" => 3, "trans_num" => 17,
        "abm_id" => "TXN0000000001700",
    })
    // Memorized: a form to start from, which nothing counts.
    .row(row! {
        "transaction_id" => 1900, "target_id" => 1901, "transaction_date" => "2024-04-28",
        "transaction_view_type" => 3, "other_id" => 401, "transaction_name_type" => 103,
        "contact_id" => "CONTACT000000401", "memo" => "Fuel",
        "is_memorized_transaction_bool" => true, "trans_num" => 19,
        "abm_id" => "TXN0000000001900",
    })
    // The tax of the two sales forms, paid to its agency. The check waits to be
    // printed and has no number yet, which its own line has nines for.
    .row(row! {
        "transaction_id" => 2300, "target_id" => 2301, "transaction_date" => "2024-05-15",
        "transaction_view_type" => 16, "vendor_id" => 202,
        "transaction_name_type" => 101, "contact_id" => "CONTACT000000202",
        "is_to_be_printed_bool" => true, "trans_num" => 23, "abm_id" => "TXN0000000002300",
    })
}

fn check_lines() -> Table {
    columns::lines("abmc_check", PURCHASE_LINES)
        .rows(
            row! {
                "transaction_id" => 1700, "transaction_date" => "2024-04-20",
                "transaction_view_type" => 3, "other_id" => 401,
            },
            [
                row! {
                    "target_id" => 1701, "is_source_bool" => true, "next_target_id" => 1702,
                    "account_id" => 1, "sibling_account_id" => 15, "amount_amt" => "0.00",
                    "foreign_amount_amt" => "0.00", "doc_num" => "502", "memo" => "VOID: Fuel",
                    "lineitem_name_type" => 103, "cleared_type" => 2,
                },
                row! {
                    "target_id" => 1702, "next_target_id" => 0, "account_id" => 15,
                    "sibling_account_id" => 1, "amount_amt" => "0.00",
                    "foreign_amount_amt" => "0.00",
                },
            ],
        )
        .rows(
            row! {
                "transaction_id" => 1900, "transaction_date" => "2024-04-28",
                "transaction_view_type" => 3, "other_id" => 401,
                "is_memorized_transaction_bool" => true,
            },
            [
                row! {
                    "target_id" => 1901, "is_source_bool" => true, "next_target_id" => 1902,
                    "account_id" => 1, "sibling_account_id" => 15, "amount_amt" => "-15.00",
                    "foreign_amount_amt" => "-15.00", "memo" => "Fuel",
                    "lineitem_name_type" => 103,
                },
                row! {
                    "target_id" => 1902, "next_target_id" => 0, "account_id" => 15,
                    "sibling_account_id" => 1, "amount_amt" => "15.00",
                    "foreign_amount_amt" => "15.00", "memo" => "Fuel",
                },
            ],
        )
        .rows(
            row! {
                "transaction_id" => 2300, "transaction_date" => "2024-05-15",
                "transaction_view_type" => 16, "vendor_id" => 202,
            },
            [
                row! {
                    "target_id" => 2301, "is_source_bool" => true, "next_target_id" => 2302,
                    "account_id" => 1, "sibling_account_id" => 9, "amount_amt" => "-8.00",
                    "foreign_amount_amt" => "-8.00", "doc_num" => "99999999999999999999",
                    "lineitem_name_type" => 101, "cleared_type" => 1,
                },
                row! {
                    "target_id" => 2302, "next_target_id" => 0, "account_id" => 9,
                    "sibling_account_id" => 1, "amount_amt" => "8.00",
                    "foreign_amount_amt" => "8.00", "memo" => "State tax", "item_id" => 503,
                },
            ],
        )
}

fn card_charges() -> Table {
    columns::headers(
        "abmc_credit_cardcharge",
        &[
            VIEW_TYPE,
            CUSTOMER,
            VENDOR,
            EMPLOYEE_AND_OTHER,
            HEADER_NAME_TYPE,
            CONTACT,
            IS_ARAP,
        ],
    )
    .row(row! {
        "transaction_id" => 1000, "target_id" => 1001, "transaction_date" => "2024-03-15",
        "transaction_view_type" => 1, "vendor_id" => 201, "transaction_name_type" => 101,
        "memo" => "Paper", "trans_num" => 10, "abm_id" => "TXN0000000001000",
    })
    .row(row! {
        "transaction_id" => 1100, "target_id" => 1101, "transaction_date" => "2024-03-20",
        "transaction_view_type" => 11, "vendor_id" => 201, "transaction_name_type" => 101,
        "memo" => "Paper, returned", "trans_num" => 11, "abm_id" => "TXN0000000001100",
    })
}

fn card_charge_lines() -> Table {
    columns::lines("abmc_credit_cardcharge", PURCHASE_LINES)
        .rows(
            row! {
                "transaction_id" => 1000, "transaction_date" => "2024-03-15",
                "transaction_view_type" => 1, "vendor_id" => 201,
            },
            [
                row! {
                    "target_id" => 1001, "is_source_bool" => true, "next_target_id" => 1002,
                    "account_id" => 8, "sibling_account_id" => 16, "amount_amt" => "-30.00",
                    "foreign_amount_amt" => "-30.00", "memo" => "Paper",
                    "lineitem_name_type" => 101, "cleared_type" => 3,
                },
                row! {
                    "target_id" => 1002, "next_target_id" => 0, "account_id" => 16,
                    "sibling_account_id" => 8, "amount_amt" => "30.00",
                    "foreign_amount_amt" => "30.00", "memo" => "Paper",
                },
            ],
        )
        .rows(
            row! {
                "transaction_id" => 1100, "transaction_date" => "2024-03-20",
                "transaction_view_type" => 11, "vendor_id" => 201,
            },
            [
                row! {
                    "target_id" => 1101, "is_source_bool" => true, "next_target_id" => 1102,
                    "account_id" => 8, "sibling_account_id" => 16, "amount_amt" => "5.00",
                    "foreign_amount_amt" => "5.00", "memo" => "Paper, returned",
                    "lineitem_name_type" => 101,
                },
                row! {
                    "target_id" => 1102, "next_target_id" => 0, "account_id" => 16,
                    "sibling_account_id" => 8, "amount_amt" => "-5.00",
                    "foreign_amount_amt" => "-5.00", "memo" => "Paper, returned",
                },
            ],
        )
}

fn transfers() -> Table {
    columns::headers(
        "abmc_transfer",
        &[
            CUSTOMER,
            VENDOR,
            EMPLOYEE_AND_OTHER,
            HEADER_NAME_TYPE,
            IS_ARAP,
        ],
    )
    .row(row! {
        "transaction_id" => 1300, "target_id" => 1301, "transaction_date" => "2024-04-01",
        "memo" => "Funds transfer", "exchange_rate" => "1.10", "trans_num" => 13,
        "abm_id" => "TXN0000000001300",
    })
}

/// The transfer is in the home currency, and what its lines have as the other
/// amount is in the currency of the account that receives it.
fn transfer_lines() -> Table {
    let own = "
        class_id integer null
    ";
    columns::lines(
        "abmc_transfer",
        &[
            CUSTOMER,
            VENDOR,
            EMPLOYEE_AND_OTHER,
            LINE_NAME_TYPE,
            CLEARING,
            own,
        ],
    )
    .rows(
        row! {
            "transaction_id" => 1300, "transaction_date" => "2024-04-01",
            "transaction_view_type" => 26, "exchange_rate" => "1.10",
        },
        [
            row! {
                "target_id" => 1301, "is_source_bool" => true, "next_target_id" => 1302,
                "account_id" => 1, "sibling_account_id" => 2, "amount_amt" => "-55.00",
                "foreign_amount_amt" => "-50.00", "memo" => "Funds transfer", "cleared_type" => 2,
            },
            row! {
                "target_id" => 1302, "next_target_id" => 0, "account_id" => 2,
                "sibling_account_id" => 1, "amount_amt" => "55.00",
                "foreign_amount_amt" => "50.00", "memo" => "Funds transfer",
            },
        ],
    )
}

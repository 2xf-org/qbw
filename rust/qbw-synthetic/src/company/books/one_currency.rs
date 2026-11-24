//! A small company of one currency. Its tables are those of the other company, of
//! which a few have rows.
//!
//! No row says which currency it is in, and both preferences about currencies are
//! off. A line repeats its amount as the one in another currency, and the list of
//! currencies is there all the same.

use crate::company::books::literal::{Values, row};
use crate::company::books::table::Table;

/// The tables whose rows every company has.
const SHARED: &[&str] = &["CF_SYSINFO", "abmc_currency", "cf_pref_attribute"];

/// The tables of the company, which are `books` with other rows.
pub(super) fn tables(books: Vec<Table>) -> Vec<Table> {
    let tables = books.into_iter().map(|table| {
        if SHARED.contains(&table.name()) {
            return table;
        }
        let rows = rows(table.name());
        table.emptied().without_currency().rows(Vec::new(), rows)
    });
    tables.collect()
}

fn rows(table: &str) -> Vec<Values> {
    match table {
        "cf_pref_attribute_value" => preferences(),
        "abmc_audit_header" => saves(),
        "abmc_account_user" => accounts(),
        "abmc_account_internal" => account_totals(),
        "abmc_customer_user" => vec![row! { "customer_id" => 101, "name" => "Ada Stores" }],
        "abmc_customer_internal" => customer_totals(),
        "abmc_vendor_user" => vec![row! { "vendor_id" => 201, "name" => "Cobalt Parts" }],
        "abmc_vendor_internal" => vendor_totals(),
        "abmc_item_user" => items(),
        "abmc_item_internal" => item_totals(),
        "abmc_general_journal_header" => journals(),
        "abmc_general_journal_lineitem" => journal_lines(),
        "abmc_bill_header" => bills(),
        "abmc_bill_lineitem" => bill_lines(),
        "abmc_invoice_header" => invoices(),
        "abmc_invoice_lineitem" => invoice_lines(),
        "abmc_bill_check_header" => bill_payments(),
        "abmc_bill_check_lineitem" => bill_payment_lines(),
        "abmc_received_payment_header" => payments(),
        "abmc_received_payment_lineitem" => payment_lines(),
        "abmc_transaction_link" => links(),
        _ => Vec::new(),
    }
}

/// Rows that have the values `shared` and, over them, their own.
fn sharing(shared: &Values, rows: [Values; 2]) -> Vec<Values> {
    rows.map(|own| [shared.clone(), own].concat()).into()
}

/// The values of the preferences, by the keys that the other company has for
/// them: the name, the start of the fiscal year, and no other currency.
fn preferences() -> Vec<Values> {
    let values = [(1, "Corner Valves"), (12, "1"), (14, "0"), (15, "0")];
    let values = (1..).zip(values).map(|(id, (preference, value))| {
        row! {
            "pref_attribute_value_id" => id,
            "preference_id" => preference,
            "preference_value" => value,
        }
    });
    values.collect()
}

fn saves() -> Vec<Values> {
    vec![row! {
        "transaction_id" => -2147483548,
        "revision_num" => 1,
        "mod_user_id" => 1,
        "mod_user_name" => "Admin",
    }]
}

fn accounts() -> Vec<Values> {
    vec![
        row! { "account_id" => 1, "name" => "Checking", "type" => 0 },
        row! { "account_id" => 3, "name" => "Accounts Receivable", "type" => 1 },
        row! { "account_id" => 7, "name" => "Accounts Payable", "type" => 5 },
        row! { "account_id" => 10, "name" => "Opening Balance Equity", "type" => 9 },
        row! { "account_id" => 11, "name" => "Retained Earnings", "type" => 9 },
        row! { "account_id" => 12, "name" => "Sales", "type" => 10 },
        row! { "account_id" => 15, "name" => "Office", "type" => 12 },
    ]
}

fn account_totals() -> Vec<Values> {
    vec![
        row! {
            "account_id" => 1, "display_index_next" => -3,
            "ending_balance_amt" => "505.00", "ending_balance_home_amt" => "505.00",
        },
        row! {
            "account_id" => 3, "special_type" => 2, "display_index_next" => 7,
            "ending_balance_amt" => "70.00", "ending_balance_home_amt" => "70.00",
        },
        row! {
            "account_id" => 7, "special_type" => 3, "display_index_next" => 10,
            "ending_balance_amt" => "-15.00", "ending_balance_home_amt" => "-15.00",
        },
        row! {
            "account_id" => 10, "special_type" => 1, "display_index_next" => 11,
            "ending_balance_amt" => "-500.00", "ending_balance_home_amt" => "-500.00",
        },
        row! {
            "account_id" => 11, "special_type" => 8, "display_index_next" => 12,
            "ending_balance_amt" => "0.00",
        },
        row! {
            "account_id" => 12, "display_index_next" => 15,
            "ending_balance_amt" => "-120.00", "ending_balance_home_amt" => "-120.00",
        },
        row! {
            "account_id" => 15, "display_index_next" => 1,
            "ending_balance_amt" => "60.00", "ending_balance_home_amt" => "60.00",
        },
    ]
}

fn customer_totals() -> Vec<Values> {
    vec![row! {
        "customer_id" => 101, "open_date" => "2024-01-02", "end_balance_amt" => "70.00",
        "end_balance_home_amt" => "70.00", "list_ident" => "80000065-1",
        "display_index_next" => -201,
    }]
}

fn vendor_totals() -> Vec<Values> {
    vec![row! {
        "vendor_id" => 201, "open_date" => "2024-01-02", "end_balance_amt" => "-15.00",
        "end_balance_home_amt" => "-15.00", "list_ident" => "800000C9-1",
        "display_index_next" => 101,
    }]
}

fn items() -> Vec<Values> {
    vec![row! {
        "item_id" => 501, "name" => "Installation", "item_type" => 0, "post_account_id" => 12,
        "description" => "Installation, per valve", "sales_price_amt" => "40.00",
        "unit_cost_amt" => "0.00",
    }]
}

fn item_totals() -> Vec<Values> {
    vec![row! { "item_id" => 501, "list_ident" => "800001F5-1", "display_index_next" => -501 }]
}

fn journals() -> Vec<Values> {
    vec![row! {
        "transaction_id" => 100, "target_id" => 101, "transaction_date" => "2024-01-02",
        "doc_num" => "1", "memo" => "Opening balance", "trans_num" => 1,
    }]
}

fn journal_lines() -> Vec<Values> {
    let shared = row! {
        "transaction_id" => 100, "transaction_date" => "2024-01-02",
        "transaction_view_type" => 8, "doc_num" => "1", "memo" => "Opening balance",
    };
    let lines = [
        row! {
            "target_id" => 101, "is_source_bool" => true, "next_target_id" => 102,
            "account_id" => 1, "sibling_account_id" => 10, "amount_amt" => "500.00",
            "foreign_amount_amt" => "500.00",
        },
        row! {
            "target_id" => 102, "next_target_id" => 0, "account_id" => 10,
            "sibling_account_id" => 1, "amount_amt" => "-500.00",
            "foreign_amount_amt" => "-500.00",
        },
    ];
    sharing(&shared, lines)
}

fn bills() -> Vec<Values> {
    vec![row! {
        "transaction_id" => 200, "target_id" => 201, "transaction_date" => "2024-01-10",
        "transaction_view_type" => 9, "doc_num" => "7781", "vendor_id" => 201,
        "due_date" => "2024-02-09", "trans_num" => 2,
    }]
}

fn bill_lines() -> Vec<Values> {
    let shared = row! {
        "transaction_id" => 200, "transaction_date" => "2024-01-10",
        "transaction_view_type" => 9, "vendor_id" => 201, "due_date" => "2024-02-09",
    };
    let lines = [
        row! {
            "target_id" => 201, "is_source_bool" => true, "next_target_id" => 202,
            "account_id" => 7, "sibling_account_id" => 15, "amount_amt" => "-60.00",
            "foreign_amount_amt" => "-60.00", "doc_num" => "7781",
            "lineitem_name_type" => 101, "open_balance_amt" => "-15.00",
        },
        row! {
            "target_id" => 202, "next_target_id" => 0, "account_id" => 15,
            "sibling_account_id" => 7, "amount_amt" => "60.00",
            "foreign_amount_amt" => "60.00", "memo" => "Paper",
        },
    ];
    sharing(&shared, lines)
}

/// Check 501 pays a part of bill 7781.
fn bill_payments() -> Vec<Values> {
    vec![row! {
        "transaction_id" => 300, "target_id" => 301, "transaction_date" => "2024-02-05",
        "doc_num" => "501", "vendor_id" => 201, "trans_num" => 3,
    }]
}

fn bill_payment_lines() -> Vec<Values> {
    let shared = row! {
        "transaction_id" => 300, "transaction_date" => "2024-02-05",
        "transaction_view_type" => 14, "vendor_id" => 201, "is_paid_bool" => true,
    };
    let lines = [
        row! {
            "target_id" => 301, "is_source_bool" => true, "next_target_id" => 302,
            "account_id" => 1, "sibling_account_id" => 7, "amount_amt" => "-45.00",
            "foreign_amount_amt" => "-45.00", "doc_num" => "501", "lineitem_name_type" => 101,
        },
        row! {
            "target_id" => 302, "next_target_id" => 0, "account_id" => 7,
            "sibling_account_id" => 1, "amount_amt" => "45.00", "foreign_amount_amt" => "45.00",
        },
    ];
    sharing(&shared, lines)
}

fn invoices() -> Vec<Values> {
    vec![row! {
        "transaction_id" => 400, "target_id" => 401, "transaction_date" => "2024-01-15",
        "doc_num" => "1041", "customer_id" => 101, "due_date" => "2024-02-14", "trans_num" => 4,
    }]
}

fn invoice_lines() -> Vec<Values> {
    let shared = row! {
        "transaction_id" => 400, "transaction_date" => "2024-01-15",
        "transaction_view_type" => 4, "customer_id" => 101, "due_date" => "2024-02-14",
    };
    let lines = [
        row! {
            "target_id" => 401, "is_source_bool" => true, "next_target_id" => 402,
            "account_id" => 3, "sibling_account_id" => 12, "amount_amt" => "120.00",
            "foreign_amount_amt" => "120.00", "doc_num" => "1041", "lineitem_name_type" => 100,
            "open_balance_amt" => "70.00",
        },
        row! {
            "target_id" => 402, "next_target_id" => 0, "account_id" => 12,
            "sibling_account_id" => 3, "amount_amt" => "-120.00",
            "foreign_amount_amt" => "-120.00", "memo" => "Installation, per valve",
            "item_id" => 501, "quantity_qnty" => "-3.00", "unit_price_amt" => "40.00",
        },
    ];
    sharing(&shared, lines)
}

/// Payment 88 pays a part of invoice 1041.
fn payments() -> Vec<Values> {
    vec![row! {
        "transaction_id" => 500, "target_id" => 501, "transaction_date" => "2024-02-20",
        "doc_num" => "88", "customer_id" => 101, "is_deposited_bool" => true, "trans_num" => 5,
    }]
}

fn payment_lines() -> Vec<Values> {
    let shared = row! {
        "transaction_id" => 500, "transaction_date" => "2024-02-20",
        "transaction_view_type" => 7, "customer_id" => 101, "is_paid_bool" => true,
    };
    let lines = [
        row! {
            "target_id" => 501, "is_source_bool" => true, "next_target_id" => 502,
            "account_id" => 1, "sibling_account_id" => 3, "amount_amt" => "50.00",
            "foreign_amount_amt" => "50.00", "doc_num" => "88", "lineitem_name_type" => 100,
        },
        row! {
            "target_id" => 502, "next_target_id" => 0, "account_id" => 3,
            "sibling_account_id" => 1, "amount_amt" => "-50.00", "foreign_amount_amt" => "-50.00",
        },
    ];
    sharing(&shared, lines)
}

/// Both links from each of their ends.
fn links() -> Vec<Values> {
    vec![
        row! {
            "link_id" => 1, "transaction_id" => 300, "target_1_id" => 302,
            "transaction_2_id" => 200, "target_2_id" => 201, "is_t1_source_bool" => true,
            "link_type" => 6, "amount_amt" => "-45.00",
        },
        row! {
            "link_id" => 2, "transaction_id" => 200, "target_1_id" => 201,
            "transaction_2_id" => 300, "target_2_id" => 302, "link_type" => 6,
            "amount_amt" => "-45.00",
        },
        row! {
            "link_id" => 3, "transaction_id" => 500, "target_1_id" => 502,
            "transaction_2_id" => 400, "target_2_id" => 402, "is_t1_source_bool" => true,
            "link_type" => 4, "amount_amt" => "-50.00",
        },
        row! {
            "link_id" => 4, "transaction_id" => 400, "target_1_id" => 402,
            "transaction_2_id" => 500, "target_2_id" => 502, "link_type" => 4,
            "amount_amt" => "-50.00",
        },
    ]
}

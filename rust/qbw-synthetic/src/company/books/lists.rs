//! The small lists.

use crate::company::books::literal::{Literal, row};
use crate::company::books::table::Table;

pub(super) fn tables() -> Vec<Table> {
    vec![
        currencies(),
        exchange_rates(),
        terms(),
        payment_methods(),
        ship_methods(),
        sales_reps(),
        types("abmc_customer_type", "customer_type_id integer key").row(row! {
            "customer_type_id" => 1, "name" => "Retail", "display_index_next" => -1,
        }),
        types("abmc_vendor_type", "vendor_type_id integer key").row(row! {
            "vendor_type_id" => 1, "name" => "Supplier", "display_index_next" => -1,
        }),
        types("abmc_job_type", "job_type_id integer key").row(row! {
            "job_type_id" => 1, "name" => "Building", "display_index_next" => -1,
        }),
        sales_tax_codes(),
        price_levels(),
        price_level_items(),
        messages(),
        templates(),
        memorized(),
        custom_field_definitions(),
        custom_fields(),
        classes(),
        sites(),
        units(),
    ]
}

fn currencies() -> Table {
    Table::new(
        "abmc_currency",
        &["
            currency_id integer key
            code varchar(17) null
            name varchar(65)
            symbol varchar(17) null
            decimal_places integer = 2
            is_hidden_bool bit
            is_deleted_bool bit
        "],
    )
    .row(row! { "currency_id" => 1, "code" => "USD", "name" => "US Dollar", "symbol" => "$" })
    .row(row! {
        "currency_id" => 2, "code" => "EUR", "name" => "Euro",
        // A euro sign in UTF-8, whatever the encoding of the file.
        "symbol" => Literal::Bytes(b"\xe2\x82\xac"),
    })
    .row(row! {
        "currency_id" => 3, "code" => "JPY", "name" => "Yen", "decimal_places" => 0,
        "symbol" => Literal::Bytes(b"\xc2\xa5"), "is_hidden_bool" => true,
    })
}

fn exchange_rates() -> Table {
    Table::new(
        "abmc_exchange_rate_history",
        &["
            exchange_rate_history_id integer key
            currency_id integer null -> abmc_currency
            as_of_date date null
            exchange_rate float null
            is_deleted_bool bit
        "],
    )
    .row(row! {
        "exchange_rate_history_id" => 1, "currency_id" => 2, "as_of_date" => "2024-02-10",
        "exchange_rate" => "1.10",
    })
    .row(row! {
        "exchange_rate_history_id" => 2, "currency_id" => 2, "as_of_date" => "2024-03-10",
        "exchange_rate" => "1.20",
    })
}

fn terms() -> Table {
    Table::new(
        "abmc_terms",
        &["
            terms_id integer key
            name varchar(32) null
            terms_type smallint null = 0
            standard_due_days smallint null = 0
            standard_discount_days smallint null = 0
            discount_pct numeric(20,5) null
            day_of_month_due smallint null = 0
            date_mininmum_days smallint null = 0
            discount_day_of_month smallint null = 0
            is_hidden_bool bit
            is_deleted_bool bit
        "],
    )
    .row(row! {
        "terms_id" => 1, "name" => "2% 10 Net 30", "standard_due_days" => 30,
        "standard_discount_days" => 10, "discount_pct" => "2.00",
    })
    .row(row! {
        "terms_id" => 2, "name" => "Due on the 5th", "terms_type" => 1, "day_of_month_due" => 5,
        "date_mininmum_days" => 10, "discount_day_of_month" => 1,
    })
}

fn payment_methods() -> Table {
    Table::new(
        "abmc_payment_method",
        &["
            payment_method_id integer key
            name varchar(32) null
            payment_method_type smallint null = 0
            is_hidden_bool bit
            is_deleted_bool bit
        "],
    )
    .row(row! { "payment_method_id" => 1, "name" => "Check", "payment_method_type" => 1 })
    .row(row! { "payment_method_id" => 2, "name" => "Wire" })
}

fn ship_methods() -> Table {
    Table::new(
        "abmc_shipping",
        &["
            shipping_id integer key
            name varchar(16) null
            is_hidden_bool bit
            is_deleted_bool bit
        "],
    )
    .row(row! { "shipping_id" => 1, "name" => "Courier" })
}

fn sales_reps() -> Table {
    Table::new(
        "abmc_sales_rep",
        &["
            sales_rep_id integer key
            name_id integer null
            initials varchar(6) null
            is_hidden_bool bit
            is_deleted_bool bit
        "],
    )
    .row(row! { "sales_rep_id" => 1, "name_id" => 301, "initials" => "EM" })
}

/// A list of types, with the key that `key` describes.
fn types(table: &str, key: &'static str) -> Table {
    let columns = "
        name varchar(32) null
        parent_id integer null
        sublevel smallint null = 0
        is_hidden_bool bit
        is_deleted_bool bit
        display_index_next integer
    ";
    Table::new(table, &[key, columns])
}

fn sales_tax_codes() -> Table {
    Table::new(
        "abmc_sales_tax_code",
        &["
            sales_tax_code_id integer key
            code varchar(4) null
            description varchar(32) null
            is_taxable_bool bit
            is_hidden_bool bit
            is_deleted_bool bit
        "],
    )
    .row(row! {
        "sales_tax_code_id" => 1, "code" => "Tax", "description" => "Taxable sales",
        "is_taxable_bool" => true,
    })
    .row(row! {
        "sales_tax_code_id" => 2, "code" => "Non", "description" => "Non-taxable sales",
    })
}

fn price_levels() -> Table {
    Table::new(
        "abmc_price_level",
        &["
            price_level_id integer key
            name varchar(32) null
            price_level_type smallint null = 0
            value_pct numeric(20,5) null
            currency_id integer null -> abmc_currency
            is_hidden_bool bit
            is_deleted_bool bit
        "],
    )
    .row(row! { "price_level_id" => 1, "name" => "Trade", "value_pct" => "-5.00" })
    .row(row! { "price_level_id" => 2, "name" => "Dealers", "price_level_type" => 1 })
}

fn price_level_items() -> Table {
    Table::new(
        "abmc_price_level_item",
        &["
            price_level_id integer key -> abmc_price_level
            item_id integer key -> abmc_item_user
            computed_price_amt numeric(20,5) null
            computed_price_pct numeric(20,5) null
        "],
    )
    .row(row! { "price_level_id" => 2, "item_id" => 502, "computed_price_amt" => "45.00" })
}

fn messages() -> Table {
    Table::new(
        "abmc_customer_message",
        &["
            customer_message_id integer key
            cust_message varchar(102) null
            is_hidden_bool bit
            is_deleted_bool bit
        "],
    )
    .row(row! { "customer_message_id" => 1, "cust_message" => "Thank you for your order." })
}

fn templates() -> Table {
    Table::new(
        "abmc_custom_style",
        &["
            custom_style_id integer key
            name varchar(42) null
            style_view_type smallint null
            is_hidden_bool bit
            is_deleted_bool bit
        "],
    )
    .row(row! { "custom_style_id" => 1, "name" => "Product invoice", "style_view_type" => 0 })
}

fn memorized() -> Table {
    Table::new(
        "abmc_memorized_transactions",
        &["
            memorized_transactions_id integer key
            name varchar(32) null
            master_record_id integer null
            parent_id integer null
            currency_id integer null -> abmc_currency = 1
            is_hidden_bool bit
            is_deleted_bool bit
            display_index_next integer
        "],
    )
    .row(row! {
        "memorized_transactions_id" => 1, "name" => "Monthly fuel", "master_record_id" => 1900,
        "display_index_next" => -1,
    })
}

fn custom_field_definitions() -> Table {
    Table::new(
        "abmc_custom_field_type",
        &["
            custom_field_type_id integer key
            custom_field_name varchar(32)
            custom_field_position integer
            is_for_customer_bool bit
            is_for_vendor_bool bit
            is_for_employee_bool bit
            is_for_item_bool bit
        "],
    )
    .row(row! {
        "custom_field_type_id" => 1, "custom_field_name" => "Region",
        "custom_field_position" => 0, "is_for_customer_bool" => true,
    })
    // The positions of item fields start at 30, and their values are stored from 0.
    .row(row! {
        "custom_field_type_id" => 2, "custom_field_name" => "Color",
        "custom_field_position" => 30, "is_for_item_bool" => true,
    })
}

fn custom_fields() -> Table {
    Table::new(
        "abmc_custom_field",
        &["
            abm_id varchar(17) key
            position integer key
            field varchar(31) null
        "],
    )
    .row(row! { "abm_id" => "NAME000000000101", "position" => 0, "field" => "North" })
    .row(row! { "abm_id" => "ITEM000000000502", "position" => 0, "field" => "Gold" })
    .row(row! { "abm_id" => "TXN0000000000400", "position" => 0, "field" => "North" })
    .row(row! { "abm_id" => "LINE000000000402", "position" => 0, "field" => "Gold" })
}

/// The company uses no classes.
fn classes() -> Table {
    Table::new(
        "abmc_class",
        &["
            class_id integer key
            name varchar(32) null
            parent_id integer null
            is_hidden_bool bit
            is_deleted_bool bit
            display_index_next integer
        "],
    )
}

/// The company keeps its stock at one site, which has no row.
fn sites() -> Table {
    Table::new(
        "abmc_site",
        &["
            site_id integer key
            name varchar(32) null
            parent_id integer null -> abmc_site
            special_type smallint null
            description varchar(101) null
            is_hidden_bool bit
            is_deleted_bool bit
            display_index_next integer
        "],
    )
}

/// The company counts in no units of measure.
fn units() -> Table {
    Table::new(
        "abmc_unit",
        &["
            unit_id integer key
            name varchar(32) null
        "],
    )
}

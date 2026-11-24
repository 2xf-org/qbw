//! Items: what the company buys, sells and taxes.

use crate::company::books::literal::row;
use crate::company::books::table::Table;

pub(super) fn tables() -> Vec<Table> {
    vec![items(), totals(), members(), sites_of_items(), categories()]
}

fn items() -> Table {
    Table::new(
        "abmc_item_user",
        &["
            item_id integer key
            name varchar(32) null
            parent_id integer null = 0
            sublevel smallint null = 0
            item_type smallint null = 0
            post_account_id integer null = 0
            cogs_account_id integer null = 0
            asset_account_id integer null = 0
            is_pass_thru_bool bit
            is_group_bool bit
            print_grouped_bool bit
            description varchar(4096) null
            purchase_description varchar(4096) null
            sales_price_amt numeric(20,5) null
            sales_price_qnty numeric(20,5) null
            sales_price_pct numeric(20,5) null
            unit_cost_amt numeric(20,5) null
            unit_cost_qnty numeric(20,5) null
            preferred_vendor_id integer null = 0
            tax_agency_id integer null = 0
            sales_tax_code_id integer null = 0
            manufacturer_part_num varchar(32) null
            reorder_point_qnty numeric(20,5) null
            assembly_build_point_qnty numeric(20,5) null
            class_id integer null -> abmc_class
            unit_set_id integer null -> abmc_unit
            category_id integer = 1
            item_abm_id varchar(17) null
            is_hidden_bool bit
            is_deleted_bool bit
            timestamp_tms integer null = 0
            modtimestamp_tms integer null = 0
        "],
    )
    .row(row! {
        "item_id" => 501, "name" => "Installation", "item_type" => 0, "post_account_id" => 12,
        "description" => "Installation, per valve", "sales_price_amt" => "40.00",
        "unit_cost_amt" => "0.00", "sales_tax_code_id" => 2, "item_abm_id" => "ITEM000000000501",
    })
    .row(row! {
        "item_id" => 502, "name" => "Valve", "item_type" => 1, "post_account_id" => 13,
        "cogs_account_id" => 14, "asset_account_id" => 6, "description" => "Brass valve",
        "purchase_description" => "Valve, brass", "sales_price_amt" => "50.00",
        "unit_cost_qnty" => "20.00", "preferred_vendor_id" => 201, "sales_tax_code_id" => 1,
        "manufacturer_part_num" => "CV-20", "reorder_point_qnty" => "12.00",
        "item_abm_id" => "ITEM000000000502",
    })
    .row(row! {
        "item_id" => 503, "name" => "State tax", "item_type" => 10, "post_account_id" => 9,
        "sales_price_pct" => "8.00", "unit_cost_amt" => "0.00", "tax_agency_id" => 202,
        "item_abm_id" => "ITEM000000000503",
    })
    .row(row! {
        "item_id" => 504, "name" => "Subtotal", "item_type" => 6, "unit_cost_amt" => "0.00",
        "item_abm_id" => "ITEM000000000504",
    })
    .row(row! {
        "item_id" => 505, "name" => "Valve kit", "item_type" => 7, "unit_cost_amt" => "0.00",
        "print_grouped_bool" => true, "item_abm_id" => "ITEM000000000505",
    })
    .row(row! {
        "item_id" => 506, "name" => "Discount", "item_type" => 8, "post_account_id" => 18,
        "sales_price_amt" => "0.00", "unit_cost_amt" => "0.00", "sales_tax_code_id" => 2,
        "item_abm_id" => "ITEM000000000506",
    })
    .row(row! {
        "item_id" => 507, "name" => "Twin valve", "item_type" => 2, "post_account_id" => 13,
        "cogs_account_id" => 14, "asset_account_id" => 6, "sales_price_amt" => "110.00",
        "unit_cost_amt" => "40.00", "sales_tax_code_id" => 1,
        "assembly_build_point_qnty" => "2.00", "item_abm_id" => "ITEM000000000507",
    })
    .row(row! {
        "item_id" => 508, "is_deleted_bool" => true, "reorder_point_qnty" => "0.00",
        "item_abm_id" => "ITEM000000000508",
    })
}

fn totals() -> Table {
    Table::new(
        "abmc_item_internal",
        &["
            item_id integer key -> abmc_item_user
            special_type smallint null = 0
            on_hand_qnty numeric(20,5) null
            value_on_hand_amt numeric(20,5) null
            average_cost_qnty numeric(20,5) null
            on_order_qnty numeric(20,5) null
            sales_order_qnty numeric(20,5) null
            to_be_built_by_pending_build_txns_qnty numeric(20,5) null
            required_by_pending_build_txns_qnty numeric(20,5) null
            list_ident varchar(65)
            display_index_next integer
        "],
    )
    .row(row! { "item_id" => 501, "list_ident" => "800001F5-1", "display_index_next" => -502 })
    .row(row! {
        "item_id" => 502, "on_hand_qnty" => "8.00", "value_on_hand_amt" => "160.00",
        "average_cost_qnty" => "20.00", "on_order_qnty" => "3.00", "sales_order_qnty" => "4.00",
        "to_be_built_by_pending_build_txns_qnty" => "0.00",
        "required_by_pending_build_txns_qnty" => "0.00", "list_ident" => "800001F6-1",
        "display_index_next" => 507,
    })
    .row(row! { "item_id" => 503, "list_ident" => "800001F7-1", "display_index_next" => 508 })
    .row(row! {
        "item_id" => 504, "special_type" => 3, "list_ident" => "800001F8-1",
        "display_index_next" => 505,
    })
    .row(row! {
        "item_id" => 505, "special_type" => 1, "list_ident" => "800001F9-1",
        "display_index_next" => 506,
    })
    .row(row! { "item_id" => 506, "list_ident" => "800001FA-1", "display_index_next" => 503 })
    .row(row! {
        "item_id" => 507, "on_hand_qnty" => "1.00", "value_on_hand_amt" => "40.00",
        "average_cost_qnty" => "40.00", "on_order_qnty" => "0.00", "sales_order_qnty" => "0.00",
        "to_be_built_by_pending_build_txns_qnty" => "0.00",
        "required_by_pending_build_txns_qnty" => "0.00", "list_ident" => "800001FB-1",
        "display_index_next" => 504,
    })
    .row(row! { "item_id" => 508, "list_ident" => "800001FC-1", "display_index_next" => 501 })
}

/// What groups and assemblies are made of.
fn members() -> Table {
    Table::new(
        "abmc_grouped_items",
        &["
            item_id integer key
            position integer key
            sub_item_id integer null
            quantity_qnty numeric(20,5) null
            unit_id integer null -> abmc_unit
        "],
    )
    .row(row! {
        "item_id" => 505, "position" => 0, "sub_item_id" => 502, "quantity_qnty" => "2.00",
    })
    .row(row! {
        "item_id" => 505, "position" => 1, "sub_item_id" => 501, "quantity_qnty" => "1.00",
    })
    .row(row! {
        "item_id" => 507, "position" => 0, "sub_item_id" => 502, "quantity_qnty" => "2.00",
    })
}

/// The company keeps its stock at one site, so this has no rows.
fn sites_of_items() -> Table {
    Table::new(
        "abmc_item_site",
        &["
            item_site_id integer key
            item_id integer null -> abmc_item_user
            site_id integer null -> abmc_site
            qnty_on_hand numeric(20,5) null
            reorder_level numeric(20,5) null
            is_deleted_bool bit
        "],
    )
}

fn categories() -> Table {
    Table::new(
        "abmc_categories",
        &["
            category_id integer key
            category_name varchar(256) null
            parent_id integer null -> abmc_categories
            is_hidden_bool bit
            is_deleted_bool bit
            display_index_next integer
        "],
    )
    .row(row! {
        "category_id" => 1, "category_name" => "Uncategorized", "display_index_next" => -1,
    })
}

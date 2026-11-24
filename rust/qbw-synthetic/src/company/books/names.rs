//! Customers and jobs, vendors, employees, other names, and their contacts.
//!
//! The four lists share their keys. One chain goes through all of them:
//! 101, 103, 102, 201, 202, 301, 401, and the deleted 104.

use crate::company::books::literal::row;
use crate::company::books::table::Table;

pub(super) fn tables() -> Vec<Table> {
    vec![
        customers(),
        customer_totals(),
        vendors(),
        vendor_totals(),
        employees(),
        other_names(),
        other_name_totals(),
        contacts(),
        channels(),
        channel_labels(),
        ship_to_addresses(),
    ]
}

fn customers() -> Table {
    Table::new(
        "abmc_customer_user",
        &["
            customer_id integer key
            name varchar(42) null
            parent_id integer null = 0
            sublevel smallint null = 0
            customer_type_id integer null = 0
            terms_id integer null
            sales_rep_id integer null = 0
            price_level_id integer null = 0
            payment_method_id integer null = 0
            sales_tax_code_id integer null = 0
            tax_item_id integer null = 0
            currency_id integer null -> abmc_currency = 1
            credit_limit_amt numeric(20,5) null
            ship_to_abm_id varchar(17) null
            job_status_type smallint null = 0
            job_type_id integer null = 0
            job_start_date date null
            job_projected_end_date date null
            job_end_date date null
            job_description varchar(100) null
            class_id integer null -> abmc_class
            resale_num varchar(21) null
            customer_abm_id varchar(17) null
            is_hidden_bool bit
            is_deleted_bool bit
            timestamp_tms integer null = 0
            modtimestamp_tms integer null = 0
        "],
    )
    .row(row! {
        "customer_id" => 101, "name" => "Ada Stores", "customer_type_id" => 1, "terms_id" => 1,
        "sales_rep_id" => 1, "price_level_id" => 1, "payment_method_id" => 1,
        "sales_tax_code_id" => 1, "tax_item_id" => 503, "credit_limit_amt" => "5000.00",
        "ship_to_abm_id" => "SHIPTO0000000101", "customer_abm_id" => "NAME000000000101",
        "timestamp_tms" => 1677657600, "modtimestamp_tms" => 1699999200,
    })
    .row(row! {
        "customer_id" => 102, "name" => "Bell Works", "terms_id" => 1, "sales_tax_code_id" => 2,
        "currency_id" => 2, "customer_abm_id" => "NAME000000000102",
    })
    .row(row! {
        "customer_id" => 103, "name" => "Annex", "parent_id" => 101, "sublevel" => 1,
        "job_status_type" => 3, "job_start_date" => "2024-04-01",
        "job_description" => "New wing", "customer_abm_id" => "NAME000000000103",
    })
    .row(row! {
        "customer_id" => 104, "is_deleted_bool" => true, "customer_abm_id" => "NAME000000000104",
    })
}

fn customer_totals() -> Table {
    Table::new(
        "abmc_customer_internal",
        &["
            customer_id integer key -> abmc_customer_user
            open_date date null
            end_balance_amt numeric(20,5) null
            end_balance_home_amt numeric(20,5) null
            has_unbilled_reimbursable_bool bit
            list_ident varchar(65)
            display_index_next integer
        "],
    )
    .row(row! {
        "customer_id" => 101, "open_date" => "2023-03-01", "end_balance_amt" => "45.00",
        "end_balance_home_amt" => "45.00", "list_ident" => "80000065-1", "display_index_next" => -103,
    })
    .row(row! {
        "customer_id" => 102, "open_date" => "2024-02-01", "end_balance_amt" => "0.00",
        "end_balance_home_amt" => "0.00", "list_ident" => "80000066-1", "display_index_next" => 201,
    })
    .row(row! {
        "customer_id" => 103, "open_date" => "2024-04-01", "end_balance_amt" => "0.00",
        "end_balance_home_amt" => "0.00", "list_ident" => "80000067-1", "display_index_next" => 102,
    })
    .row(row! {
        "customer_id" => 104, "end_balance_amt" => "0.00", "end_balance_home_amt" => "0.00",
        "list_ident" => "80000068-1", "display_index_next" => 101,
    })
}

fn vendors() -> Table {
    Table::new(
        "abmc_vendor_user",
        &["
            vendor_id integer key
            name varchar(42) null
            vendor_type_id integer null = 0
            terms_id integer null
            currency_id integer null -> abmc_currency = 1
            credit_limit_amt numeric(20,5) null
            name_on_check varchar(42) null
            is_1099_vendor_bool bit
            is_tax_agency_bool bit
            vendor_abm_id varchar(17) null
            is_hidden_bool bit
            is_deleted_bool bit
            timestamp_tms integer null = 0
            modtimestamp_tms integer null = 0
        "],
    )
    .row(row! {
        "vendor_id" => 201, "name" => "Cobalt Parts", "vendor_type_id" => 1, "terms_id" => 1,
        "name_on_check" => "Cobalt Parts Inc", "is_1099_vendor_bool" => true,
        "vendor_abm_id" => "NAME000000000201",
    })
    .row(row! {
        "vendor_id" => 202, "name" => "State Tax Office", "vendor_abm_id" => "NAME000000000202",
    })
}

fn vendor_totals() -> Table {
    Table::new(
        "abmc_vendor_internal",
        &["
            vendor_id integer key -> abmc_vendor_user
            open_date date null
            end_balance_amt numeric(20,5) null
            end_balance_home_amt numeric(20,5) null
            list_ident varchar(65)
            display_index_next integer
        "],
    )
    .row(row! {
        "vendor_id" => 201, "open_date" => "2023-05-01", "end_balance_amt" => "-20.00",
        "end_balance_home_amt" => "-20.00", "list_ident" => "800000C9-1",
        "display_index_next" => 202,
    })
    .row(row! {
        "vendor_id" => 202, "open_date" => "2023-11-01", "end_balance_amt" => "0.00",
        "end_balance_home_amt" => "0.00", "list_ident" => "800000CA-1",
        "display_index_next" => 301,
    })
}

fn employees() -> Table {
    Table::new(
        "abmc_employee",
        &["
            employee_id integer key
            name varchar(42) null
            open_date date null
            employee_abm_id varchar(17) null
            name_on_check varchar(42) null
            list_ident varchar(65)
            display_index_next integer
            is_hidden_bool bit
            is_deleted_bool bit
            timestamp_tms integer null = 0
            modtimestamp_tms integer null = 0
        "],
    )
    .row(row! {
        "employee_id" => 301, "name" => "Eve Moss", "open_date" => "2023-02-01",
        "list_ident" => "8000012D-1", "display_index_next" => 401,
    })
}

fn other_names() -> Table {
    Table::new(
        "abmc_other_user",
        &["
            other_id integer key
            name varchar(42) null
            other_abm_id varchar(17) null
            currency_id integer null -> abmc_currency
            class_id integer null -> abmc_class
            is_hidden_bool bit
            is_deleted_bool bit
            timestamp_tms integer null = 0
            modtimestamp_tms integer null = 0
        "],
    )
    .row(row! { "other_id" => 401, "name" => "Quay Fuel" })
}

fn other_name_totals() -> Table {
    Table::new(
        "abmc_other_internal",
        &["
            other_id integer key -> abmc_other_user
            open_date date null
            list_ident varchar(65)
            display_index_next integer
        "],
    )
    .row(row! { "other_id" => 401, "list_ident" => "80000191-1", "display_index_next" => 104 })
}

/// A name has one row that has not expired. The others are what forms of
/// earlier days point at, and the people to ask for.
fn contacts() -> Table {
    Table::new(
        "abmc_contacts",
        &["
            contacts_abm_id varchar(17) key
            name_id integer null
            expired_date date null
            additional_contacts_type smallint = 0
            alternate_id integer null
            entity_id integer null
            company_name varchar(42) null
            salutation varchar(16) null
            first_name varchar(26) null
            middle_initial varchar(6) null
            last_name varchar(26) null
            title varchar(42) null
            contact1 varchar(42) null
            contact2 varchar(42) null
            telephone_num1 varchar(22) null
            telephone_num2 varchar(22) null
            cellular_num varchar(22) null
            fax_num varchar(22) null
            email varchar(1024) null
            emailCC varchar(1024) null
            account_num varchar(100) null
            address_line1 varchar(42) null
            address_line2 varchar(42) null
            address_line3 varchar(42) null
            address_line4 varchar(42) null
            address_line5 varchar(42) null
            street_line1 varchar(42) null
            street_line2 varchar(42) null
            street_line3 varchar(42) null
            street_line4 varchar(42) null
            street_line5 varchar(42) null
            city varchar(32) null
            state varchar(22) null
            postal_code varchar(14) null
            country char(32) null
        "],
    )
    .row(row! {
        "contacts_abm_id" => "CONTACT000000101", "name_id" => 101, "company_name" => "Ada Stores",
        "first_name" => "Ada", "last_name" => "Reed", "telephone_num1" => "555-0101",
        "email" => "ada@ada-stores.example", "account_num" => "A-101",
        "address_line1" => "Ada Stores", "address_line2" => "7 Mill Lane",
        "address_line3" => "Northport, NP 10002", "street_line1" => "7 Mill Lane",
        "city" => "Northport", "state" => "NP", "postal_code" => "10002",
    })
    // The address before Ada Stores moved, which invoice 1041 still has.
    .row(row! {
        "contacts_abm_id" => "CONTACT000000100", "name_id" => 101, "expired_date" => "2024-01-02",
        "company_name" => "Ada Stores", "address_line1" => "Ada Stores",
        "address_line2" => "3 Old Road", "street_line1" => "3 Old Road", "city" => "Northport",
        "state" => "NP", "postal_code" => "10002",
    })
    .row(row! {
        "contacts_abm_id" => "CONTACT000000109", "additional_contacts_type" => 1,
        "alternate_id" => 101, "entity_id" => 1, "first_name" => "Sam", "last_name" => "Reed",
        "title" => "Buyer",
    })
    .row(row! {
        "contacts_abm_id" => "CONTACT000000102", "name_id" => 102, "company_name" => "Bell Works",
        "address_line1" => "Bell Works", "address_line2" => "Hafenstr. 4", "city" => "Hamburg",
        "country" => "Germany",
    })
    .row(row! { "contacts_abm_id" => "CONTACT000000103", "name_id" => 103 })
    .row(row! { "contacts_abm_id" => "CONTACT000000104", "name_id" => 104 })
    .row(row! {
        "contacts_abm_id" => "CONTACT000000201", "name_id" => 201,
        "company_name" => "Cobalt Parts Inc", "telephone_num1" => "555-0201",
        "address_line1" => "Cobalt Parts Inc", "address_line2" => "9 Foundry Way",
        "city" => "Easton", "state" => "EA", "postal_code" => "20001",
    })
    .row(row! { "contacts_abm_id" => "CONTACT000000202", "name_id" => 202 })
    .row(row! {
        "contacts_abm_id" => "CONTACT000000301", "name_id" => 301, "first_name" => "Eve",
        "last_name" => "Moss",
    })
    .row(row! { "contacts_abm_id" => "CONTACT000000401", "name_id" => 401 })
}

fn channels() -> Table {
    Table::new(
        "abmc_contacts_info",
        &["
            name_id integer key
            channel_category_id integer key
            position integer
            field_value varchar(254) null
            is_deleted_bool bit
        "],
    )
    .row(row! {
        "name_id" => 101, "channel_category_id" => 1, "position" => 1, "field_value" => "555-0101",
    })
    .row(row! {
        "name_id" => 101, "channel_category_id" => 9, "position" => 5,
        "field_value" => "ada@ada-stores.example",
    })
    .row(row! { "name_id" => 101, "channel_category_id" => 5, "position" => 4 })
}

fn channel_labels() -> Table {
    Table::new(
        "abmc_contacts_channel",
        &["
            channel_category_id integer key
            channel_category_name varchar(42)
        "],
    )
    .row(row! { "channel_category_id" => 1, "channel_category_name" => "Main Phone" })
    .row(row! { "channel_category_id" => 5, "channel_category_name" => "Fax" })
    .row(row! { "channel_category_id" => 9, "channel_category_name" => "Main Email" })
}

/// The addresses in a customer's list have a label and have not expired. The
/// others belong to the forms that used them.
fn ship_to_addresses() -> Table {
    Table::new(
        "abmc_ship_to",
        &["
            ship_to_abm_id varchar(17) key
            name_id integer
            expired_date date null
            name varchar(21) null
            address_line1 varchar(42) null
            address_line2 varchar(42) null
            address_line3 varchar(42) null
            address_line4 varchar(42) null
            address_line5 varchar(42) null
            street_line1 varchar(42) null
            street_line2 varchar(42) null
            street_line3 varchar(42) null
            street_line4 varchar(42) null
            street_line5 varchar(42) null
            city varchar(32) null
            state varchar(22) null
            postal_code varchar(14) null
            country varchar(32) null
        "],
    )
    .row(row! {
        "ship_to_abm_id" => "SHIPTO0000000101", "name_id" => 101, "name" => "Warehouse",
        "address_line1" => "Ada Stores", "address_line2" => "Dock 2", "city" => "Northport",
        "state" => "NP", "postal_code" => "10003",
    })
    .row(row! {
        "ship_to_abm_id" => "SHIPTO0000000400", "name_id" => 101, "expired_date" => "2023-11-15",
        "address_line1" => "Ada Stores", "address_line2" => "Dock 1", "city" => "Northport",
    })
}

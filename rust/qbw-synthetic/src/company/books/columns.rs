//! The columns that the tables of transactions share.

use crate::company::books::table::Table;

/// The columns of every header.
const HEADER: &str = "
    transaction_id integer key
    target_id integer
    transaction_date date null
    doc_num varchar(21) null
    memo varchar(4096) null
    currency_id integer null -> abmc_currency = 1
    exchange_rate float = 1
    last_auditable_action smallint = 1
    last_edit_by integer null = 1
    trans_num integer null
    timestamp_tms integer null
    modtimestamp_tms integer null
    abm_id varchar(17)
    is_no_post_bool bit
    is_memorized_transaction_bool bit
";

/// The columns of every line.
const LINE: &str = "
    target_id integer key
    transaction_id integer
    transaction_date date null
    transaction_view_type smallint null
    is_source_bool bit
    next_target_id integer null
    account_id integer null -> abmc_account_user
    sibling_account_id integer null
    memo varchar(4096) null
    amount_amt numeric(20,5) null
    foreign_amount_amt numeric(20,5) null
    is_gain_loss_target_bool bit
    doc_num varchar(21) null
    currency_id integer null -> abmc_currency = 1
    exchange_rate float = 1
    txn_line_order integer null
    abm_id varchar(17)
    is_no_post_bool bit
    is_memorized_transaction_bool bit
";

/// The table of the transactions of `form`, with the columns of every header
/// and the ones that `columns` describe.
pub(crate) fn headers(form: &str, columns: &[&'static str]) -> Table {
    let described = [&[HEADER], columns].concat();
    let headers = Table::new(&format!("{form}_header"), &described);
    headers.referring("target_id", &format!("{form}_lineitem"))
}

/// The table of the lines of `form`, with the columns of every line and the
/// ones that `columns` describe.
pub(crate) fn lines(form: &str, columns: &[&'static str]) -> Table {
    let described = [&[LINE], columns].concat();
    let lines = Table::new(&format!("{form}_lineitem"), &described);
    lines.referring("transaction_id", &format!("{form}_header"))
}

/// The table of what the lines of items in stock of `form` post besides.
pub(crate) fn inventory_lines(form: &str) -> Table {
    let columns = "
        inventory_record_id integer key
        target_type integer
        target_id integer
        account_id integer null
        amount_amt numeric(20,5) null
        foreign_amount_amt numeric(20,5) null
        quantity_qnty numeric(20,5) null
    ";
    let lines = Table::new(&format!("{form}_inventory_lineitem"), &[columns]);
    lines.referring("target_id", &format!("{form}_lineitem"))
}

/// The kind, on the headers that have it.
pub(crate) const VIEW_TYPE: &str = "
    transaction_view_type smallint null
";

pub(crate) const CUSTOMER: &str = "
    customer_id integer null -> abmc_customer_user
";

pub(crate) const VENDOR: &str = "
    vendor_id integer null -> abmc_vendor_user
";

pub(crate) const EMPLOYEE_AND_OTHER: &str = "
    employee_id integer null -> abmc_employee
    other_id integer null -> abmc_other_user
";

/// Which of the four names a header or a line has.
pub(crate) const HEADER_NAME_TYPE: &str = "
    transaction_name_type smallint null = 105
";

/// Which of the four names is a line's own.
pub(crate) const LINE_NAME_TYPE: &str = "
    lineitem_name_type smallint null = 105
";

pub(crate) const CONTACT: &str = "
    contact_id varchar(17) null
";

pub(crate) const TERMS: &str = "
    terms_id integer null
";

pub(crate) const DUE_DATE: &str = "
    due_date date null
";

pub(crate) const IS_PAID: &str = "
    is_paid_bool bit
";

/// On a payment, whether it moved no money.
pub(crate) const IS_ARAP: &str = "
    is_arap_bool bit
";

pub(crate) const PRINT_LATER: &str = "
    is_to_be_printed_bool bit
";

/// What the forms that customers get have.
pub(crate) const SALES_FORM: &str = "
    ship_to_id varchar(17) null
    ship_date date null
    sales_rep_id integer null -> abmc_sales_rep
    po_num varchar(26) null
    fob varchar(14) null
    other varchar(30) null
    customer_message_id integer null -> abmc_customer_message
    sales_tax_code_id integer null -> abmc_sales_tax_code
    custom_style_id integer null -> abmc_custom_style
    is_to_be_sent_bool bit
    site_id integer null -> abmc_site
";

pub(crate) const SHIP_VIA: &str = "
    ship_via_id integer null
";

/// The class of a line.
pub(crate) const CLASS: &str = "
    class_id integer null -> abmc_class
";

/// What every line has that a bank can confirm.
pub(crate) const CLEARING: &str = "
    cleared_type smallint null = 0
    reconcile_date date null
    reimbursed_type smallint null = 0
";

pub(crate) const IS_SPLIT: &str = "
    is_split_bool bit
";

/// What is left to pay of an own line.
pub(crate) const OPEN_BALANCE: &str = "
    is_paid_bool bit
    open_balance_amt numeric(20,5) null
";

pub(crate) const DISCOUNT: &str = "
    is_discount_bool bit
    is_applied_discount_bool bit
";

pub(crate) const PAYMENT_METHOD: &str = "
    payment_method_id integer null -> abmc_payment_method
";

/// What the lines that can have an item have.
pub(crate) const ITEM: &str = "
    item_id integer null -> abmc_item_user
    quantity_qnty numeric(20,5) null
    is_groupend_row_bool bit
    site_id integer null -> abmc_site
    unit_id integer null -> abmc_unit
";

pub(crate) const BIN: &str = "
    rsb_id integer null -> abmc_site
";

pub(crate) const PRICE: &str = "
    unit_price_amt numeric(20,5) null
    unit_price_qnty numeric(20,5) null
    unit_price_pct numeric(20,5) null
";

pub(crate) const GROSS_PRICE: &str = "
    gross_unit_price_amt numeric(20,5) null
    gross_unit_price_qnty numeric(20,5) null
";

pub(crate) const PART_NUMBER: &str = "
    manufacturer_part_num varchar(32) null
";

/// How much of an ordered line is done.
pub(crate) const ORDER: &str = "
    open_qnty numeric(20,5) null
    is_received_bool bit
    is_invoiced_bool bit
    is_manually_closed_bool bit
";

pub(crate) const IS_TAX: &str = "
    is_trans_tax_bool bit
";

/// What the lines of the forms that customers get, and of orders, have.
pub(crate) const SALES_LINE: &str = "
    sales_tax_code_id integer null -> abmc_sales_tax_code
    service_date date null
    other1 varchar(30) null
    other2 varchar(30) null
    markup_amt numeric(20,5) null
    markup_amt_pct numeric(20,5) null
    is_inactive_bool bit
";

/// The lines of credit memos.
pub(crate) const CREDIT_LINES: &[&str] = &[
    IS_SPLIT,
    ITEM,
    BIN,
    CUSTOMER,
    VENDOR,
    LINE_NAME_TYPE,
    CLASS,
    PRICE,
    GROSS_PRICE,
    SALES_LINE,
    IS_TAX,
    DISCOUNT,
    OPEN_BALANCE,
    DUE_DATE,
    ORDER,
    CLEARING,
    PAYMENT_METHOD,
];

/// The lines of invoices, estimates and orders.
pub(crate) fn sales_lines() -> Vec<&'static str> {
    [CREDIT_LINES, &[PART_NUMBER]].concat()
}

/// The lines of bills, checks and card charges.
pub(crate) const PURCHASE_LINES: &[&str] = &[
    IS_SPLIT,
    ITEM,
    BIN,
    CUSTOMER,
    VENDOR,
    EMPLOYEE_AND_OTHER,
    LINE_NAME_TYPE,
    CLASS,
    PRICE,
    GROSS_PRICE,
    IS_TAX,
    DISCOUNT,
    OPEN_BALANCE,
    DUE_DATE,
    ORDER,
    CLEARING,
    PART_NUMBER,
];

/// The lines of payments, received and made.
pub(crate) const PAYMENT_LINES: &[&str] = &[
    IS_SPLIT,
    CUSTOMER,
    VENDOR,
    LINE_NAME_TYPE,
    CLASS,
    DISCOUNT,
    OPEN_BALANCE,
    DUE_DATE,
    CLEARING,
    PAYMENT_METHOD,
];

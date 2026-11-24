//! The forms that the company wrote nothing on. A file has the tables of every
//! form.

use crate::company::books::columns::{
    self, CLASS, CLEARING, CONTACT, CREDIT_LINES, CUSTOMER, DUE_DATE, IS_ARAP, IS_PAID, IS_SPLIT,
    ITEM, LINE_NAME_TYPE, OPEN_BALANCE, PAYMENT_LINES, PAYMENT_METHOD, PRICE, PRINT_LATER,
    SALES_FORM, SHIP_VIA, VENDOR,
};
use crate::company::books::table::Table;

pub(super) fn tables() -> Vec<Table> {
    let bill_cards = [VENDOR, CONTACT, IS_ARAP];
    let stock_moves = [IS_SPLIT, ITEM, CLASS, CLEARING];
    let refunds = [CUSTOMER, CONTACT, PAYMENT_METHOD, IS_ARAP];
    let receipts = [
        CUSTOMER,
        CONTACT,
        SALES_FORM,
        SHIP_VIA,
        PAYMENT_METHOD,
        PRINT_LATER,
    ];
    let charge_lines = [
        IS_SPLIT,
        ITEM,
        CUSTOMER,
        LINE_NAME_TYPE,
        CLASS,
        PRICE,
        OPEN_BALANCE,
        DUE_DATE,
        CLEARING,
    ];
    vec![
        columns::headers("abmc_bill_ccard", &bill_cards),
        columns::lines("abmc_bill_ccard", PAYMENT_LINES),
        columns::headers("abmc_inventory_transfer", &[]),
        columns::lines("abmc_inventory_transfer", &stock_moves),
        columns::headers("abmc_statement_charge", &[CUSTOMER, DUE_DATE, IS_PAID]),
        columns::lines("abmc_statement_charge", &charge_lines),
        columns::inventory_lines("abmc_statement_charge"),
        columns::headers("abmc_customer_refund", &refunds),
        columns::lines("abmc_customer_refund", PAYMENT_LINES),
        columns::headers("abmc_sales_receipt", &receipts),
        columns::lines("abmc_sales_receipt", CREDIT_LINES),
        // QuickBooks spells this one with one sale.
        columns::inventory_lines("abmc_sale_receipt")
            .referring("target_id", "abmc_sales_receipt_lineitem"),
    ]
}

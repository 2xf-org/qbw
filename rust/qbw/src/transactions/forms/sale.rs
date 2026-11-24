//! What a form has that customers get.

use chrono::NaiveDate;

use crate::ids::{
    AddressId, MessageId, PaymentMethodId, SalesRepId, SalesTaxCodeId, ShipMethodId, TemplateId,
    TermsId,
};
use crate::tables::headers::SaleRow;
use crate::tables::lines::LineRow;

/// What an invoice, a sales receipt, a credit memo, an estimate, a sales order
/// or a statement charge has. Sales receipts are unchecked.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Sale {
    /// The terms of payment.
    pub terms: Option<TermsId>,
    /// Where the goods go.
    pub ship_to: Option<AddressId>,
    /// When they leave.
    pub ship_date: Option<NaiveDate>,
    /// How they travel.
    pub ship_via: Option<ShipMethodId>,
    /// Who sold them.
    pub sales_rep: Option<SalesRepId>,
    /// The customer's number for its order.
    pub po_number: Option<String>,
    /// Where the goods become the customer's, as the form says it.
    pub fob: Option<String>,
    /// The field that the form has for the company's own use.
    pub other: Option<String>,
    /// The message to the customer.
    pub message: Option<MessageId>,
    /// Whether the customer is taxed.
    pub tax_code: Option<SalesTaxCodeId>,
    /// The template that the form is printed with.
    pub template: Option<TemplateId>,
    /// How a sales receipt was paid.
    pub payment_method: Option<PaymentMethodId>,
    /// The number of the sales order that an invoice was made from.
    pub sales_order_number: Option<String>,
    /// Whether an invoice charges for being late. Unchecked.
    pub is_finance_charge: bool,
    /// Whether an estimate still stands.
    pub is_active: bool,
    /// Whether it waits to be printed.
    pub print_later: bool,
    /// Whether it waits to be sent by e-mail.
    pub email_later: bool,
}

impl Sale {
    pub(crate) fn new(row: SaleRow, own: Option<&LineRow>) -> Self {
        Self {
            terms: row.terms,
            ship_to: row.ship_to,
            ship_date: row.ship_date,
            ship_via: row.ship_via,
            sales_rep: row.sales_rep,
            po_number: row.po_number,
            fob: row.fob,
            other: row.other,
            message: row.message,
            tax_code: row.tax_code,
            template: row.template,
            payment_method: row.payment_method,
            sales_order_number: row.sales_order_number,
            is_finance_charge: row.is_finance_charge,
            is_active: !own.is_some_and(|own| own.is_inactive),
            print_later: row.is_to_be_printed,
            email_later: row.is_to_be_sent,
        }
    }
}

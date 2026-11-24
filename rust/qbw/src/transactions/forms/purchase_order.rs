//! What a purchase order has.

use chrono::NaiveDate;

use crate::ids::{AddressId, NameId, ShipMethodId, SiteId, TemplateId, TermsId};
use crate::tables::headers::PurchaseOrderRow;

/// What a purchase order has.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct PurchaseOrder {
    /// The terms of payment.
    pub terms: Option<TermsId>,
    /// When the goods are expected.
    pub expected: Option<NaiveDate>,
    /// Where the goods go.
    pub ship_to: Option<AddressId>,
    /// Who the goods go to, when that is not the company.
    pub ship_to_name: Option<NameId>,
    /// How they travel.
    pub ship_via: Option<ShipMethodId>,
    /// Where the goods become the company's, as the form says it.
    pub fob: Option<String>,
    /// The field that the form has for the company's own use.
    pub other: Option<String>,
    /// The message to the vendor.
    pub vendor_message: Option<String>,
    /// The template that the form is printed with.
    pub template: Option<TemplateId>,
    /// The site that the goods are for.
    pub site: Option<SiteId>,
    /// Whether it waits to be printed.
    pub print_later: bool,
    /// Whether it waits to be sent by e-mail.
    pub email_later: bool,
}

impl PurchaseOrder {
    pub(crate) fn new(row: PurchaseOrderRow) -> Self {
        Self {
            terms: row.terms,
            expected: row.expected,
            ship_to: row.ship_to,
            ship_to_name: row.ship_to_name,
            ship_via: row.ship_via,
            fob: row.fob,
            other: row.other,
            vendor_message: row.vendor_message,
            template: row.template,
            site: row.site,
            print_later: row.is_to_be_printed,
            email_later: row.is_to_be_sent,
        }
    }
}

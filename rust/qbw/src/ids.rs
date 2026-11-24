//! What records are known by.
//!
//! An id is small and can be copied. It means something only in the company
//! it came from.

mod number_ids;
mod text_ids;

pub use number_ids::{
    AccountId, CategoryId, ClassId, CurrencyId, CustomFieldId, CustomerTypeId, ItemId, JobTypeId,
    LineId, MemorizedId, MessageId, NameId, PaymentMethodId, PriceLevelId, SalesRepId,
    SalesTaxCodeId, ShipMethodId, SiteId, TemplateId, TermsId, TransactionId, UserId, VendorTypeId,
};
pub use text_ids::{AddressId, ContactId, OwnerKey};

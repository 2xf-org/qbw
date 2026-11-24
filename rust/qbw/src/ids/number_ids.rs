//! The ids that the file stores as numbers.

use std::fmt;

use crate::number::FromNumber;

macro_rules! number_ids {
    ($($(#[$doc:meta])* $name:ident,)+) => {$(
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(i32);

        impl $name {
            /// The id that the file stores as `number`.
            #[must_use]
            pub const fn new(number: i32) -> Self {
                Self(number)
            }

            /// The number that the file stores.
            #[must_use]
            pub const fn number(self) -> i32 {
                self.0
            }
        }

        impl FromNumber for $name {
            fn from_number(number: i64) -> Option<Self> {
                i32::try_from(number).ok().map(Self)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    )+};
}

number_ids! {
    /// An account of the chart of accounts.
    AccountId,
    /// A category that items are sorted into.
    CategoryId,
    /// A class.
    ClassId,
    /// A currency.
    CurrencyId,
    /// A custom field that the company defined.
    CustomFieldId,
    /// A customer type.
    CustomerTypeId,
    /// An item: what a company buys and sells.
    ItemId,
    /// A job type.
    JobTypeId,
    /// A line of a transaction. A line of any transaction has its own.
    LineId,
    /// A memorized transaction.
    MemorizedId,
    /// A message that prints on sales forms.
    MessageId,
    /// A customer or job, a vendor, an employee or another name. The four
    /// lists share one set of ids.
    NameId,
    /// A payment method.
    PaymentMethodId,
    /// A price level.
    PriceLevelId,
    /// A sales rep.
    SalesRepId,
    /// A sales tax code.
    SalesTaxCodeId,
    /// A way of shipping.
    ShipMethodId,
    /// An inventory site, or a bin of one.
    SiteId,
    /// A template that forms are printed with.
    TemplateId,
    /// Payment terms.
    TermsId,
    /// A transaction. A transaction of any kind has its own.
    TransactionId,
    /// A person who works on the books in QuickBooks.
    UserId,
    /// A vendor type.
    VendorTypeId,
}

impl CurrencyId {
    /// The currency of a company that keeps its books in one. Its file does
    /// not say which currency that is.
    pub const COMPANY: Self = Self(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_id_is_the_number_that_the_file_stores() {
        assert_eq!(AccountId::new(7).number(), 7);
        assert_eq!(AccountId::new(7).to_string(), "7");
        assert_eq!(NameId::from_number(7), Some(NameId::new(7)));
        assert_eq!(NameId::from_number(i64::from(i32::MAX) + 1), None);
        assert!(LineId::new(1) < LineId::new(2));
    }
}

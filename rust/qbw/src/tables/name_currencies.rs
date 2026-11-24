//! The currencies of customers and vendors, as the ledger reads them.

use crate::ids::{CurrencyId, NameId};
#[cfg(test)]
use crate::tables::tests::Declared;
use crate::tables::{Table, stored_row};

pub(crate) const CUSTOMER_CURRENCIES: Table<CustomerCurrencyRow> = Table::new("abmc_customer_user");

pub(crate) const VENDOR_CURRENCIES: Table<VendorCurrencyRow> = Table::new("abmc_vendor_user");

stored_row! {
    /// The currency that a customer trades in.
    pub(crate) struct CustomerCurrencyRow {
        required {
            id: NameId = "customer_id",
        }
        optional {
            /// Nothing in a company of one currency.
            currency: Option<CurrencyId> = "currency_id",
        }
    }
}

stored_row! {
    /// The currency that a vendor trades in.
    pub(crate) struct VendorCurrencyRow {
        required {
            id: NameId = "vendor_id",
        }
        optional {
            /// Nothing in a company of one currency.
            currency: Option<CurrencyId> = "currency_id",
        }
    }
}

#[cfg(test)]
pub(super) fn declared() -> Vec<Box<dyn Declared>> {
    vec![Box::new(CUSTOMER_CURRENCIES), Box::new(VENDOR_CURRENCIES)]
}

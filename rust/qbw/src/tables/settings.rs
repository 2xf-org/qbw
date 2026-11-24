//! The company's settings.

#[cfg(test)]
use crate::tables::tests::Declared;
use crate::tables::{Table, stored_row};

pub(crate) const PREFERENCES: Table<PreferenceRow> = Table::new("cf_pref_attribute");

pub(crate) const PREFERENCE_VALUES: Table<PreferenceValueRow> =
    Table::new("cf_pref_attribute_value");

/// The column that holds a value, for an error about one.
pub(crate) const PREFERENCE_VALUE: &str = "preference_value";

stored_row! {
    /// A preference.
    pub(crate) struct PreferenceRow {
        required {
            id: i32 = "preference_id",
            name: String = "preference_name",
        }
        optional {}
    }
}

stored_row! {
    /// A value of a preference. A preference can have several.
    pub(crate) struct PreferenceValueRow {
        required {
            preference: i32 = "preference_id",
            /// Whose preference it is. Nothing for the whole company's.
            user: Option<String> = "user_name",
            value: Option<String> = "preference_value",
        }
        optional {}
    }
}

/// The names of the preferences that this crate reads.
pub(crate) mod preference {
    pub(crate) const ACCOUNT_NUMBERS: &str = "USEANUMBERS";
    pub(crate) const AGE_FROM_TRANSACTION_DATE: &str = "AGEFROMTXDATE";
    pub(crate) const AVAILABLE_LESS_BUILDS: &str = "INCLUDEBUILDSINQTYAVAIL";
    pub(crate) const AVAILABLE_LESS_SALES_ORDERS: &str = "M_AVAILISONHANDMINUSONSO";
    pub(crate) const BINS: &str = "ROW_SHELF_BIN";
    pub(crate) const CLASSES: &str = "USECLASSES";
    pub(crate) const CLOSING_DATE: &str = "CURRENT_CLOSING_DATE";
    pub(crate) const DEFAULT_TAX_ITEM: &str = "DEFAULTTAXITEM";
    pub(crate) const FISCAL_YEAR_START: &str = "FISCALBEGIN";
    pub(crate) const HOME_CURRENCY: &str = "HOMECURRENCY";
    pub(crate) const INVENTORY: &str = "INVENTORY";
    pub(crate) const MULTICURRENCY: &str = "USEMULTICURRENCY";
    pub(crate) const SALES_ORDERS: &str = "M_ENABLESALESORDER";
    pub(crate) const SALES_TAX: &str = "SALESTAXENABLED";
    pub(crate) const SERIAL_NUMBERS: &str = "SERIAL_NUMBER_TRACKING";
    pub(crate) const SITES: &str = "MULTI_SITE";
    pub(crate) const TAX_YEAR_START: &str = "TAXBEGIN";
}

#[cfg(test)]
pub(super) fn declared() -> Vec<Box<dyn Declared>> {
    vec![Box::new(PREFERENCES), Box::new(PREFERENCE_VALUES)]
}

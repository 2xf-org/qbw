//! What the file says about the company and itself, and who works on it.

use crate::ids::UserId;
#[cfg(test)]
use crate::tables::tests::Declared;
use crate::tables::{Table, stored_row};
use crate::unix_time::UnixTime;

/// The keys of the figures that QuickBooks works out of the company's years.
pub(crate) mod fact {
    pub(crate) const LAST_YEAR_NET_INCOME: &str = "QBF_NetIncomePnLAccrualInLastFiscalYear";
    pub(crate) const THIS_YEAR_NET_INCOME: &str = "QBF_NetProfitPnLAccrualInCurrentFiscalYear";
    pub(crate) const THIS_YEAR_INCOME: &str = "QBF_RevenuePnLAccrualInCurrentFiscalYear";
}

pub(crate) const FACTS: Table<FactRow> = Table::new("abmc_cf_facts");

pub(crate) const SAVES: Table<SaveRow> = Table::new("abmc_audit_header");

pub(crate) const SYSTEM_VALUES: Table<SystemValueRow> = Table::new("CF_SYSINFO");

stored_row! {
    /// A figure that QuickBooks worked out for itself.
    pub(crate) struct FactRow {
        required {
            key: Option<String> = "fact_key",
            value: Option<String> = "fact_value",
        }
        optional {}
    }
}

stored_row! {
    /// A time that a transaction was saved or deleted, as far as it says who
    /// did it.
    pub(crate) struct SaveRow {
        required {
            user: Option<UserId> = "mod_user_id",
            user_name: Option<String> = "mod_user_name",
        }
        optional {
            time: Option<UnixTime> = "modtimestamp_tms",
            /// Whoever deleted the transaction, and when.
            deleting_user: Option<UserId> = "delete_user_id",
            deleting_user_name: Option<String> = "delete_user_name",
            delete_time: Option<UnixTime> = "delete_tms",
        }
    }
}

stored_row! {
    /// Something that the file says about itself.
    pub(crate) struct SystemValueRow {
        required {
            name: String = "sysinfo_name",
            value: String = "sysinfo_value",
        }
        optional {}
    }
}

/// The names of the values that the file says about itself.
pub(crate) mod system_value {
    /// The release, counted from 1990.
    pub(crate) const RELEASE: &str = "DBMajorVersion";
    pub(crate) const UPDATE: &str = "DBReleaseNumber";
    pub(crate) const EDITION: &str = "DBFlavorStratum";
    pub(crate) const NEWEST_VERSION: &str = "NewestQBVersionUsed";
}

/// The names of the preferences that say who the company is.
pub(crate) mod company_preference {
    pub(crate) const ADDRESS_LINE: &str = "ADDR";
    pub(crate) const CREATED: &str = "CREATETIME";
    pub(crate) const EMAIL: &str = "COMPANYEMAIL";
    pub(crate) const FAX: &str = "COMPANYFAXNUM";
    pub(crate) const LEGAL_CITY: &str = "LEGALCITY";
    pub(crate) const LEGAL_COUNTRY: &str = "LEGALCOUNTRY";
    pub(crate) const LEGAL_NAME: &str = "LEGALNAME";
    pub(crate) const LEGAL_POSTAL_CODE: &str = "LEGALZIP";
    pub(crate) const LEGAL_STATE: &str = "LEGALSTATE";
    pub(crate) const LEGAL_STREET: [&str; 2] = ["LEGALADDR1", "LEGALADDR2"];
    pub(crate) const NAME: &str = "COMPANYNAME";
    pub(crate) const PHONE: &str = "COMPANYPHONENUM";
    pub(crate) const STARTED: &str = "STARTDATE";
    pub(crate) const WEBSITE: &str = "COMPANYWEBSITE";
}

#[cfg(test)]
pub(super) fn declared() -> Vec<Box<dyn Declared>> {
    vec![Box::new(FACTS), Box::new(SAVES), Box::new(SYSTEM_VALUES)]
}

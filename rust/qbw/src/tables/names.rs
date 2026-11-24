//! Customers and jobs, vendors, employees and other names.

use chrono::NaiveDate;
use sa17::Decimal;

use crate::codes::JobStatus;
use crate::ids::{
    AddressId, ClassId, CurrencyId, CustomerTypeId, ItemId, JobTypeId, NameId, OwnerKey,
    PaymentMethodId, PriceLevelId, SalesRepId, SalesTaxCodeId, TermsId, VendorTypeId,
};
#[cfg(test)]
use crate::tables::tests::Declared;
use crate::tables::{Table, stored_row};
use crate::unix_time::UnixTime;

pub(crate) const CUSTOMERS: Table<CustomerRow> = Table::new("abmc_customer_user");

pub(crate) const CUSTOMER_TOTALS: Table<CustomerTotalsRow> = Table::new("abmc_customer_internal");

pub(crate) const VENDORS: Table<VendorRow> = Table::new("abmc_vendor_user");

pub(crate) const VENDOR_TOTALS: Table<VendorTotalsRow> = Table::new("abmc_vendor_internal");

pub(crate) const EMPLOYEES: Table<EmployeeRow> = Table::new("abmc_employee");

pub(crate) const OTHER_NAMES: Table<OtherNameRow> = Table::new("abmc_other_user");

pub(crate) const OTHER_NAME_TOTALS: Table<OtherNameTotalsRow> = Table::new("abmc_other_internal");

stored_row! {
    /// A customer or a job as people keep it.
    pub(crate) struct CustomerRow {
        required {
            id: NameId = "customer_id",
            name: Option<String> = "name",
            /// The customer that a job is of.
            parent: Option<NameId> = "parent_id",
            is_hidden: bool = "is_hidden_bool",
            is_deleted: bool = "is_deleted_bool",
        }
        optional {
            customer_type: Option<CustomerTypeId> = "customer_type_id",
            terms: Option<TermsId> = "terms_id",
            sales_rep: Option<SalesRepId> = "sales_rep_id",
            price_level: Option<PriceLevelId> = "price_level_id",
            payment_method: Option<PaymentMethodId> = "payment_method_id",
            tax_code: Option<SalesTaxCodeId> = "sales_tax_code_id",
            tax_item: Option<ItemId> = "tax_item_id",
            /// Nothing in a company of one currency.
            currency: Option<CurrencyId> = "currency_id",
            credit_limit: Option<Decimal> = "credit_limit_amt",
            ship_to: Option<AddressId> = "ship_to_abm_id",
            job_status: Option<JobStatus> = "job_status_type",
            job_type: Option<JobTypeId> = "job_type_id",
            job_start: Option<NaiveDate> = "job_start_date",
            job_projected_end: Option<NaiveDate> = "job_projected_end_date",
            job_end: Option<NaiveDate> = "job_end_date",
            job_description: Option<String> = "job_description",
            class: Option<ClassId> = "class_id",
            resale_number: Option<String> = "resale_num",
            owner_key: Option<OwnerKey> = "customer_abm_id",
            created: Option<UnixTime> = "timestamp_tms",
            modified: Option<UnixTime> = "modtimestamp_tms",
        }
    }
}

stored_row! {
    /// What QuickBooks keeps about a customer or a job.
    pub(crate) struct CustomerTotalsRow {
        required {
            id: NameId = "customer_id",
        }
        optional {
            /// The id of the next name, whichever list it is in, with a minus
            /// sign on the first name.
            next: Option<i32> = "display_index_next",
            opened: Option<NaiveDate> = "open_date",
            balance: Option<Decimal> = "end_balance_amt",
            home_balance: Option<Decimal> = "end_balance_home_amt",
            list_id: Option<String> = "list_ident",
        }
    }
}

stored_row! {
    /// A vendor as people keep it.
    pub(crate) struct VendorRow {
        required {
            id: NameId = "vendor_id",
            name: Option<String> = "name",
            is_hidden: bool = "is_hidden_bool",
            is_deleted: bool = "is_deleted_bool",
        }
        optional {
            vendor_type: Option<VendorTypeId> = "vendor_type_id",
            terms: Option<TermsId> = "terms_id",
            /// Nothing in a company of one currency.
            currency: Option<CurrencyId> = "currency_id",
            credit_limit: Option<Decimal> = "credit_limit_amt",
            name_on_check: Option<String> = "name_on_check",
            is_1099: bool = "is_1099_vendor_bool",
            owner_key: Option<OwnerKey> = "vendor_abm_id",
            created: Option<UnixTime> = "timestamp_tms",
            modified: Option<UnixTime> = "modtimestamp_tms",
        }
    }
}

stored_row! {
    /// What QuickBooks keeps about a vendor.
    pub(crate) struct VendorTotalsRow {
        required {
            id: NameId = "vendor_id",
        }
        optional {
            next: Option<i32> = "display_index_next",
            opened: Option<NaiveDate> = "open_date",
            balance: Option<Decimal> = "end_balance_amt",
            home_balance: Option<Decimal> = "end_balance_home_amt",
            list_id: Option<String> = "list_ident",
        }
    }
}

stored_row! {
    /// An employee, as far as a name goes. Pay is not read.
    pub(crate) struct EmployeeRow {
        required {
            id: NameId = "employee_id",
            name: Option<String> = "name",
            is_hidden: bool = "is_hidden_bool",
            is_deleted: bool = "is_deleted_bool",
        }
        optional {
            next: Option<i32> = "display_index_next",
            hired: Option<NaiveDate> = "open_date",
            name_on_check: Option<String> = "name_on_check",
            owner_key: Option<OwnerKey> = "employee_abm_id",
            list_id: Option<String> = "list_ident",
            created: Option<UnixTime> = "timestamp_tms",
            modified: Option<UnixTime> = "modtimestamp_tms",
        }
    }
}

stored_row! {
    /// Another name as people keep it.
    pub(crate) struct OtherNameRow {
        required {
            id: NameId = "other_id",
            name: Option<String> = "name",
            is_hidden: bool = "is_hidden_bool",
            is_deleted: bool = "is_deleted_bool",
        }
        optional {
            owner_key: Option<OwnerKey> = "other_abm_id",
            created: Option<UnixTime> = "timestamp_tms",
            modified: Option<UnixTime> = "modtimestamp_tms",
        }
    }
}

stored_row! {
    /// What QuickBooks keeps about another name.
    pub(crate) struct OtherNameTotalsRow {
        required {
            id: NameId = "other_id",
        }
        optional {
            next: Option<i32> = "display_index_next",
            opened: Option<NaiveDate> = "open_date",
            list_id: Option<String> = "list_ident",
        }
    }
}

#[cfg(test)]
pub(super) fn declared() -> Vec<Box<dyn Declared>> {
    vec![
        Box::new(CUSTOMERS),
        Box::new(CUSTOMER_TOTALS),
        Box::new(VENDORS),
        Box::new(VENDOR_TOTALS),
        Box::new(EMPLOYEES),
        Box::new(OTHER_NAMES),
        Box::new(OTHER_NAME_TOTALS),
    ]
}

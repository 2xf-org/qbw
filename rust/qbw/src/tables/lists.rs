//! The small lists.

use chrono::NaiveDate;
use sa17::Decimal;

use crate::codes::{PaymentMethodKind, PriceLevelKind, TemplateForm, TermsKind};
use crate::ids::{
    CategoryId, ClassId, CurrencyId, CustomerTypeId, ItemId, JobTypeId, MemorizedId, MessageId,
    NameId, PaymentMethodId, PriceLevelId, SalesRepId, SalesTaxCodeId, ShipMethodId, SiteId,
    TemplateId, TermsId, TransactionId, VendorTypeId,
};
#[cfg(test)]
use crate::tables::tests::Declared;
use crate::tables::{Table, stored_row};

pub(crate) const CATEGORIES: Table<CategoryRow> = Table::new("abmc_categories");

pub(crate) const CLASSES: Table<ClassRow> = Table::new("abmc_class");

pub(crate) const CURRENCIES: Table<CurrencyRow> = Table::new("abmc_currency");

pub(crate) const CUSTOMER_MESSAGES: Table<CustomerMessageRow> = Table::new("abmc_customer_message");

pub(crate) const CUSTOMER_TYPES: Table<CustomerTypeRow> = Table::new("abmc_customer_type");

pub(crate) const EXCHANGE_RATES: Table<ExchangeRateRow> = Table::new("abmc_exchange_rate_history");

pub(crate) const JOB_TYPES: Table<JobTypeRow> = Table::new("abmc_job_type");

pub(crate) const MEMORIZED: Table<MemorizedRow> = Table::new("abmc_memorized_transactions");

pub(crate) const PAYMENT_METHODS: Table<PaymentMethodRow> = Table::new("abmc_payment_method");

pub(crate) const PRICE_LEVELS: Table<PriceLevelRow> = Table::new("abmc_price_level");

pub(crate) const PRICE_LEVEL_ITEMS: Table<PriceLevelItemRow> = Table::new("abmc_price_level_item");

pub(crate) const SALES_REPS: Table<SalesRepRow> = Table::new("abmc_sales_rep");

pub(crate) const SALES_TAX_CODES: Table<SalesTaxCodeRow> = Table::new("abmc_sales_tax_code");

pub(crate) const SHIP_METHODS: Table<ShipMethodRow> = Table::new("abmc_shipping");

pub(crate) const SITES: Table<SiteRow> = Table::new("abmc_site");

pub(crate) const TEMPLATES: Table<TemplateRow> = Table::new("abmc_custom_style");

pub(crate) const TERMS: Table<TermsRow> = Table::new("abmc_terms");

pub(crate) const VENDOR_TYPES: Table<VendorTypeRow> = Table::new("abmc_vendor_type");

stored_row! {
    /// A category of items.
    pub(crate) struct CategoryRow {
        required {
            id: CategoryId = "category_id",
            name: Option<String> = "category_name",
        }
        optional {
            parent: Option<CategoryId> = "parent_id",
            next: Option<i32> = "display_index_next",
            is_hidden: bool = "is_hidden_bool",
            is_deleted: bool = "is_deleted_bool",
        }
    }
}

stored_row! {
    /// A class.
    pub(crate) struct ClassRow {
        required {
            id: ClassId = "class_id",
            name: Option<String> = "name",
        }
        optional {
            parent: Option<ClassId> = "parent_id",
            next: Option<i32> = "display_index_next",
            is_hidden: bool = "is_hidden_bool",
            is_deleted: bool = "is_deleted_bool",
        }
    }
}

stored_row! {
    /// A type of customers.
    pub(crate) struct CustomerTypeRow {
        required {
            id: CustomerTypeId = "customer_type_id",
            name: Option<String> = "name",
        }
        optional {
            parent: Option<CustomerTypeId> = "parent_id",
            next: Option<i32> = "display_index_next",
            is_hidden: bool = "is_hidden_bool",
            is_deleted: bool = "is_deleted_bool",
        }
    }
}

stored_row! {
    /// A type of vendors.
    pub(crate) struct VendorTypeRow {
        required {
            id: VendorTypeId = "vendor_type_id",
            name: Option<String> = "name",
        }
        optional {
            parent: Option<VendorTypeId> = "parent_id",
            next: Option<i32> = "display_index_next",
            is_hidden: bool = "is_hidden_bool",
            is_deleted: bool = "is_deleted_bool",
        }
    }
}

stored_row! {
    /// A type of jobs.
    pub(crate) struct JobTypeRow {
        required {
            id: JobTypeId = "job_type_id",
            name: Option<String> = "name",
        }
        optional {
            parent: Option<JobTypeId> = "parent_id",
            next: Option<i32> = "display_index_next",
            is_hidden: bool = "is_hidden_bool",
            is_deleted: bool = "is_deleted_bool",
        }
    }
}

stored_row! {
    /// A site that stock is kept at, or a bin of one.
    pub(crate) struct SiteRow {
        required {
            id: SiteId = "site_id",
            name: Option<String> = "name",
        }
        optional {
            /// The site that a bin is at.
            parent: Option<SiteId> = "parent_id",
            special: Option<i32> = "special_type",
            description: Option<String> = "description",
            next: Option<i32> = "display_index_next",
            is_hidden: bool = "is_hidden_bool",
            is_deleted: bool = "is_deleted_bool",
        }
    }
}

stored_row! {
    /// A memorized transaction, or a group of them.
    pub(crate) struct MemorizedRow {
        required {
            id: MemorizedId = "memorized_transactions_id",
            name: Option<String> = "name",
            /// The transaction that is made again.
            transaction: Option<TransactionId> = "master_record_id",
        }
        optional {
            /// The group that it is in.
            parent: Option<MemorizedId> = "parent_id",
            next: Option<i32> = "display_index_next",
            is_hidden: bool = "is_hidden_bool",
            is_deleted: bool = "is_deleted_bool",
        }
    }
}

stored_row! {
    /// Payment terms. Which of the columns count depends on the kind.
    pub(crate) struct TermsRow {
        required {
            id: TermsId = "terms_id",
            name: Option<String> = "name",
        }
        optional {
            kind: Option<TermsKind> = "terms_type",
            due_days: Option<i32> = "standard_due_days",
            discount_days: Option<i32> = "standard_discount_days",
            discount_percent: Option<Decimal> = "discount_pct",
            due_day: Option<i32> = "day_of_month_due",
            /// The file's own spelling.
            next_month_days: Option<i32> = "date_mininmum_days",
            discount_day: Option<i32> = "discount_day_of_month",
            is_hidden: bool = "is_hidden_bool",
            is_deleted: bool = "is_deleted_bool",
        }
    }
}

stored_row! {
    /// A way of paying.
    pub(crate) struct PaymentMethodRow {
        required {
            id: PaymentMethodId = "payment_method_id",
            name: Option<String> = "name",
        }
        optional {
            kind: Option<PaymentMethodKind> = "payment_method_type",
            is_hidden: bool = "is_hidden_bool",
            is_deleted: bool = "is_deleted_bool",
        }
    }
}

stored_row! {
    /// A way of shipping.
    pub(crate) struct ShipMethodRow {
        required {
            id: ShipMethodId = "shipping_id",
            name: Option<String> = "name",
        }
        optional {
            is_hidden: bool = "is_hidden_bool",
            is_deleted: bool = "is_deleted_bool",
        }
    }
}

stored_row! {
    /// Someone who makes sales.
    pub(crate) struct SalesRepRow {
        required {
            id: SalesRepId = "sales_rep_id",
            person: Option<NameId> = "name_id",
            initials: Option<String> = "initials",
        }
        optional {
            is_hidden: bool = "is_hidden_bool",
            is_deleted: bool = "is_deleted_bool",
        }
    }
}

stored_row! {
    /// Whether what it is on is taxed.
    pub(crate) struct SalesTaxCodeRow {
        required {
            id: SalesTaxCodeId = "sales_tax_code_id",
            code: Option<String> = "code",
            is_taxable: bool = "is_taxable_bool",
        }
        optional {
            description: Option<String> = "description",
            is_hidden: bool = "is_hidden_bool",
            is_deleted: bool = "is_deleted_bool",
        }
    }
}

stored_row! {
    /// A set of prices for some customers.
    pub(crate) struct PriceLevelRow {
        required {
            id: PriceLevelId = "price_level_id",
            name: Option<String> = "name",
        }
        optional {
            kind: Option<PriceLevelKind> = "price_level_type",
            percent: Option<Decimal> = "value_pct",
            /// Nothing in a company of one currency.
            currency: Option<CurrencyId> = "currency_id",
            is_hidden: bool = "is_hidden_bool",
            is_deleted: bool = "is_deleted_bool",
        }
    }
}

stored_row! {
    /// The price of one item at a price level.
    pub(crate) struct PriceLevelItemRow {
        required {
            level: PriceLevelId = "price_level_id",
            item: ItemId = "item_id",
        }
        optional {
            price: Option<Decimal> = "computed_price_amt",
            percent: Option<Decimal> = "computed_price_pct",
        }
    }
}

stored_row! {
    /// A currency.
    pub(crate) struct CurrencyRow {
        required {
            id: CurrencyId = "currency_id",
            name: Option<String> = "name",
            code: Option<String> = "code",
        }
        optional {
            /// The bytes are UTF-8, whatever the encoding of the file.
            symbol: Option<String> = "symbol",
            decimal_places: Option<i32> = "decimal_places",
            is_hidden: bool = "is_hidden_bool",
            is_deleted: bool = "is_deleted_bool",
        }
    }
}

stored_row! {
    /// What a currency was worth from a day on.
    pub(crate) struct ExchangeRateRow {
        required {
            /// Nothing on a deleted rate, as are the next two.
            currency: Option<CurrencyId> = "currency_id",
            as_of: Option<NaiveDate> = "as_of_date",
            rate: Option<f32> = "exchange_rate",
        }
        optional {
            is_deleted: bool = "is_deleted_bool",
        }
    }
}

stored_row! {
    /// A message for the forms that customers get.
    pub(crate) struct CustomerMessageRow {
        required {
            id: MessageId = "customer_message_id",
            text: Option<String> = "cust_message",
        }
        optional {
            is_hidden: bool = "is_hidden_bool",
            is_deleted: bool = "is_deleted_bool",
        }
    }
}

stored_row! {
    /// What forms are printed with.
    pub(crate) struct TemplateRow {
        required {
            id: TemplateId = "custom_style_id",
            name: Option<String> = "name",
            form: TemplateForm = "style_view_type",
        }
        optional {
            is_hidden: bool = "is_hidden_bool",
            is_deleted: bool = "is_deleted_bool",
        }
    }
}

#[cfg(test)]
pub(super) fn declared() -> Vec<Box<dyn Declared>> {
    vec![
        Box::new(CATEGORIES),
        Box::new(CLASSES),
        Box::new(CURRENCIES),
        Box::new(CUSTOMER_MESSAGES),
        Box::new(CUSTOMER_TYPES),
        Box::new(EXCHANGE_RATES),
        Box::new(JOB_TYPES),
        Box::new(MEMORIZED),
        Box::new(PAYMENT_METHODS),
        Box::new(PRICE_LEVELS),
        Box::new(PRICE_LEVEL_ITEMS),
        Box::new(SALES_REPS),
        Box::new(SALES_TAX_CODES),
        Box::new(SHIP_METHODS),
        Box::new(SITES),
        Box::new(TEMPLATES),
        Box::new(TERMS),
        Box::new(VENDOR_TYPES),
    ]
}

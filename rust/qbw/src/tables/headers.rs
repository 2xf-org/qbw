//! The headers of transactions.
//!
//! The headers of every kind share the columns of [`HeaderRow`]. What a kind
//! has besides is in a row of its own, which is read from the same table.

use chrono::NaiveDate;

use crate::codes::{LastAction, TransactionKind};
use crate::ids::{
    AddressId, ContactId, CurrencyId, ItemId, LineId, MessageId, NameId, OwnerKey, PaymentMethodId,
    SalesRepId, SalesTaxCodeId, ShipMethodId, SiteId, TemplateId, TermsId, TransactionId, UserId,
};
use crate::party::FourNames;
use crate::posting_state::PostingFlags;
#[cfg(test)]
use crate::tables::Form;
use crate::tables::stored_row;
#[cfg(test)]
use crate::tables::tests::Declared;
use crate::unix_time::UnixTime;

stored_row! {
    /// What finding a transaction takes.
    pub(crate) struct HeaderKeyRow {
        required {
            id: TransactionId = "transaction_id",
            own_line: LineId = "target_id",
            date: NaiveDate = "transaction_date",
            number: Option<String> = "doc_num",
            last_action: LastAction = "last_auditable_action",
        }
        optional {
            /// Not to be read where the table decides the kind.
            kind: Option<TransactionKind> = "transaction_view_type",
            /// One of the four names is there at most.
            customer: Option<NameId> = "customer_id",
            vendor: Option<NameId> = "vendor_id",
            employee: Option<NameId> = "employee_id",
            other_name: Option<NameId> = "other_id",
            is_non_posting: bool = "is_no_post_bool",
            is_memorized: bool = "is_memorized_transaction_bool",
            /// On a payment, whether it moved no money. Other kinds have the
            /// flag, too, and it is not to be read there.
            is_without_money: bool = "is_arap_bool",
        }
    }
}

impl HeaderKeyRow {
    /// What the header of a transaction of `kind` says of whether it counts.
    pub(crate) fn flags(&self, kind: TransactionKind) -> PostingFlags {
        PostingFlags {
            kind,
            is_memorized: self.is_memorized,
            is_non_posting: self.is_non_posting,
        }
    }

    pub(crate) fn names(&self) -> FourNames {
        FourNames {
            customer: self.customer,
            vendor: self.vendor,
            employee: self.employee,
            other_name: self.other_name,
        }
    }
}

stored_row! {
    /// What a transaction of any kind has.
    pub(crate) struct HeaderRow {
        required {
            memo: Option<String> = "memo",
            /// Nothing in a company of one currency.
            currency: Option<CurrencyId> = "currency_id",
            exchange_rate: f32 = "exchange_rate",
            last_action: LastAction = "last_auditable_action",
            edited_by: Option<UserId> = "last_edit_by",
            sequence: Option<i32> = "trans_num",
            owner_key: Option<OwnerKey> = "abm_id",
            created: Option<UnixTime> = "timestamp_tms",
            modified: Option<UnixTime> = "modtimestamp_tms",
        }
        optional {
            billing: Option<ContactId> = "contact_id",
        }
    }
}

stored_row! {
    /// What a form has that customers get.
    pub(crate) struct SaleRow {
        required {}
        optional {
            terms: Option<TermsId> = "terms_id",
            ship_to: Option<AddressId> = "ship_to_id",
            ship_date: Option<NaiveDate> = "ship_date",
            ship_via: Option<ShipMethodId> = "ship_via_id",
            sales_rep: Option<SalesRepId> = "sales_rep_id",
            po_number: Option<String> = "po_num",
            fob: Option<String> = "fob",
            other: Option<String> = "other",
            message: Option<MessageId> = "customer_message_id",
            tax_code: Option<SalesTaxCodeId> = "sales_tax_code_id",
            template: Option<TemplateId> = "custom_style_id",
            payment_method: Option<PaymentMethodId> = "payment_method_id",
            sales_order_number: Option<String> = "sales_order_num",
            is_finance_charge: bool = "is_finance_charge_bool",
            is_to_be_printed: bool = "is_to_be_printed_bool",
            is_to_be_sent: bool = "is_to_be_sent_bool",
        }
    }
}

stored_row! {
    /// What a bill has.
    pub(crate) struct BillRow {
        required {}
        optional {
            terms: Option<TermsId> = "terms_id",
        }
    }
}

stored_row! {
    /// What a purchase order has.
    pub(crate) struct PurchaseOrderRow {
        required {}
        optional {
            terms: Option<TermsId> = "terms_id",
            /// The day the goods are expected. The column that is called so is
            /// empty.
            expected: Option<NaiveDate> = "ship_date",
            ship_to: Option<AddressId> = "ship_to_id",
            ship_to_name: Option<NameId> = "ship_to_name_id",
            ship_via: Option<ShipMethodId> = "ship_via_id",
            fob: Option<String> = "fob",
            other: Option<String> = "other",
            vendor_message: Option<String> = "vendor_message",
            template: Option<TemplateId> = "custom_style_id",
            site: Option<SiteId> = "site_id",
            is_to_be_printed: bool = "is_to_be_printed_bool",
            is_to_be_sent: bool = "is_to_be_sent_bool",
        }
    }
}

stored_row! {
    /// What money received from a customer, or given back to one, has.
    pub(crate) struct ReceiptRow {
        required {}
        optional {
            payment_method: Option<PaymentMethodId> = "payment_method_id",
            is_deposited: bool = "is_deposited_bool",
            is_bounced: bool = "is_bounced_bool",
        }
    }
}

stored_row! {
    /// What a check has.
    pub(crate) struct CheckRow {
        required {}
        optional {
            is_to_be_printed: bool = "is_to_be_printed_bool",
        }
    }
}

stored_row! {
    /// What a journal entry has.
    pub(crate) struct JournalRow {
        required {}
        optional {
            is_home_currency_adjustment: bool = "is_home_currency_adjustment_bool",
            is_shown_in_home_currency: bool = "is_shown_in_home_currency_bool",
        }
    }
}

stored_row! {
    /// What a change of the stock has.
    pub(crate) struct AdjustmentRow {
        required {}
        optional {
            site: Option<SiteId> = "site_id",
        }
    }
}

stored_row! {
    /// What a build of an assembly has.
    pub(crate) struct BuildRow {
        required {}
        optional {
            item: Option<ItemId> = "item_id",
        }
    }
}

stored_row! {
    /// The memo of a transaction.
    pub(crate) struct MemoRow {
        required {
            memo: Option<String> = "memo",
        }
        optional {}
    }
}

#[cfg(test)]
pub(super) fn declared() -> Vec<Box<dyn Declared>> {
    let mut declared: Vec<Box<dyn Declared>> = Vec::new();
    for form in Form::all() {
        declared.push(Box::new(form.headers::<HeaderKeyRow>()));
        declared.push(Box::new(form.headers::<HeaderRow>()));
        declared.push(Box::new(form.headers::<SaleRow>()));
        declared.push(Box::new(form.headers::<BillRow>()));
        declared.push(Box::new(form.headers::<PurchaseOrderRow>()));
        declared.push(Box::new(form.headers::<ReceiptRow>()));
        declared.push(Box::new(form.headers::<CheckRow>()));
        declared.push(Box::new(form.headers::<JournalRow>()));
        declared.push(Box::new(form.headers::<AdjustmentRow>()));
        declared.push(Box::new(form.headers::<BuildRow>()));
        declared.push(Box::new(form.headers::<MemoRow>()));
    }
    declared
}

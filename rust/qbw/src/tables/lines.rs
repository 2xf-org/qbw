//! The lines of transactions, as their forms show them.
//!
//! The lines of every kind have one shape. What a kind has no use for, its
//! table lacks.

use chrono::NaiveDate;
use sa17::Decimal;

use crate::amount::Amount;
use crate::codes::{Cleared, NameColumn};
use crate::ids::{
    AccountId, ClassId, CurrencyId, ItemId, LineId, NameId, OwnerKey, PaymentMethodId,
    SalesTaxCodeId, SiteId, TransactionId,
};
use crate::party::FourNames;
#[cfg(test)]
use crate::tables::Form;
use crate::tables::stored_row;
#[cfg(test)]
use crate::tables::tests::Declared;

stored_row! {
    /// Which transaction a line is of.
    pub(crate) struct LineKeyRow {
        required {
            transaction: TransactionId = "transaction_id",
        }
        optional {}
    }
}

stored_row! {
    /// What a list of transactions shows of a line: of an own line all of it, and
    /// of the others what is left.
    pub(crate) struct SummaryLineRow {
        required {
            id: LineId = "target_id",
            transaction: TransactionId = "transaction_id",
            is_own: bool = "is_source_bool",
            account: Option<AccountId> = "account_id",
            currency: Option<CurrencyId> = "currency_id",
            home_amount: Option<Amount> = "amount_amt",
            foreign_amount: Option<Amount> = "foreign_amount_amt",
        }
        optional {
            is_paid: bool = "is_paid_bool",
            open_balance: Option<Amount> = "open_balance_amt",
            due_date: Option<NaiveDate> = "due_date",
        }
    }
}

stored_row! {
    /// What a list of transactions shows of a line of an order or of a transfer.
    /// The other forms have many more lines.
    pub(crate) struct OrderOrTransferRow {
        required {
            id: LineId = "target_id",
            transaction: TransactionId = "transaction_id",
            is_own: bool = "is_source_bool",
            account: Option<AccountId> = "account_id",
            other_account: Option<AccountId> = "sibling_account_id",
            currency: Option<CurrencyId> = "currency_id",
            home_amount: Option<Amount> = "amount_amt",
            foreign_amount: Option<Amount> = "foreign_amount_amt",
        }
        optional {
            is_paid: bool = "is_paid_bool",
            open_balance: Option<Amount> = "open_balance_amt",
            due_date: Option<NaiveDate> = "due_date",
            is_received: bool = "is_received_bool",
            is_invoiced: bool = "is_invoiced_bool",
            is_closed: bool = "is_manually_closed_bool",
        }
    }
}

stored_row! {
    /// A line of a transaction. A debit is positive.
    pub(crate) struct LineRow {
        required {
            id: LineId = "target_id",
            transaction: TransactionId = "transaction_id",
            /// Whether it is the transaction's own side, such as the bank's.
            is_own: bool = "is_source_bool",
            /// The next line of the transaction. Nothing on the last.
            next: Option<LineId> = "next_target_id",
            account: Option<AccountId> = "account_id",
            /// The account on the other side: the own line's, or on the own
            /// line the first of the others.
            other_account: Option<AccountId> = "sibling_account_id",
            memo: Option<String> = "memo",
            number: Option<String> = "doc_num",
            /// Nothing on the lines of a company of one currency.
            currency: Option<CurrencyId> = "currency_id",
            home_amount: Option<Amount> = "amount_amt",
            /// The amount in the transaction's currency, when that is another
            /// one. Nothing on an exchange difference.
            foreign_amount: Option<Amount> = "foreign_amount_amt",
            is_exchange_difference: bool = "is_gain_loss_target_bool",
            owner_key: Option<OwnerKey> = "abm_id",
        }
        optional {
            /// Whether the own line has more than one line on its other side.
            is_split: bool = "is_split_bool",
            item: Option<ItemId> = "item_id",
            quantity: Option<Amount> = "quantity_qnty",
            /// Which of the four names is the line's own.
            name_column: Option<NameColumn> = "lineitem_name_type",
            customer: Option<NameId> = "customer_id",
            vendor: Option<NameId> = "vendor_id",
            employee: Option<NameId> = "employee_id",
            other_name: Option<NameId> = "other_id",
            class: Option<ClassId> = "class_id",
            /// A price of two digits after the point.
            price: Option<Decimal> = "unit_price_amt",
            /// A price of more.
            exact_price: Option<Decimal> = "unit_price_qnty",
            price_percent: Option<Decimal> = "unit_price_pct",
            /// The price with tax in it, which is the only one on a few lines.
            gross_price: Option<Decimal> = "gross_unit_price_amt",
            exact_gross_price: Option<Decimal> = "gross_unit_price_qnty",
            markup: Option<Decimal> = "markup_amt",
            markup_percent: Option<Decimal> = "markup_amt_pct",
            tax_code: Option<SalesTaxCodeId> = "sales_tax_code_id",
            is_tax: bool = "is_trans_tax_bool",
            is_group_end: bool = "is_groupend_row_bool",
            /// Whether it is the receivable's or the payable's side of a
            /// discount.
            is_discount: bool = "is_discount_bool",
            /// Whether it is the other side of a discount.
            is_discount_account: bool = "is_applied_discount_bool",
            /// Set on some lines without an account, too.
            is_cash_back: bool = "is_less_cashback_bool",
            is_paid: bool = "is_paid_bool",
            /// What is left to pay, in the transaction's currency. Nothing
            /// when it is the whole amount.
            open_balance: Option<Amount> = "open_balance_amt",
            due_date: Option<NaiveDate> = "due_date",
            /// What is left of an order's quantity. Nothing when it is the
            /// whole quantity.
            open_quantity: Option<Amount> = "open_qnty",
            is_received: bool = "is_received_bool",
            is_invoiced: bool = "is_invoiced_bool",
            is_closed: bool = "is_manually_closed_bool",
            cleared: Option<Cleared> = "cleared_type",
            /// The date of the statement that a line was reconciled with, on
            /// some of the reconciled lines.
            reconciled_on: Option<NaiveDate> = "reconcile_date",
            service_date: Option<NaiveDate> = "service_date",
            payment_method: Option<PaymentMethodId> = "payment_method_id",
            site: Option<SiteId> = "site_id",
            bin: Option<SiteId> = "rsb_id",
            part_number: Option<String> = "manufacturer_part_num",
            other1: Option<String> = "other1",
            other2: Option<String> = "other2",
            /// On the own line of an estimate.
            is_inactive: bool = "is_inactive_bool",
            /// On the lines of a journal entry, the same on each.
            is_adjustment: bool = "is_adjustment_bool",
            reimbursed_code: Option<i32> = "reimbursed_type",
        }
    }
}

impl LineRow {
    pub(crate) fn names(&self) -> FourNames {
        FourNames {
            customer: self.customer,
            vendor: self.vendor,
            employee: self.employee,
            other_name: self.other_name,
        }
    }
}

impl From<&LineRow> for SummaryLineRow {
    fn from(row: &LineRow) -> Self {
        Self {
            id: row.id,
            transaction: row.transaction,
            is_own: row.is_own,
            account: row.account,
            currency: row.currency,
            home_amount: row.home_amount,
            foreign_amount: row.foreign_amount,
            is_paid: row.is_paid,
            open_balance: row.open_balance,
            due_date: row.due_date,
        }
    }
}

/// What a list of transactions shows of the own line of an order or a transfer,
/// besides what it shows of all.
#[derive(Clone, Copy, Debug)]
pub(crate) struct OrderOrTransfer {
    pub(crate) other_account: Option<AccountId>,
    pub(crate) is_received: bool,
    pub(crate) is_invoiced: bool,
    pub(crate) is_closed: bool,
}

impl From<&LineRow> for OrderOrTransfer {
    fn from(row: &LineRow) -> Self {
        Self {
            other_account: row.other_account,
            is_received: row.is_received,
            is_invoiced: row.is_invoiced,
            is_closed: row.is_closed,
        }
    }
}

impl From<&OrderOrTransferRow> for OrderOrTransfer {
    fn from(row: &OrderOrTransferRow) -> Self {
        Self {
            other_account: row.other_account,
            is_received: row.is_received,
            is_invoiced: row.is_invoiced,
            is_closed: row.is_closed,
        }
    }
}

impl From<&OrderOrTransferRow> for SummaryLineRow {
    fn from(row: &OrderOrTransferRow) -> Self {
        Self {
            id: row.id,
            transaction: row.transaction,
            is_own: row.is_own,
            account: row.account,
            currency: row.currency,
            home_amount: row.home_amount,
            foreign_amount: row.foreign_amount,
            is_paid: row.is_paid,
            open_balance: row.open_balance,
            due_date: row.due_date,
        }
    }
}

#[cfg(test)]
pub(super) fn declared() -> Vec<Box<dyn Declared>> {
    let mut declared: Vec<Box<dyn Declared>> = Vec::new();
    for form in Form::all() {
        declared.push(Box::new(form.lines::<LineKeyRow>()));
        declared.push(Box::new(form.lines::<SummaryLineRow>()));
        declared.push(Box::new(form.lines::<OrderOrTransferRow>()));
        declared.push(Box::new(form.lines::<LineRow>()));
    }
    declared
}

//! The lines of transactions, as the ledger reads them.

use chrono::NaiveDate;

use crate::amount::Amount;
use crate::codes::{Cleared, InventorySide, NameColumn, TransactionKind};
use crate::ids::{AccountId, ItemId, LineId, NameId, TransactionId};
use crate::party::FourNames;
use crate::posting_state::PostingFlags;
#[cfg(test)]
use crate::tables::Form;
#[cfg(test)]
use crate::tables::tests::Declared;
use crate::tables::{Table, stored_row};

/// What the register shows of a line of the table called `lines`.
pub(crate) const fn register_lines(lines: &'static str) -> Table<RegisterLineRow> {
    Table::new(lines)
}

stored_row! {
    /// A line of a transaction, with what it posts. A debit is positive.
    ///
    /// The lines of every kind of transaction have this shape. What a kind has
    /// no use for, its table lacks.
    pub(crate) struct PostingLineRow {
        required {
            id: LineId = "target_id",
            transaction: TransactionId = "transaction_id",
            date: NaiveDate = "transaction_date",
            kind: TransactionKind = "transaction_view_type",
            /// Nothing on a line that only says something.
            account: Option<AccountId> = "account_id",
            /// Whether it is the transaction's own side, such as the bank's.
            is_own: bool = "is_source_bool",
            /// The account on the other side: the own line's, or on the own
            /// line the first of the others.
            other_account: Option<AccountId> = "sibling_account_id",
            /// The next line of the transaction. Nothing on the last.
            next: Option<LineId> = "next_target_id",
            home_amount: Option<Amount> = "amount_amt",
            /// The amount in the other currency, for a transaction or an
            /// account that has one. Nothing on an exchange difference.
            foreign_amount: Option<Amount> = "foreign_amount_amt",
        }
        optional {
            /// Whether the own line has more than one line on its other side.
            is_split: bool = "is_split_bool",
            is_non_posting: bool = "is_no_post_bool",
            is_memorized: bool = "is_memorized_transaction_bool",
            item: Option<ItemId> = "item_id",
            name_column: Option<NameColumn> = "lineitem_name_type",
            customer: Option<NameId> = "customer_id",
            vendor: Option<NameId> = "vendor_id",
            employee: Option<NameId> = "employee_id",
            other_name: Option<NameId> = "other_id",
            cleared: Option<Cleared> = "cleared_type",
            due_date: Option<NaiveDate> = "due_date",
            /// Whether nothing is left of it, where something can be.
            is_paid: bool = "is_paid_bool",
            /// What is left of it, with its sign. Nothing where all of it is.
            open_balance: Option<Amount> = "open_balance_amt",
            /// What one of the transaction's currency is in the company's.
            exchange_rate: Option<f32> = "exchange_rate",
        }
    }
}

impl PostingLineRow {
    /// What the line of a transaction of `kind` says of whether that counts.
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
    /// What a line of an item in stock posts besides: a change of the stock's
    /// value, or of what the goods sold had cost.
    pub(crate) struct InventoryLineRow {
        required {
            /// The line it belongs to.
            line: LineId = "target_id",
            side: InventorySide = "target_type",
            account: Option<AccountId> = "account_id",
            home_amount: Option<Amount> = "amount_amt",
        }
        optional {
            foreign_amount: Option<Amount> = "foreign_amount_amt",
        }
    }
}

stored_row! {
    /// What a register shows of a line besides what it posts.
    pub(crate) struct RegisterLineRow {
        required {}
        optional {
            memo: Option<String> = "memo",
            /// The date of the statement that a line was reconciled with, on
            /// some of the reconciled lines.
            reconciled_on: Option<NaiveDate> = "reconcile_date",
        }
    }
}

#[cfg(test)]
pub(super) fn declared() -> Vec<Box<dyn Declared>> {
    let mut declared: Vec<Box<dyn Declared>> = Vec::new();
    for form in Form::all() {
        declared.push(Box::new(form.lines::<PostingLineRow>()));
        declared.push(Box::new(form.lines::<RegisterLineRow>()));
        if let Some(lines) = form.stock_lines() {
            declared.push(Box::new(lines));
        }
    }
    declared
}

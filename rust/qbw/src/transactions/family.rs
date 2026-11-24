//! The families of kinds.

use crate::codes::TransactionKind;

/// The kinds whose transactions have the same fields of their own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Family {
    Sale,
    Payment,
    Refund,
    Bill,
    PurchaseOrder,
    BillPayment,
    Check,
    CardCharge,
    Deposit,
    Transfer,
    Journal,
    InventoryAdjustment,
    InventoryTransfer,
    Build,
    Other,
}

impl Family {
    pub(super) const fn of(kind: TransactionKind) -> Self {
        match kind {
            TransactionKind::Invoice
            | TransactionKind::SalesReceipt
            | TransactionKind::CreditMemo
            | TransactionKind::Estimate
            | TransactionKind::SalesOrder
            | TransactionKind::StatementCharge => Self::Sale,
            TransactionKind::Payment => Self::Payment,
            TransactionKind::CustomerRefund => Self::Refund,
            TransactionKind::Bill
            | TransactionKind::VendorCredit
            | TransactionKind::ItemReceipt => Self::Bill,
            TransactionKind::PurchaseOrder => Self::PurchaseOrder,
            TransactionKind::BillPaymentCheck | TransactionKind::BillPaymentCreditCard => {
                Self::BillPayment
            }
            TransactionKind::Check
            | TransactionKind::SalesTaxPayment
            | TransactionKind::Paycheck
            | TransactionKind::PayrollLiabilityCheck => Self::Check,
            TransactionKind::CreditCardCharge | TransactionKind::CreditCardCredit => {
                Self::CardCharge
            }
            TransactionKind::Deposit => Self::Deposit,
            TransactionKind::Transfer => Self::Transfer,
            TransactionKind::JournalEntry => Self::Journal,
            TransactionKind::InventoryAdjustment => Self::InventoryAdjustment,
            TransactionKind::InventoryTransfer => Self::InventoryTransfer,
            TransactionKind::AssemblyBuild => Self::Build,
            _ => Self::Other,
        }
    }
}

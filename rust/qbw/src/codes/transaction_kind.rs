//! The kinds of transactions.

use crate::code::{codes, words};

codes! {
    /// The kind of a transaction.
    ///
    /// The kinds from [`Self::Carryover`] on, and those whose description says
    /// so, were in none of the files that this crate was checked against.
    pub enum TransactionKind {
        /// A purchase by credit card.
        CreditCardCharge = 1,
        /// A deposit to a bank.
        Deposit = 2,
        /// A check.
        Check = 3,
        /// An invoice.
        Invoice = 4,
        /// A sale paid at once. Unchecked.
        SalesReceipt = 5,
        /// A credit memo.
        CreditMemo = 6,
        /// A payment received from a customer.
        Payment = 7,
        /// A journal entry.
        JournalEntry = 8,
        /// A bill from a vendor.
        Bill = 9,
        /// A refund to a credit card.
        CreditCardCredit = 11,
        /// A credit from a vendor.
        VendorCredit = 12,
        /// A refund to a customer. Unchecked.
        CustomerRefund = 13,
        /// Bills paid by check.
        BillPaymentCheck = 14,
        /// Bills paid by credit card.
        BillPaymentCreditCard = 15,
        /// Sales tax paid to its agency.
        SalesTaxPayment = 16,
        /// A purchase order.
        PurchaseOrder = 17,
        /// A change of quantities or values in stock.
        InventoryAdjustment = 18,
        /// Goods received before their bill.
        ItemReceipt = 19,
        /// A paycheck. Unchecked.
        Paycheck = 20,
        /// A check for payroll liabilities. Unchecked.
        PayrollLiabilityCheck = 21,
        /// Pay of earlier in the year. Unchecked.
        YearToDateAdjustment = 22,
        /// An adjustment of payroll liabilities. Unchecked.
        LiabilityAdjustment = 23,
        /// An estimate.
        Estimate = 24,
        /// A charge on a customer's statement.
        StatementCharge = 25,
        /// A transfer between two accounts.
        Transfer = 26,
        /// A sales order.
        SalesOrder = 27,
        /// A build of an assembly.
        AssemblyBuild = 29,
        /// Stock moved from one site to another.
        InventoryTransfer = 36,
        /// What is carried over. Its meaning is not known.
        Carryover = 28,
        /// A check for payroll liabilities, paid through the tax service.
        ElectronicLiabilityCheck = 30,
        /// A payment of earlier payroll liabilities.
        PriorPayment = 31,
        /// A check that refunds a customer.
        RefundCheck = 32,
        /// An electronic payment.
        ElectronicPayment = 33,
        /// An electronic refund.
        ElectronicRefund = 34,
        /// Bills paid electronically.
        ElectronicBillPayment = 35,
    }
}

impl TransactionKind {
    /// Whether transactions of the kind are never part of the books: estimates
    /// and orders, which say what may happen.
    #[must_use]
    pub const fn never_posts(self) -> bool {
        matches!(
            self,
            Self::Estimate | Self::PurchaseOrder | Self::SalesOrder
        )
    }
}

words! {
    /// Writes the type as QuickBooks' reports have it, such as `Bill Pmt -Check`.
    /// Those of the kinds that were in no file are inferred.
    TransactionKind, a "kind of transaction", or else "Transaction" {
        CreditCardCharge => "Credit Card Charge",
        Deposit => "Deposit",
        Check => "Check",
        Invoice => "Invoice",
        SalesReceipt => "Sales Receipt",
        CreditMemo => "Credit Memo",
        Payment => "Payment",
        JournalEntry => "General Journal",
        Bill => "Bill",
        CreditCardCredit => "Credit Card Credit",
        VendorCredit => "Bill Credit",
        CustomerRefund => "Refund",
        BillPaymentCheck => "Bill Pmt -Check",
        BillPaymentCreditCard => "Bill Pmt -CCard",
        SalesTaxPayment => "Sales Tax Payment",
        PurchaseOrder => "Purchase Order",
        InventoryAdjustment => "Inventory Adjust",
        ItemReceipt => "Item Receipt",
        Paycheck => "Paycheck",
        PayrollLiabilityCheck => "Liability Check",
        YearToDateAdjustment => "YTD Adjust.",
        LiabilityAdjustment => "Liability Adjust.",
        Estimate => "Estimate",
        StatementCharge => "Stmt Charge",
        Transfer => "Transfer",
        SalesOrder => "Sales Order",
        AssemblyBuild => "Build Assembly",
        InventoryTransfer => "Inventory Transfer",
        Carryover => "Carryover",
        ElectronicLiabilityCheck => "Electronic Liability Check",
        PriorPayment => "Prior Payment",
        RefundCheck => "Refund Check",
        ElectronicPayment => "Electronic Payment",
        ElectronicRefund => "Electronic Refund",
        ElectronicBillPayment => "Electronic Bill Payment",
    }
}

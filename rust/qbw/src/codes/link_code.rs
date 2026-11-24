//! What ties two transactions together.

use crate::code::codes;

codes! {
    /// What ties a line of a transaction to a line of another.
    ///
    /// A number that has no name here was only ever seen between a transaction
    /// and itself, which no screen of QuickBooks shows.
    pub enum LinkCode {
        /// A check that refunds a credit memo.
        CreditRefund = 2,
        /// A credit memo, applied to an invoice.
        Credit = 3,
        /// A payment, applied to an invoice.
        Payment = 4,
        /// A discount taken with a payment.
        PaymentDiscount = 5,
        /// A bill paid by check.
        BillCheck = 6,
        /// A bill paid by card, or with a journal entry.
        BillCard = 7,
        /// A credit of a vendor, applied to a bill.
        BillCredit = 8,
        /// A payment in a deposit.
        Deposit = 9,
        /// A check that refunds what was paid too much.
        PaymentRefund = 10,
        /// Goods received on a purchase order, as a quantity.
        Receipt = 15,
        /// A discount taken with a payment of bills.
        BillDiscount = 16,
        /// What was invoiced of a sales order, as a quantity.
        InvoicedQuantity = 20,
        /// What was invoiced of a sales order, as an amount.
        InvoicedAmount = 21,
        /// A journal entry, applied to an invoice.
        JournalCredit = 22,
    }
}

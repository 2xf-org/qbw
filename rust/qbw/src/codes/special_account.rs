//! The accounts that QuickBooks makes for itself.

use crate::code::codes;

codes! {
    /// What QuickBooks uses an account for that it made itself. Its name may
    /// have been changed, and this has not.
    pub enum SpecialAccount {
        /// The other side of balances that the company started with.
        OpeningBalanceEquity = 1,
        /// What customers owe. An account in another currency has it, too.
        AccountsReceivable = 2,
        /// What vendors are owed. An account in another currency has it, too.
        AccountsPayable = 3,
        /// Sales tax collected and not yet paid.
        SalesTaxPayable = 4,
        /// Money received and not yet taken to a bank.
        UndepositedFunds = 5,
        /// The other side of balances that customers started with.
        UncategorizedIncome = 6,
        /// The other side of balances that vendors started with.
        UncategorizedExpenses = 7,
        /// The profit of earlier years.
        RetainedEarnings = 8,
        /// Purchase orders.
        PurchaseOrders = 10,
        /// What is owed on wages that were paid.
        PayrollLiabilities = 11,
        /// Wages.
        PayrollExpenses = 12,
        /// Estimates.
        Estimates = 14,
        /// The value of the stock.
        InventoryAsset = 15,
        /// What the goods that were sold had cost.
        CostOfGoodsSold = 16,
        /// Sales orders.
        SalesOrders = 20,
        /// What did not agree when an account was reconciled.
        ReconciliationDiscrepancies = 23,
        /// What exchange rates gave or took.
        ExchangeGainOrLoss = 24,
    }
}

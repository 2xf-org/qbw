//! The types of accounts.

use crate::code::{codes, words};

codes! {
    /// The type of an account, which decides where it stands in the statements.
    pub enum AccountType {
        /// Money in a bank.
        Bank = 0,
        /// What customers owe.
        AccountsReceivable = 1,
        /// An asset that turns into money within a year, such as stock.
        OtherCurrentAsset = 2,
        /// Something owned for years, such as a vehicle.
        FixedAsset = 3,
        /// Any other asset.
        OtherAsset = 4,
        /// What vendors are owed.
        AccountsPayable = 5,
        /// A credit card.
        CreditCard = 6,
        /// A debt due within a year, such as sales tax.
        OtherCurrentLiability = 7,
        /// A debt due later.
        LongTermLiability = 8,
        /// What the owners have in the company.
        Equity = 9,
        /// What the company earns in its trade.
        Income = 10,
        /// What the goods it sold had cost.
        CostOfGoodsSold = 11,
        /// What it spends in its trade.
        Expense = 12,
        /// What it earns outside its trade.
        OtherIncome = 13,
        /// What it spends outside its trade.
        OtherExpense = 14,
        /// Orders and estimates, which are not part of the books.
        NonPosting = 15,
    }
}

words! {
    /// Writes the type as the chart of accounts has it.
    AccountType, a "type of account", or else "Account type" {
        Bank => "Bank",
        AccountsReceivable => "Accounts Receivable",
        OtherCurrentAsset => "Other Current Asset",
        FixedAsset => "Fixed Asset",
        OtherAsset => "Other Asset",
        AccountsPayable => "Accounts Payable",
        CreditCard => "Credit Card",
        OtherCurrentLiability => "Other Current Liability",
        LongTermLiability => "Long Term Liability",
        Equity => "Equity",
        Income => "Income",
        CostOfGoodsSold => "Cost of Goods Sold",
        Expense => "Expense",
        OtherIncome => "Other Income",
        OtherExpense => "Other Expense",
        NonPosting => "Non-Posting",
    }
}

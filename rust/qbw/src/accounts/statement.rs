//! The statement that an account is on.

use crate::codes::AccountType;

/// The statement that an account is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Statement {
    /// The balance sheet: its balance goes on from year to year.
    BalanceSheet,
    /// Profit and loss: its balance starts again each year.
    ProfitAndLoss,
}

impl AccountType {
    /// The statement that accounts of this type are on, or `None` for a type
    /// that is not part of the books or that this version does not know.
    #[must_use]
    pub const fn statement(self) -> Option<Statement> {
        match self {
            Self::Bank
            | Self::AccountsReceivable
            | Self::OtherCurrentAsset
            | Self::FixedAsset
            | Self::OtherAsset
            | Self::AccountsPayable
            | Self::CreditCard
            | Self::OtherCurrentLiability
            | Self::LongTermLiability
            | Self::Equity => Some(Statement::BalanceSheet),
            Self::Income
            | Self::CostOfGoodsSold
            | Self::Expense
            | Self::OtherIncome
            | Self::OtherExpense => Some(Statement::ProfitAndLoss),
            Self::NonPosting | Self::Other(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_balance_sheet_ends_with_equity() {
        let last = AccountType::Equity.code();
        for code in 0..15 {
            let expected = if code <= last {
                Statement::BalanceSheet
            } else {
                Statement::ProfitAndLoss
            };
            let statement = AccountType::from_code(code).statement();
            assert_eq!(statement, Some(expected), "{code}");
        }
        assert_eq!(AccountType::NonPosting.statement(), None);
    }
}

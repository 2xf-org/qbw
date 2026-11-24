//! The side that an account grows on.

use std::ops::Neg;

use crate::codes::AccountType;
use crate::money::Money;

/// The side that an account grows on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum NormalSide {
    /// Debits add to it: assets, costs and expenses.
    Debit,
    /// Credits add to it: liabilities, equity and income.
    Credit,
}

impl AccountType {
    /// The side that accounts of this type grow on, or `None` for a type that
    /// is not part of the books or that this version does not know.
    #[must_use]
    pub const fn normal_side(self) -> Option<NormalSide> {
        match self {
            Self::Bank
            | Self::AccountsReceivable
            | Self::OtherCurrentAsset
            | Self::FixedAsset
            | Self::OtherAsset
            | Self::CostOfGoodsSold
            | Self::Expense
            | Self::OtherExpense => Some(NormalSide::Debit),
            Self::AccountsPayable
            | Self::CreditCard
            | Self::OtherCurrentLiability
            | Self::LongTermLiability
            | Self::Equity
            | Self::Income
            | Self::OtherIncome => Some(NormalSide::Credit),
            Self::NonPosting | Self::Other(_) => None,
        }
    }

    /// An amount or a balance as an account of this type is kept: above zero on
    /// the side that it grows on. In the books a debit is positive, and so is it
    /// in `posted`, such as a [`Posting`], a [`Balance`], a [`Register`] or an
    /// [`AccountListing`] has it. An account without a side keeps the sign.
    ///
    /// ```
    /// use qbw::accounts::AccountType;
    /// use qbw::{AsOf, Company};
    ///
    /// # let file = qbw_synthetic::company::books().build();
    /// let company = Company::from_bytes(file)?;
    /// let card = company.accounts()?.named("Company Card").ok_or("no such account")?;
    /// let balance = company.ledger()?.balance(card.id, AsOf::Latest);
    /// let balance = balance.ok_or("no balance")?;
    /// assert_eq!(balance.amount.to_string(), "-25.00");
    /// assert_eq!(AccountType::CreditCard.as_kept(balance).amount.to_string(), "25.00");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [`AccountListing`]: crate::reports::AccountListing
    /// [`Balance`]: crate::ledger::Balance
    /// [`Posting`]: crate::ledger::Posting
    /// [`Register`]: crate::registers::Register
    pub fn as_kept<T: Neg<Output = T>>(self, posted: T) -> T {
        match self.normal_side() {
            Some(NormalSide::Credit) => -posted,
            Some(NormalSide::Debit) | None => posted,
        }
    }

    /// What `posted` adds to an account of this type, for the column of a
    /// register that has what adds to it: deposits in a bank, charges of a card,
    /// what vendors bill. `None` for what takes off, and for nothing.
    #[must_use]
    pub fn increase(self, posted: Money) -> Option<Money> {
        let kept = self.as_kept(posted);
        Some(kept).filter(|kept| !kept.is_zero() && !kept.is_negative())
    }

    /// What `posted` takes off an account of this type, as an amount above zero,
    /// for the other column: payments from a bank, payments of a card. `None` for
    /// what adds, and for nothing.
    #[must_use]
    pub fn decrease(self, posted: Money) -> Option<Money> {
        self.increase(-posted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::CurrencyId;

    #[test]
    fn every_type_of_the_books_has_a_side() {
        for code in 0..15 {
            assert!(
                AccountType::from_code(code).normal_side().is_some(),
                "{code}"
            );
        }
        assert_eq!(AccountType::NonPosting.normal_side(), None);
        assert_eq!(AccountType::Other(99).normal_side(), None);
    }

    #[test]
    fn an_account_is_kept_above_zero_on_the_side_it_grows_on() {
        let money = |text: &str| Money::new(text.parse().unwrap(), CurrencyId::COMPANY);
        let (debit, credit) = (money("5"), money("-5"));
        let columns = |account_type: AccountType, posted| {
            let columns = [account_type.increase(posted), account_type.decrease(posted)];
            (account_type.as_kept(posted), columns)
        };
        let adds = (debit, [Some(debit), None]);
        let takes_off = (credit, [None, Some(debit)]);
        assert_eq!(columns(AccountType::Bank, debit), adds);
        assert_eq!(columns(AccountType::Bank, credit), takes_off);
        assert_eq!(columns(AccountType::CreditCard, credit), adds);
        assert_eq!(columns(AccountType::CreditCard, debit), takes_off);
        // One without a side keeps the sign.
        assert_eq!(AccountType::NonPosting.as_kept(credit), credit);
        assert_eq!(columns(AccountType::Income, money("0")).1, [None, None]);
    }
}

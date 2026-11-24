//! The statements and reports of a company.

use chrono::NaiveDate;
use sa17::Source;

use crate::as_of::AsOf;
use crate::company::Company;
use crate::error::Result;
use crate::ids::NameId;
use crate::period::Period;
use crate::reports;
use crate::reports::{
    AccountListing, BalanceSheet, Books, GeneralLedger, Journal, NameBalances, NameLedger,
    ProfitAndLoss, TrialBalance,
};
use crate::side::Side;

/// # Statements and reports
impl<S: Source> Company<S> {
    /// The fiscal year that `date` is in.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::preferences`].
    pub fn fiscal_year(&self, date: NaiveDate) -> Result<Period> {
        let preferences = self.preferences()?;
        Ok(Period::year_of(date, preferences.fiscal_year_start))
    }

    /// The balance of every account at the end of a day.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::ledger`].
    pub fn trial_balance(&self, as_of: impl Into<AsOf>) -> Result<TrialBalance> {
        let day = self.day(as_of)?;
        Ok(reports::trial_balance(self.books()?, day))
    }

    /// What the company earned and spent in a period, such as a
    /// [fiscal year](Self::fiscal_year).
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::ledger`].
    pub fn profit_and_loss(&self, period: impl Into<Period>) -> Result<ProfitAndLoss> {
        Ok(reports::profit_and_loss(self.books()?, period.into()))
    }

    /// What the company has and what it owes at the end of a day.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::ledger`].
    pub fn balance_sheet(&self, as_of: impl Into<AsOf>) -> Result<BalanceSheet> {
        let day = self.day(as_of)?;
        Ok(reports::balance_sheet(self.books()?, day))
    }

    /// What is posted to every account in a period.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::ledger`].
    pub fn general_ledger(&self, period: impl Into<Period>) -> Result<GeneralLedger<'_>> {
        Ok(reports::general_ledger(self.books()?, period.into()))
    }

    /// Every transaction of a period with what it posts.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::ledger`].
    pub fn journal(&self, period: impl Into<Period>) -> Result<Journal<'_>> {
        Ok(Journal::new(self.ledger()?, period.into()))
    }

    /// The chart of accounts with its balances.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::ledger`].
    pub fn account_listing(&self, as_of: impl Into<AsOf>) -> Result<AccountListing> {
        Ok(reports::account_listing(self.books()?, as_of.into()))
    }

    /// What every customer owes or every vendor is owed.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::ledger`] and [`Self::names`].
    pub fn name_balances(&self, side: Side, as_of: impl Into<AsOf>) -> Result<NameBalances> {
        let (ledger, names) = (self.ledger()?, self.names()?);
        Ok(reports::name_balances(ledger, names, side, as_of.into()))
    }

    /// What a customer owes or a vendor is owed in a period, posting by posting.
    /// `None` when there is no such customer or vendor.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::ledger`] and [`Self::names`].
    pub fn name_ledger(
        &self,
        side: Side,
        name: NameId,
        period: impl Into<Period>,
    ) -> Result<Option<NameLedger>> {
        if !self.names()?.is_on(side, name) {
            return Ok(None);
        }
        let ledger = self.ledger()?;
        let report = reports::name_ledger(ledger, side, name, period.into());
        Ok(Some(report))
    }

    /// The day of a moment: of now, the last that something is posted on.
    pub(super) fn day(&self, as_of: impl Into<AsOf>) -> Result<NaiveDate> {
        Ok(match as_of.into() {
            AsOf::Date(day) => day,
            AsOf::Latest => self.ledger()?.last_day().unwrap_or_default(),
        })
    }

    pub(super) fn books(&self) -> Result<Books<'_>> {
        Ok(Books {
            ledger: self.ledger()?,
            accounts: self.accounts()?,
            year_start: self.preferences()?.fiscal_year_start,
        })
    }
}

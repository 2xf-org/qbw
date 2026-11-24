//! The registers of a company.

use sa17::Source;

use crate::company::Company;
use crate::error::Result;
use crate::ids::AccountId;
use crate::ledger::Posting;
use crate::page::Page;
use crate::period::Period;
use crate::registers;
use crate::registers::{PostingTexts, Register};

/// # Registers
impl<S: Source> Company<S> {
    /// A page of the register of an account in a period. Only the rows of the
    /// page are read from the file. `None` when there is no such account and
    /// nothing is posted to one.
    ///
    /// The register of an account of profit and loss opens as in the [general
    /// ledger](Self::general_ledger): with what was posted since the fiscal year
    /// started. That of Retained Earnings opens with the profit of the years
    /// before. [`Ledger::balance`](crate::ledger::Ledger::balance) is what was
    /// posted to it in all.
    ///
    /// ```
    /// use qbw::{AccountId, Company, Page, Period};
    ///
    /// # let file = qbw_synthetic::company::books().build();
    /// let company = Company::from_bytes(file)?;
    /// let checking = AccountId::new(1);
    /// let register = company.register(checking, Period::all(), Page::new(0, 100))?;
    /// let register = register.ok_or("no such account")?;
    /// let last = register.rows.last().ok_or("no rows")?;
    /// assert_eq!(last.balance, register.closing.amount);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::ledger`].
    pub fn register(
        &self,
        account: AccountId,
        period: impl Into<Period>,
        page: Page,
    ) -> Result<Option<Register>> {
        let (books, headers) = (self.books()?, self.header_index()?);
        if !books.ledger.knows(account) {
            return Ok(None);
        }
        let of = (period.into(), page);
        registers::read(self.source(), books, headers, account, of).map(Some)
    }

    /// The number and the memo of postings, such as the ones of a journal, a
    /// general ledger or a name ledger that are shown. One row of the file is
    /// read for each.
    ///
    /// ```
    /// use qbw::{Company, Page, Period};
    ///
    /// # let file = qbw_synthetic::company::books().build();
    /// let company = Company::from_bytes(file)?;
    /// let journal = company.journal(Period::all())?;
    /// for entry in journal.page(Page::new(0, 50)) {
    ///     let texts = company.posting_texts(&entry.postings)?;
    ///     for (posting, text) in entry.postings.iter().zip(&texts.rows) {
    ///         println!("{} {:?} {}", entry.kind, text.memo, posting.amount);
    ///     }
    /// }
    /// # Ok::<(), qbw::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::ledger`].
    pub fn posting_texts(&self, postings: &[Posting]) -> Result<PostingTexts> {
        let (ledger, headers) = (self.ledger()?, self.header_index()?);
        registers::texts(self.source(), ledger, headers, postings)
    }
}

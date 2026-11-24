//! The transactions of a company.

use sa17::Source;

use crate::codes::TransactionKind;
use crate::company::Company;
use crate::error::{Error, Result};
use crate::ids::TransactionId;
use crate::page::Page;
use crate::period::Period;
use crate::query::{Search, TransactionPage, TransactionQuery, TransactionSummary};
use crate::tables::Form;
use crate::transactions::{
    Header, Headers, LineIndex, OwnLines, PostingState, Reader, Transaction,
};

/// # Transactions
impl<S: Source> Company<S> {
    /// The transaction with `id`, with its lines. `None` when the file has no
    /// such transaction.
    ///
    /// What cannot be read of its lines is in [`Transaction::damage`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Damaged`] when the transaction is there and its header
    /// cannot be read, and an error when the file lacks a table or a column of
    /// it.
    pub fn transaction(&self, id: TransactionId) -> Result<Option<Transaction>> {
        let Some(header) = self.header_index()?.get(id) else {
            return self.lost_header(id);
        };
        self.transaction_of(header).map(Some)
    }

    /// The transaction with `id` as a list of transactions shows it, without its
    /// lines. `None` when the file has no such transaction.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::transaction`].
    pub fn transaction_summary(&self, id: TransactionId) -> Result<Option<TransactionSummary>> {
        if self.header_index()?.get(id).is_none() {
            self.lost_header(id)?;
            return Ok(None);
        }
        self.search(&TransactionQuery::new(), |search| search.one(id))
    }

    /// The days from the first transaction to the last, with those that are not
    /// part of the books, such as orders, and without the memorized ones. `None`
    /// when there are no transactions. Only the headers are read, and one that
    /// cannot be read counts for nothing.
    ///
    /// ```
    /// use qbw::Company;
    ///
    /// # let file = qbw_synthetic::company::books().build();
    /// let company = Company::from_bytes(file)?;
    /// let days = company.transaction_days()?.ok_or("no transactions")?;
    /// let days = [days.first(), days.last()].map(|day| day.map(|day| day.to_string()));
    /// assert_eq!(days.map(Option::unwrap_or_default), ["2023-03-01", "2024-06-30"]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when the file lacks a table of headers or a column of it.
    pub fn transaction_days(&self) -> Result<Option<Period>> {
        let headers = self.header_index()?.iter();
        let listed = headers.filter(|header| header.posting != PostingState::Memorized);
        let days = listed.map(|header| (header.date, header.date));
        let days = days.reduce(|(first, last), (day, _)| (first.min(day), last.max(day)));
        Ok(days.map(|(first, last)| Period::between(first, last)))
    }

    /// The page `page` of the transactions that `query` asks for.
    ///
    /// # Errors
    ///
    /// Returns an error when the file lacks a table or a column of it, or when
    /// the company's currency cannot be understood.
    pub fn transactions(&self, query: &TransactionQuery, page: Page) -> Result<TransactionPage> {
        self.search(query, |search| search.page(page))
    }

    /// The transaction of `kind` with `number`, ignoring case. Numbers repeat,
    /// and of several it is the one with the latest date. A memorized one is
    /// not found, and neither is one whose header cannot be read.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::transactions`].
    pub fn find(&self, kind: TransactionKind, number: &str) -> Result<Option<TransactionId>> {
        let found = self.find_all(kind, number)?;
        Ok(found.first().copied())
    }

    /// Every transaction of `kind` with `number`, the latest first. Memorized
    /// ones and those whose header cannot be read are not among them, and
    /// [`Self::transactions`] says how many of these there are.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::transactions`].
    pub fn find_all(&self, kind: TransactionKind, number: &str) -> Result<Vec<TransactionId>> {
        let query = TransactionQuery::new().kinds([kind]).number(number);
        self.search(&query, |search| search.ids())
    }

    /// The transaction of `header`, with its lines.
    pub(super) fn transaction_of(&self, header: &Header) -> Result<Transaction> {
        let lines = self.line_index(header.form)?;
        self.reader()?.read(header, lines.rows(header.id))
    }

    /// What reads transactions.
    pub(super) fn reader(&self) -> Result<Reader<'_>> {
        Ok(Reader {
            source: self.source(),
            tables: &self.pieces().opened_tables,
            home: self.preferences()?.home_currency,
            accounts: self.accounts()?,
            items: self.items()?,
            terms: self.terms()?,
            sales_reps: self.sales_reps()?,
        })
    }

    fn search<T>(
        &self,
        query: &TransactionQuery,
        answer: impl FnOnce(&Search<'_>) -> Result<T>,
    ) -> Result<T> {
        let names = query.is_about_names().then(|| self.names());
        let ledger = query.is_about_postings().then(|| self.ledger());
        answer(&Search {
            query,
            source: self.source(),
            home: self.preferences()?.home_currency,
            headers: self.header_index()?,
            accounts: self.accounts()?,
            names: names.transpose()?,
            ledger: ledger.transpose()?,
            own_lines: &|form| self.own_lines(form),
        })
    }

    /// What became of a transaction that no header which can be read is of:
    /// an error when it has lines, and else `None`.
    fn lost_header(&self, id: TransactionId) -> Result<Option<Transaction>> {
        let headers = self.header_index()?;
        for form in Form::all() {
            let Some(lost) = headers.lost(form) else {
                continue;
            };
            if self.line_index(form)?.rows(id).len() > 0 {
                return Err(Error::Damaged(lost));
            }
        }
        Ok(None)
    }

    /// The headers of every transaction.
    pub(super) fn header_index(&self) -> Result<&Headers> {
        let pieces = self.pieces();
        pieces.headers.get_or_build(|| Headers::read(self.source()))
    }

    /// Where the lines of the transactions of `form` are.
    pub(super) fn line_index(&self, form: Form) -> Result<&LineIndex> {
        let piece = self.pieces().line_indexes.of(form);
        piece.get_or_build(|| LineIndex::read(self.source(), form))
    }

    /// The own lines of the transactions of `form`.
    pub(super) fn own_lines(&self, form: Form) -> Result<&OwnLines> {
        let piece = self.pieces().own_lines.of(form);
        piece.get_or_build(|| {
            let home = self.preferences()?.home_currency;
            OwnLines::read(self.source(), form, self.accounts()?, home)
        })
    }
}

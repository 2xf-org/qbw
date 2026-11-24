//! What the transactions of a company have to do with each other.

use sa17::Source;

use crate::company::Company;
use crate::error::Result;
use crate::ids::TransactionId;
use crate::links::{LineLinks, LinkDates, LinkIndex, LinkReader, Links, Relation, Settled};

/// # What paid what
impl<S: Source> Company<S> {
    /// How a transaction belongs to others: what it pays or was paid by, what
    /// it was deposited in, which order it was made from. `None` when there is
    /// no such transaction.
    ///
    /// # Errors
    ///
    /// Returns an error when the file lacks the table of the links, or a column
    /// of them or of the headers.
    pub fn links(&self, transaction: TransactionId) -> Result<Option<Links>> {
        self.link_reader()?.links(transaction)
    }

    /// What an invoice or a bill was paid by: the payments, the credits and the
    /// discounts that settled it, or the part of it that is settled. `None` when
    /// there is no such transaction.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::links`].
    pub fn paid_by(&self, transaction: TransactionId) -> Result<Option<Settled>> {
        let links = self.links(transaction)?;
        Ok(links.map(|links| links.settled(Relation::is_settled_by)))
    }

    /// What a payment or a credit paid: the invoices or the bills that it was
    /// applied to, with the discounts taken. `None` when there is no such
    /// transaction.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::links`].
    pub fn applied_to(&self, transaction: TransactionId) -> Result<Option<Settled>> {
        let links = self.links(transaction)?;
        Ok(links.map(|links| links.settled(Relation::settles)))
    }

    /// How the lines of a transaction belong to lines of others, such as a line
    /// of a bill to the one of the order that it fills. `None` when there is no
    /// such transaction.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::links`].
    pub fn line_links(&self, transaction: TransactionId) -> Result<Option<LineLinks>> {
        self.link_reader()?.line_links(transaction)
    }

    fn link_reader(&self) -> Result<LinkReader<'_>> {
        Ok(LinkReader {
            source: self.source(),
            index: self.link_index()?,
            headers: self.header_index()?,
            home: self.preferences()?.home_currency,
        })
    }

    /// The links of every transaction.
    pub(super) fn link_index(&self) -> Result<&LinkIndex> {
        let pieces = self.pieces();
        pieces.links.get_or_build(|| LinkIndex::read(self.source()))
    }

    /// The dates that the transactions with links post on.
    pub(super) fn link_dates(&self) -> Result<&LinkDates> {
        let pieces = self.pieces();
        pieces.link_dates.get_or_build(|| {
            let dates = self.ledger()?.dates();
            Ok(LinkDates::of(self.link_index()?, dates))
        })
    }
}

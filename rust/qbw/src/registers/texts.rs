//! What postings show besides what the ledger keeps.

use chrono::NaiveDate;
use sa17::Source;

use crate::error::Result;
use crate::ledger::{Ledger, Posting};
use crate::left_out::LeftOut;
use crate::registers::lines::Lines;
use crate::transactions::{Header, Headers, ShownHeaders};

/// What postings show besides what the ledger keeps of them.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct PostingTexts {
    /// The texts of each posting, in the order they were asked for.
    pub rows: Vec<PostingText>,
    /// How much could not be read.
    pub left_out: LeftOut,
}

/// What a posting shows besides what the ledger keeps of it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct PostingText {
    /// The number of the transaction, as the form shows it.
    pub number: Option<String>,
    /// What the line is about.
    pub memo: Option<String>,
    /// The date of the statement that the line was reconciled with, which the
    /// file has for some of the reconciled lines.
    pub reconciled_on: Option<NaiveDate>,
}

/// Reads the texts of `postings`, one row of `source` for each.
///
/// # Errors
///
/// Returns an error when the table of a line lacks a column.
pub(crate) fn texts(
    source: &dyn Source,
    ledger: &Ledger,
    headers: &Headers,
    postings: &[Posting],
) -> Result<PostingTexts> {
    let mut lines = Lines::new(source, ledger);
    let mut headers = ShownHeaders::new(headers);
    let rows = postings.iter().map(|posting| {
        let line = lines.read(posting.transaction, posting.line, posting.origin)?;
        let header = headers.of_kind(posting.transaction, posting.kind);
        Ok(PostingText {
            number: header.and_then(Header::number),
            reconciled_on: line.as_ref().and_then(|line| line.reconciled_on),
            memo: line.and_then(|line| line.memo),
        })
    });
    let rows = rows.collect::<Result<_>>()?;
    let texts = lines.left_out().with(headers.left_out());
    Ok(PostingTexts {
        rows,
        left_out: ledger.left_out().with(texts),
    })
}

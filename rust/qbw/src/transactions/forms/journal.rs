//! What a journal entry has.

use crate::tables::headers::JournalRow;
use crate::tables::lines::LineRow;

/// What a journal entry has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct JournalForm {
    /// Whether an accountant made it to adjust the books.
    pub is_adjustment: bool,
    /// Whether it changes what accounts in another currency are worth in the
    /// company's, and not what they hold.
    pub is_home_currency_adjustment: bool,
    /// Whether its form shows the company's currency, which
    /// [`Line::debit`](crate::transactions::Line::debit) and
    /// [`Line::credit`](crate::transactions::Line::credit) are in then.
    pub is_shown_in_home_currency: bool,
}

impl JournalForm {
    pub(crate) fn new(row: &JournalRow, own: Option<&LineRow>) -> Self {
        Self {
            is_adjustment: own.is_some_and(|own| own.is_adjustment),
            is_home_currency_adjustment: row.is_home_currency_adjustment,
            is_shown_in_home_currency: row.is_shown_in_home_currency,
        }
    }
}

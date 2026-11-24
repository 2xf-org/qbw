//! The header of a transaction.

use chrono::NaiveDate;
use sa17::RowId;

use crate::codes::{LastAction, TransactionKind};
use crate::ids::{LineId, NameId, TransactionId};
use crate::origin::Origin;
use crate::party::Party;
use crate::posting_state::PostingState;
use crate::tables::Form;
use crate::tables::headers::HeaderKeyRow;

/// What finding a transaction takes, of its header.
#[derive(Debug)]
pub(crate) struct Header {
    pub(crate) id: TransactionId,
    pub(crate) form: Form,
    pub(crate) row: RowId,
    pub(crate) own_line: LineId,
    pub(crate) kind: TransactionKind,
    pub(crate) date: NaiveDate,
    pub(crate) number: Option<Box<str>>,
    pub(crate) party: Option<Party>,
    pub(crate) posting: PostingState,
    pub(crate) last_action: LastAction,
    /// Whether it is a payment that moved no money: a discount is all of it.
    pub(crate) is_without_money: bool,
}

impl Header {
    pub(super) fn new(form: Form, row: RowId, read: HeaderKeyRow) -> Self {
        let kind = form.kind(read.kind);
        Self {
            id: read.id,
            form,
            row,
            own_line: read.own_line,
            kind,
            date: read.date,
            party: read.names().of_transaction(),
            posting: read.flags(kind).state(),
            last_action: read.last_action,
            is_without_money: kind == TransactionKind::Payment && read.is_without_money,
            number: read.number.map(String::into_boxed_str),
        }
    }

    /// Whether the form shows the own line among the others. That of a journal
    /// entry is its first line, and that of a payment without money a discount.
    pub(crate) fn shows_own_line(&self) -> bool {
        self.is_without_money || self.kind == TransactionKind::JournalEntry
    }

    /// The number, as the form shows it.
    pub(crate) fn number(&self) -> Option<String> {
        self.number.as_deref().map(str::to_owned)
    }

    pub(crate) fn name(&self) -> Option<NameId> {
        self.party.map(|party| party.name)
    }

    pub(crate) fn origin(&self) -> Origin {
        Origin::new(self.form.headers::<HeaderKeyRow>().name(), self.row)
    }
}

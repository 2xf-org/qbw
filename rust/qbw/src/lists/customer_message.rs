//! Messages for customers.

use sa17::Source;

use crate::error::Result;
use crate::ids::MessageId;
use crate::list::{List, ListEntry, sealed};
use crate::lists::reading;
use crate::origin::Origin;
use crate::tables::lists::CUSTOMER_MESSAGES;

/// A message that prints on the forms that customers get.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct CustomerMessage {
    /// What the message is known by.
    pub id: MessageId,
    /// What it says, which the list calls it by.
    pub text: String,
    /// Whether it can be used in new transactions.
    pub is_active: bool,
    /// The stored row.
    pub origin: Origin,
}

impl sealed::Sealed for CustomerMessage {}

impl ListEntry for CustomerMessage {
    type Id = MessageId;

    fn id(&self) -> MessageId {
        self.id
    }

    fn name(&self) -> &str {
        &self.text
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn origin(&self) -> Origin {
        self.origin
    }
}

/// Reads the messages.
///
/// # Errors
///
/// Returns an error when their table lacks a column.
pub(crate) fn read(source: &dyn Source) -> Result<List<CustomerMessage>> {
    reading::by_name(source, &CUSTOMER_MESSAGES, |origin, row| {
        (!row.is_deleted).then(|| CustomerMessage {
            id: row.id,
            text: row.text.unwrap_or_default(),
            is_active: !row.is_hidden,
            origin,
        })
    })
}

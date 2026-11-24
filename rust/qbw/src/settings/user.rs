//! A person who works on the books.

use std::fmt;

use crate::ids::UserId;

/// A person who works on the books in QuickBooks, as far as the audit trail
/// tells.
///
/// The name someone signs in with can be an e-mail address, so no name is in
/// the `Debug` output.
#[derive(Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct User {
    /// What the person is known by.
    pub id: UserId,
    /// The names that transactions were saved or deleted under with this id,
    /// the latest first. An id can have several at one time.
    pub names: Vec<String>,
}

impl fmt::Debug for User {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("User")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

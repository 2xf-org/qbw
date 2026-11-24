//! What a record has in a field that the company defined.

use crate::ids::OwnerKey;
use crate::origin::Origin;

/// What a record has in a field that the company defined.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct CustomField {
    /// The record: a name, an item, a transaction or a line.
    pub owner: OwnerKey,
    /// Which of the fields it is. Those of names and of their transactions are
    /// counted on their own, and so are those of items and of lines, both from
    /// 0 on.
    pub position: i32,
    /// What the record has in it.
    pub text: String,
    /// The stored row.
    pub origin: Origin,
}

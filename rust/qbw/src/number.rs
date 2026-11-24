//! What the file stores as a whole number.

/// A type that the file stores as a whole number: an id, a code or a time.
///
/// Where a column may have nothing, the file writes 0 or `NULL`, so an
/// `Option` of such a type reads both as `None`.
pub(crate) trait FromNumber: Sized {
    /// The value that `number` stands for, or `None` when there is no such value.
    fn from_number(number: i64) -> Option<Self>;
}

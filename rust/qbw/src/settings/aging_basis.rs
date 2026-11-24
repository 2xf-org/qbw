//! What the age of an open amount is counted from.

/// What the age of an open invoice or bill is counted from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AgingBasis {
    /// The date it is due.
    DueDate,
    /// The date it was entered for.
    TransactionDate,
}

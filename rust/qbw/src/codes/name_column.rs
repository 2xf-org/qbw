//! Which of its four names is a line's own.

use crate::number::FromNumber;

/// Which of the four columns for a name holds a line's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NameColumn {
    /// A customer or a job.
    Customer,
    /// A vendor.
    Vendor,
    /// An employee.
    Employee,
    /// Another name.
    OtherName,
    /// None of them: the columns repeat the name of the transaction.
    Transaction,
}

impl FromNumber for NameColumn {
    fn from_number(number: i64) -> Option<Self> {
        Some(match number {
            100 => Self::Customer,
            101 => Self::Vendor,
            102 => Self::Employee,
            103 => Self::OtherName,
            // The file writes 105.
            _ => Self::Transaction,
        })
    }
}

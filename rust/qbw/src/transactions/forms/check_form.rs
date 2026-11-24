//! What a check has.

use crate::tables::headers::CheckRow;

/// What a check, a payment of sales tax or a paycheck has. Payroll is
/// unchecked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct CheckForm {
    /// Whether it waits to be printed.
    pub print_later: bool,
}

impl CheckForm {
    pub(crate) fn new(row: &CheckRow) -> Self {
        Self {
            print_later: row.is_to_be_printed,
        }
    }
}

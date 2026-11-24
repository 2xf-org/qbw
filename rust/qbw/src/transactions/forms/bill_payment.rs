//! What a payment of bills has.

use crate::tables::headers::CheckRow;

/// What a payment of bills has, by check or by credit card.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BillPayment {
    /// Whether its check waits to be printed.
    pub print_later: bool,
}

impl BillPayment {
    pub(crate) fn new(row: &CheckRow) -> Self {
        Self {
            print_later: row.is_to_be_printed,
        }
    }
}

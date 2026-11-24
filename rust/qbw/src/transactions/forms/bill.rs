//! What a bill has.

use crate::ids::TermsId;
use crate::tables::headers::BillRow;

/// What a bill, a vendor credit or an item receipt has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Bill {
    /// The terms of payment.
    pub terms: Option<TermsId>,
}

impl Bill {
    pub(crate) fn new(row: &BillRow) -> Self {
        Self { terms: row.terms }
    }
}

//! What a refund to a customer has.

use crate::ids::PaymentMethodId;
use crate::tables::headers::ReceiptRow;

/// What a refund to a customer has. Unchecked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Refund {
    /// How it was made.
    pub payment_method: Option<PaymentMethodId>,
    /// Whether it is part of a deposit.
    pub is_deposited: bool,
}

impl Refund {
    pub(crate) fn new(row: &ReceiptRow) -> Self {
        Self {
            payment_method: row.payment_method,
            is_deposited: row.is_deposited,
        }
    }
}

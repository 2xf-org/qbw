//! What a payment from a customer has.

use crate::ids::PaymentMethodId;
use crate::tables::headers::ReceiptRow;

/// What a payment received from a customer has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Payment {
    /// How it was made.
    pub payment_method: Option<PaymentMethodId>,
    /// Whether it was taken to a bank.
    pub is_deposited: bool,
    /// Whether it moved money. One that did not only takes a discount off an
    /// invoice or applies a credit, and its total is zero.
    pub moved_money: bool,
    /// Whether the bank sent it back. Unchecked.
    pub is_bounced: bool,
}

impl Payment {
    pub(crate) fn new(row: &ReceiptRow, is_without_money: bool) -> Self {
        Self {
            payment_method: row.payment_method,
            is_deposited: row.is_deposited,
            moved_money: !is_without_money,
            is_bounced: row.is_bounced,
        }
    }
}

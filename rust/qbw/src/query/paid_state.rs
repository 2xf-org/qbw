//! Whether something is left of a transaction.

/// Whether something is left of a transaction: to pay of an invoice or a bill,
/// or to use of a payment or a credit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PaidState {
    /// Nothing is left.
    Paid,
    /// Something is.
    Unpaid,
}

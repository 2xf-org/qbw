//! How two transactions belong together.

use std::fmt;

use crate::codes::{LinkCode, TransactionKind};

/// How a transaction belongs to another, seen from the first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Relation {
    /// It pays or credits the other: a payment an invoice, a check a bill.
    AppliedTo,
    /// The other pays it.
    PaidBy,
    /// The other credits it, as a credit memo does an invoice.
    CreditedBy,
    /// It takes a discount on the other.
    DiscountOn,
    /// The other takes a discount on it.
    DiscountFrom,
    /// The other brings it to the bank.
    DepositedIn,
    /// It brings the other to the bank.
    DepositOf,
    /// The other gives its money back.
    RefundedBy,
    /// It gives the other's money back.
    RefundOf,
    /// It was made from the other, which is an order.
    OrderedOn,
    /// The other received what it orders.
    ReceivedOn,
    /// The other invoiced what it orders.
    InvoicedOn,
    /// In a way that this version has no name for.
    LinkedTo,
}

/// The two ends of a link: how its source belongs to the other transaction, and
/// how the other belongs to the source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Ends {
    pub(super) source: Relation,
    pub(super) target: Relation,
}

impl Relation {
    /// How the two ends of a link of `code` belong together. `kind` is that of
    /// the transaction which it is asked for.
    pub(super) fn of(code: LinkCode, kind: TransactionKind) -> Ends {
        let ends = |source, target| Ends { source, target };
        match code {
            LinkCode::Payment | LinkCode::BillCheck | LinkCode::BillCard => {
                ends(Self::AppliedTo, Self::PaidBy)
            }
            LinkCode::Credit | LinkCode::BillCredit | LinkCode::JournalCredit => {
                ends(Self::AppliedTo, Self::CreditedBy)
            }
            LinkCode::PaymentDiscount | LinkCode::BillDiscount => {
                ends(Self::DiscountOn, Self::DiscountFrom)
            }
            LinkCode::Deposit => ends(Self::DepositedIn, Self::DepositOf),
            // Either of the two is the source of a refund.
            LinkCode::CreditRefund | LinkCode::PaymentRefund => {
                ends(Self::of_refund(kind), Self::of_refund(kind))
            }
            LinkCode::Receipt => ends(Self::OrderedOn, Self::ReceivedOn),
            LinkCode::InvoicedQuantity | LinkCode::InvoicedAmount => {
                ends(Self::OrderedOn, Self::InvoicedOn)
            }
            LinkCode::Other(_) => ends(Self::LinkedTo, Self::LinkedTo),
        }
    }

    const fn of_refund(kind: TransactionKind) -> Self {
        match kind {
            TransactionKind::CreditMemo | TransactionKind::Payment => Self::RefundedBy,
            _ => Self::RefundOf,
        }
    }

    /// Whether the other transaction settles this one: pays it, credits it or
    /// takes a discount on it.
    #[must_use]
    pub const fn is_settled_by(self) -> bool {
        matches!(self, Self::PaidBy | Self::CreditedBy | Self::DiscountFrom)
    }

    /// Whether this transaction settles the other: pays it, credits it or takes
    /// a discount on it.
    #[must_use]
    pub const fn settles(self) -> bool {
        matches!(self, Self::AppliedTo | Self::DiscountOn)
    }
}

impl fmt::Display for Relation {
    /// Writes the relation, to go before the other transaction: `paid by`.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.pad(match self {
            Self::AppliedTo => "applied to",
            Self::PaidBy => "paid by",
            Self::CreditedBy => "credited by",
            Self::DiscountOn => "discount on",
            Self::DiscountFrom => "discount from",
            Self::DepositedIn => "deposited in",
            Self::DepositOf => "deposit of",
            Self::RefundedBy => "refunded by",
            Self::RefundOf => "refund of",
            Self::OrderedOn => "ordered on",
            Self::ReceivedOn => "received on",
            Self::InvoicedOn => "invoiced on",
            Self::LinkedTo => "linked to",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_link_has_two_ends() {
        let ends = |code| Relation::of(code, TransactionKind::Invoice);
        let expected = |source, target| Ends { source, target };
        let paid = expected(Relation::AppliedTo, Relation::PaidBy);
        assert_eq!(ends(LinkCode::Payment), paid);
        let discount = expected(Relation::DiscountOn, Relation::DiscountFrom);
        assert_eq!(ends(LinkCode::PaymentDiscount), discount);
        let invoiced = expected(Relation::OrderedOn, Relation::InvoicedOn);
        assert_eq!(ends(LinkCode::InvoicedAmount), invoiced);
        let unknown = expected(Relation::LinkedTo, Relation::LinkedTo);
        assert_eq!(ends(LinkCode::Other(77)), unknown);
    }

    #[test]
    fn what_gave_the_money_back_is_the_refund() {
        let ends = |kind| Relation::of(LinkCode::CreditRefund, kind);
        let memo = ends(TransactionKind::CreditMemo);
        assert_eq!(
            (memo.source, memo.target),
            (Relation::RefundedBy, Relation::RefundedBy)
        );
        let check = ends(TransactionKind::Check);
        assert_eq!(
            (check.source, check.target),
            (Relation::RefundOf, Relation::RefundOf)
        );
    }
}

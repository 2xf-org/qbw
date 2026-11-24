//! Receivable or payable.

use std::ops::Neg;

use crate::codes::{AccountType, LinkCode, TransactionKind};

/// Whether something is owed to the company or by it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Side {
    /// What customers owe the company.
    Receivable,
    /// What the company owes vendors.
    Payable,
}

impl Side {
    /// The type of the accounts that it is on.
    #[must_use]
    pub const fn account_type(self) -> AccountType {
        match self {
            Self::Receivable => AccountType::AccountsReceivable,
            Self::Payable => AccountType::AccountsPayable,
        }
    }

    /// The kind of the orders that are placed on it.
    pub(crate) const fn order_kind(self) -> TransactionKind {
        match self {
            Self::Receivable => TransactionKind::SalesOrder,
            Self::Payable => TransactionKind::PurchaseOrder,
        }
    }

    /// The side whose accounts are of a type, if they are of one.
    pub(crate) const fn of_type(account_type: AccountType) -> Option<Self> {
        match account_type {
            AccountType::AccountsReceivable => Some(Self::Receivable),
            AccountType::AccountsPayable => Some(Self::Payable),
            _ => None,
        }
    }

    /// A sum of postings as something owed: above zero where the customer owes
    /// it, or the company owes it to the vendor.
    pub(crate) fn owed<T: Neg<Output = T>>(self, posted: T) -> T {
        match self {
            Self::Receivable => posted,
            Self::Payable => -posted,
        }
    }

    /// Whether a link of `code` settles something on this side.
    pub(crate) const fn is_settled_by(self, code: LinkCode) -> bool {
        match self {
            Self::Receivable => matches!(
                code,
                LinkCode::Payment
                    | LinkCode::PaymentDiscount
                    | LinkCode::Credit
                    | LinkCode::JournalCredit
                    | LinkCode::CreditRefund
                    | LinkCode::PaymentRefund
            ),
            Self::Payable => matches!(
                code,
                LinkCode::BillCheck
                    | LinkCode::BillCard
                    | LinkCode::BillCredit
                    | LinkCode::BillDiscount
            ),
        }
    }
}

impl TransactionKind {
    /// Whose order a transaction of this kind is: a sales order is of a customer,
    /// on the side of [`Side::Receivable`], and a purchase order is to a vendor, on
    /// the side of [`Side::Payable`]. `None` for what is no order.
    #[must_use]
    pub const fn order_side(self) -> Option<Side> {
        match self {
            Self::SalesOrder => Some(Side::Receivable),
            Self::PurchaseOrder => Some(Side::Payable),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_kind_of_order_is_of_the_side_it_is_placed_on() {
        for side in [Side::Receivable, Side::Payable] {
            assert_eq!(side.order_kind().order_side(), Some(side));
        }
        assert_eq!(TransactionKind::Estimate.order_side(), None);
        assert_eq!(TransactionKind::Invoice.order_side(), None);
    }
}

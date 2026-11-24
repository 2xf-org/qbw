//! What a transaction has because of its kind.

use crate::ids::{AddressId, PaymentMethodId, SalesRepId, TermsId};
use crate::transactions::forms::{
    Bill, BillPayment, Build, CheckForm, InventoryAdjustment, JournalForm, Payment, PurchaseOrder,
    Refund, Sale, Transfer,
};

/// What a transaction has because of its kind.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Details {
    /// An invoice, a sales receipt, a credit memo, an estimate, a sales order
    /// or a statement charge.
    Sale(Sale),
    /// A payment from a customer.
    Payment(Payment),
    /// A refund to a customer. Unchecked.
    Refund(Refund),
    /// A bill, a vendor credit or an item receipt.
    Bill(Bill),
    /// A purchase order.
    PurchaseOrder(PurchaseOrder),
    /// Bills paid by check or by credit card.
    BillPayment(BillPayment),
    /// A check, a payment of sales tax or a paycheck.
    Check(CheckForm),
    /// A purchase by credit card, or a refund to one. It has nothing more than
    /// every transaction has.
    CardCharge,
    /// A deposit, which has nothing more. Cash that was kept back is a line.
    Deposit,
    /// A transfer between two accounts.
    Transfer(Transfer),
    /// A journal entry.
    Journal(JournalForm),
    /// A change of the quantities or the values in stock.
    InventoryAdjustment(InventoryAdjustment),
    /// Stock moved between two sites, which has nothing more. A line takes it
    /// from one and another brings it to the other.
    InventoryTransfer,
    /// A build of an assembly.
    Build(Build),
    /// A kind that this version knows nothing more of.
    Other,
}

impl Details {
    pub(super) fn terms(&self) -> Option<TermsId> {
        match self {
            Self::Sale(sale) => sale.terms,
            Self::Bill(bill) => bill.terms,
            Self::PurchaseOrder(order) => order.terms,
            _ => None,
        }
    }

    pub(super) fn sales_rep(&self) -> Option<SalesRepId> {
        match self {
            Self::Sale(sale) => sale.sales_rep,
            _ => None,
        }
    }

    pub(super) fn ship_to(&self) -> Option<AddressId> {
        match self {
            Self::Sale(sale) => sale.ship_to,
            Self::PurchaseOrder(order) => order.ship_to,
            _ => None,
        }
    }

    pub(super) fn payment_method(&self) -> Option<PaymentMethodId> {
        match self {
            Self::Sale(sale) => sale.payment_method,
            Self::Payment(payment) => payment.payment_method,
            Self::Refund(refund) => refund.payment_method,
            _ => None,
        }
    }
}

//! The signs of a form.

use std::ops::Neg;

use crate::codes::TransactionKind;

/// The side of the books that the own line of a form is on.
///
/// The file keeps debits as positive amounts. A form shows its total and its
/// lines as positive amounts, whatever side they are on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FormSide {
    /// The own line is a debit and the others are credits, as on an invoice.
    Debit,
    /// The own line is a credit and the others are debits, as on a bill.
    Credit,
    /// The lines are on either side, as in a journal entry, and the form shows
    /// them as they are.
    Either,
}

impl FormSide {
    /// The side of the forms of `kind`. Sales receipts, refunds and payroll
    /// are unchecked.
    pub(crate) const fn of(kind: TransactionKind) -> Self {
        match kind {
            TransactionKind::Invoice
            | TransactionKind::Estimate
            | TransactionKind::SalesOrder
            | TransactionKind::SalesReceipt
            | TransactionKind::StatementCharge
            | TransactionKind::Payment
            | TransactionKind::Deposit
            | TransactionKind::CreditCardCredit
            | TransactionKind::VendorCredit
            | TransactionKind::AssemblyBuild => Self::Debit,
            TransactionKind::CreditMemo
            | TransactionKind::CustomerRefund
            | TransactionKind::CreditCardCharge
            | TransactionKind::Check
            | TransactionKind::SalesTaxPayment
            | TransactionKind::Paycheck
            | TransactionKind::PayrollLiabilityCheck
            | TransactionKind::Bill
            | TransactionKind::ItemReceipt
            | TransactionKind::BillPaymentCheck
            | TransactionKind::BillPaymentCreditCard
            | TransactionKind::PurchaseOrder
            | TransactionKind::Transfer => Self::Credit,
            _ => Self::Either,
        }
    }

    /// The total that the form shows for `own`, the amount of its own line.
    pub(crate) fn total<T: Neg<Output = T>>(self, own: T) -> Option<T> {
        match self {
            Self::Debit => Some(own),
            Self::Credit => Some(-own),
            Self::Either => None,
        }
    }

    /// What the form shows for `stored`, the amount or the quantity of a line, or
    /// the other way round.
    pub(crate) fn shown<T: Neg<Output = T>>(self, stored: T) -> T {
        match self {
            Self::Debit => -stored,
            Self::Credit | Self::Either => stored,
        }
    }

    /// Whether a line takes its item out of the stock. Those of a form that is a
    /// debit do, such as an invoice's. The own line of one puts something into
    /// it: the assembly of a build, whose other lines are what it is made of.
    pub(crate) const fn takes_out(self, is_own: bool) -> bool {
        match self {
            Self::Debit => !is_own,
            Self::Credit => is_own,
            Self::Either => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_form_shows_both_sides_as_positive() {
        let invoice = FormSide::of(TransactionKind::Invoice);
        assert_eq!((invoice.total(202), invoice.shown(-150)), (Some(202), 150));
        let bill = FormSide::of(TransactionKind::Bill);
        assert_eq!((bill.total(-200), bill.shown(200)), (Some(200), 200));
        let journal = FormSide::of(TransactionKind::JournalEntry);
        assert_eq!((journal.total(1000), journal.shown(-1000)), (None, -1000));
    }

    #[test]
    fn a_build_takes_out_its_parts_and_puts_in_what_is_built() {
        let build = FormSide::of(TransactionKind::AssemblyBuild);
        assert!(build.takes_out(false) && !build.takes_out(true));
        let takes_out = |kind| FormSide::of(kind).takes_out(false);
        assert!(takes_out(TransactionKind::Invoice));
        assert!(takes_out(TransactionKind::CreditCardCredit));
        assert!(!takes_out(TransactionKind::CreditMemo));
        assert!(!takes_out(TransactionKind::Bill));
        assert!(!takes_out(TransactionKind::InventoryAdjustment));
    }

    #[test]
    fn what_is_given_back_is_on_the_other_side() {
        let sides = [
            (TransactionKind::Invoice, TransactionKind::CreditMemo),
            (
                TransactionKind::CreditCardCredit,
                TransactionKind::CreditCardCharge,
            ),
            (TransactionKind::VendorCredit, TransactionKind::Bill),
        ];
        for (debit, credit) in sides {
            assert_eq!(FormSide::of(debit), FormSide::Debit);
            assert_eq!(FormSide::of(credit), FormSide::Credit);
        }
    }
}

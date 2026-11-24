//! The kinds of payment methods.

use crate::code::codes;

codes! {
    /// The kind of a payment method.
    #[derive(Default)]
    pub enum PaymentMethodKind {
        /// Cash, or a method that the company added.
        #[default]
        CashOrOther = 0,
        /// A check.
        Check = 1,
        /// An American Express card.
        AmericanExpress = 2,
        /// A Discover card.
        Discover = 3,
        /// A MasterCard.
        MasterCard = 4,
        /// A Visa card.
        Visa = 5,
        /// A debit card.
        DebitCard = 8,
        /// A gift card.
        GiftCard = 9,
        /// A check sent electronically.
        ElectronicCheck = 10,
    }
}

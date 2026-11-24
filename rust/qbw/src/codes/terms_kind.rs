//! What payment terms count from.

use crate::code::codes;

codes! {
    /// What payment terms count from.
    #[derive(Default)]
    pub enum TermsKind {
        /// The days since the date of the invoice or bill.
        #[default]
        Standard = 0,
        /// A day of the month.
        DateDriven = 1,
    }
}

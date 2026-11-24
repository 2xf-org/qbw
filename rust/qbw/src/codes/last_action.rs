//! What was last done to a transaction.

use crate::code::codes;

codes! {
    /// What somebody last did to a transaction.
    pub enum LastAction {
        /// Entered it.
        Entered = 1,
        /// Changed it. A transaction that was voided and changed after that
        /// has this, too.
        Edited = 2,
        /// Voided it: it keeps its place in the books, with amounts of zero.
        Voided = 3,
    }
}

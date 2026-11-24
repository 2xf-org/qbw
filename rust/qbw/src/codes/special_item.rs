//! The items that QuickBooks makes for itself.

use crate::code::codes;

codes! {
    /// What QuickBooks uses an item for that it made itself.
    ///
    /// Both were seen, and what they are for is inferred from their kinds.
    pub enum SpecialItem {
        /// The group that costs charged on to a customer are billed under.
        ReimbursableGroup = 1,
        /// The subtotal of such a group.
        ReimbursableSubtotal = 3,
    }
}

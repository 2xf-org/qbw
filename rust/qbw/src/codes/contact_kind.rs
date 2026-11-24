//! Which of a name's contacts one is.

use crate::code::codes;

codes! {
    /// Which of a name's contacts one is.
    ///
    /// Every name has a main contact, and at most one of each of the others.
    /// What tells those two apart is inferred from that and from what
    /// QuickBooks calls them.
    #[derive(Default)]
    pub enum ContactKind {
        /// The name's own: its address, and who to ask for.
        #[default]
        Main = 0,
        /// A second person to ask for.
        Secondary = 1,
        /// A further person to ask for.
        Additional = 2,
    }
}

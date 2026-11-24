//! Whether a bank has confirmed a line.

use crate::code::{codes, words};

codes! {
    /// Whether a line of a bank or a card account was found on its statement.
    #[derive(Default)]
    pub enum Cleared {
        /// Not yet.
        #[default]
        NotCleared = 0,
        /// Ticked off, in a reconciliation that may not be finished.
        Ticked = 1,
        /// Part of a finished reconciliation.
        Reconciled = 2,
        /// Matched with what the bank sent.
        MatchedByBankFeed = 3,
    }
}

words! {
    /// Writes the state in QuickBooks' words, which its registers mark with a sign.
    Cleared, a "state of a line on a statement", or else "State" {
        NotCleared => "Not cleared",
        Ticked => "Cleared",
        Reconciled => "Reconciled",
        MatchedByBankFeed => "Matched",
    }
}

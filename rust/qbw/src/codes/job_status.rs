//! How far a job is.

use crate::code::codes;

codes! {
    /// How far a job is.
    ///
    /// The file keeps a number for each of the statuses that QuickBooks
    /// offers. Which number stands for which is not known.
    #[derive(Default)]
    pub enum JobStatus {
        /// No status was chosen.
        #[default]
        None = 0,
    }
}

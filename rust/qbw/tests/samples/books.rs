//! The company files, opened once for the tests that ask them little.

use std::sync::OnceLock;

use qbw::Company;

use crate::support;

/// Every company file of the folder. The tests that take it share what the
/// companies have read.
pub(crate) fn books() -> &'static [Company] {
    static BOOKS: OnceLock<Vec<Company>> = OnceLock::new();
    BOOKS.get_or_init(|| support::samples().collect())
}

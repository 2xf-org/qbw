//! The answer of a check.

use crate::check::{CheckKind, Finding};
use crate::left_out::LeftOut;

/// How what is worked out from the books agrees with what QuickBooks keeps, and
/// with the rest of the books.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Check {
    /// How often figures agree, for each kind that the file has figures of.
    pub compared: Vec<Compared>,
    /// Where they do not, in the order of the kinds.
    pub findings: Vec<Finding>,
    /// What could not be compared, since the file lacks a table for it.
    pub not_compared: Vec<NotCompared>,
    /// How much could not be read.
    pub left_out: LeftOut,
}

/// Figures that could not be compared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct NotCompared {
    /// What was not compared.
    pub what: CheckKind,
    /// The table that the file lacks.
    pub table: &'static str,
}

/// How often figures of one kind agree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Compared {
    /// What was compared.
    pub what: CheckKind,
    /// How many figures there are.
    pub looked_at: usize,
    /// How many of them agree.
    pub agreed: usize,
}

impl Check {
    /// How often figures of one kind agree. `None` where the file has none.
    #[must_use]
    pub fn get(&self, what: CheckKind) -> Option<Compared> {
        let mut compared = self.compared.iter();
        compared.find(|compared| compared.what == what).copied()
    }

    /// The findings of one kind.
    pub fn of(&self, what: CheckKind) -> impl Iterator<Item = &Finding> {
        self.findings.iter().filter(move |found| found.what == what)
    }

    /// The findings whose causes do not explain all of the difference. A company
    /// whose books are in order can have some, such as a total that QuickBooks
    /// has not brought up to date.
    pub fn unexplained(&self) -> impl Iterator<Item = &Finding> {
        let findings = self.findings.iter();
        findings.filter(|found| !found.is_explained())
    }
}

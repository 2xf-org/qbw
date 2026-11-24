//! Counting comparisons.

use std::collections::{BTreeMap, HashSet};

use crate::check::{Cause, Check, CheckKind, Compared, Figure, Finding, NotCompared, Subject};
use crate::damage::{Damage, Place};
use crate::left_out::LeftOut;

/// What is worked out, and what it should be.
#[derive(Clone, Copy, Debug)]
pub(super) struct Figures {
    pub(super) computed: Option<Figure>,
    pub(super) expected: Option<Figure>,
}

impl Figures {
    pub(super) fn new(computed: impl Into<Figure>, expected: impl Into<Figure>) -> Self {
        Self {
            computed: Some(computed.into()),
            expected: Some(expected.into()),
        }
    }
}

/// The comparisons that were made so far.
#[derive(Debug, Default)]
pub(crate) struct Tally {
    compared: BTreeMap<CheckKind, Compared>,
    findings: Vec<Finding>,
    not_compared: Vec<NotCompared>,
    /// Where something could not be read.
    lost: HashSet<Place>,
    /// What else is left out.
    left_out: LeftOut,
}

impl Tally {
    /// Compares two figures. The causes of a difference are asked for when there
    /// is one.
    pub(super) fn compare(
        &mut self,
        what: CheckKind,
        subject: Subject,
        Figures { computed, expected }: Figures,
        causes: impl FnOnce() -> Vec<Cause>,
    ) {
        if self.count(what, computed == expected) {
            return;
        }
        self.findings.push(Finding {
            what,
            subject,
            computed,
            expected,
            causes: causes(),
        });
    }

    /// Counts what should be so, whatever its figures are.
    pub(super) fn expect(&mut self, what: CheckKind, subject: Subject, is_so: bool) {
        if !self.count(what, is_so) {
            self.findings.push(Finding {
                what,
                subject,
                computed: None,
                expected: None,
                causes: Vec::new(),
            });
        }
    }

    /// Adds that `kinds` cannot be compared, since the file lacks `table`.
    pub(crate) fn skip(&mut self, kinds: &[CheckKind], table: &'static str) {
        let skipped = kinds.iter().map(|&what| NotCompared { what, table });
        self.not_compared.extend(skipped);
    }

    /// Adds what could not be read. What several parts lack counts once.
    pub(crate) fn lose<'damage>(&mut self, damage: impl IntoIterator<Item = &'damage Damage>) {
        self.lost.extend(damage.into_iter().map(Damage::place));
    }

    /// Adds what is left out, though it can be read.
    pub(crate) fn leave_out(&mut self, left_out: LeftOut) {
        self.left_out = self.left_out.with(left_out);
    }

    /// Whether everything of one kind that was compared so far agrees.
    pub(super) fn all_agree(&self, what: CheckKind) -> bool {
        let compared = self.compared.get(&what);
        compared.is_none_or(|compared| compared.agreed == compared.looked_at)
    }

    pub(crate) fn into_check(self) -> Check {
        let mut findings = self.findings;
        findings.sort_by_key(|found| found.what);
        Check {
            compared: self.compared.into_values().collect(),
            findings,
            not_compared: self.not_compared,
            left_out: LeftOut::of_places(self.lost).with(self.left_out),
        }
    }

    /// Counts a comparison, and says again whether it agreed.
    fn count(&mut self, what: CheckKind, agreed: bool) -> bool {
        let compared = self.compared.entry(what).or_insert(Compared {
            what,
            looked_at: 0,
            agreed: 0,
        });
        compared.looked_at += 1;
        compared.agreed += usize::from(agreed);
        agreed
    }
}

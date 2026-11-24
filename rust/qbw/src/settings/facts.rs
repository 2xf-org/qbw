//! The figures that QuickBooks keeps about the company.

use sa17::Source;

use crate::damage::Damage;
use crate::error::Result;
use crate::left_out::LeftOut;
use crate::settings::Fact;
use crate::tables::information::FACTS;

/// The figures that QuickBooks worked out for itself, to be found by key.
#[derive(Clone, Debug)]
pub struct Facts {
    /// Ordered by key.
    facts: Vec<Fact>,
    damage: Vec<Damage>,
}

impl Facts {
    /// The fact called `key`.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Fact> {
        let place = self
            .facts
            .binary_search_by(|fact| fact.key.as_str().cmp(key));
        self.facts.get(place.ok()?)
    }

    /// The facts, ordered by key.
    pub fn iter(&self) -> std::slice::Iter<'_, Fact> {
        self.facts.iter()
    }

    /// The number of facts.
    #[must_use]
    pub fn len(&self) -> usize {
        self.facts.len()
    }

    /// Whether there are no facts.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }

    /// How much could not be read.
    #[must_use]
    pub fn left_out(&self) -> LeftOut {
        LeftOut::of(&self.damage)
    }

    /// What could not be read.
    #[must_use]
    pub fn damage(&self) -> &[Damage] {
        &self.damage
    }
}

impl<'facts> IntoIterator for &'facts Facts {
    type Item = &'facts Fact;
    type IntoIter = std::slice::Iter<'facts, Fact>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Reads the facts, of which a file may have none.
///
/// # Errors
///
/// Returns an error when the table of them lacks a column.
pub(crate) fn read(source: &dyn Source) -> Result<Facts> {
    let mut damage = Vec::new();
    let mut facts: Vec<Fact> = match FACTS.find(source)? {
        Some(table) => table
            .scan(&mut damage)
            .filter_map(|(origin, row)| {
                Some(Fact {
                    key: row.key?,
                    value: row.value?,
                    origin,
                })
            })
            .collect(),
        None => Vec::new(),
    };
    facts.sort_by(|left, right| left.key.cmp(&right.key));
    Ok(Facts { facts, damage })
}

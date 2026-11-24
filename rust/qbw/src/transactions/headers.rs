//! The headers of every transaction.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use sa17::Source;

use crate::damage::{Damage, Problem};
use crate::error::Result;
use crate::ids::TransactionId;
use crate::tables::Form;
use crate::tables::headers::HeaderKeyRow;
use crate::transactions::Header;

/// The headers of every transaction, to find transactions by.
#[derive(Debug)]
pub(crate) struct Headers {
    /// Form by form, and those of one form as they are stored.
    headers: Vec<Header>,
    places: HashMap<TransactionId, usize>,
    /// What could not be read of each table that is damaged.
    damage: HashMap<&'static str, Vec<Damage>>,
}

impl Headers {
    /// Reads the headers of `source`.
    ///
    /// # Errors
    ///
    /// Returns an error when a table lacks a column that it has in every file.
    pub(crate) fn read(source: &dyn Source) -> Result<Self> {
        let mut headers: Vec<Header> = Vec::new();
        let mut places = HashMap::new();
        let mut damage = HashMap::new();
        for form in Form::all() {
            let Some(table) = form.headers_in::<HeaderKeyRow>(source)? else {
                continue;
            };
            let mut unread = Vec::new();
            let mut repeated = Vec::new();
            for (origin, row) in table.scan(&mut unread) {
                let header = Header::new(form, origin.row, row);
                match places.entry(header.id) {
                    Entry::Vacant(place) => {
                        place.insert(headers.len());
                        headers.push(header);
                    }
                    Entry::Occupied(_) => repeated.push(Damage::repeated(origin)),
                }
            }
            unread.append(&mut repeated);
            if !unread.is_empty() {
                damage.insert(form.headers::<HeaderKeyRow>().name(), unread);
            }
        }
        Ok(Self {
            headers,
            places,
            damage,
        })
    }

    pub(crate) fn get(&self, id: TransactionId) -> Option<&Header> {
        self.headers.get(*self.places.get(&id)?)
    }

    /// The headers form by form, and those of one form as they are stored.
    pub(crate) fn iter(&self) -> impl Iterator<Item = &Header> {
        self.headers.iter()
    }

    /// What could not be read of the headers of `form`.
    pub(crate) fn damage(&self, form: Form) -> &[Damage] {
        let damage = self.damage.get(form.headers::<HeaderKeyRow>().name());
        damage.map(Vec::as_slice).unwrap_or_default()
    }

    /// What could not be read of the headers of `form` and lost one.
    pub(super) fn unread(&self, form: Form) -> impl Iterator<Item = &Damage> {
        let damage = self.damage(form).iter();
        damage.filter(|damage| damage.problem != Problem::Repeated)
    }

    /// What is wrong with a header of `form` that is not among the others.
    /// `None` when all could be read.
    pub(crate) fn lost(&self, form: Form) -> Option<Damage> {
        let mut lost = self.unread(form);
        let first = lost.next()?;
        Some(match lost.next() {
            None => first.clone(),
            // Nothing tells which of them it is.
            Some(_) => Damage::new(first.table, None, Problem::Header),
        })
    }
}

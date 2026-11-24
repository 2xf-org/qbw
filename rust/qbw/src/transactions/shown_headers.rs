//! The headers that an answer shows something of.

use std::collections::HashSet;

use crate::codes::TransactionKind;
use crate::damage::Damage;
use crate::ids::TransactionId;
use crate::left_out::LeftOut;
use crate::tables::{FORMS, Form};
use crate::transactions::{Header, Headers};

/// The headers that an answer shows something of, such as the numbers of its
/// transactions, with what it has to do without.
#[derive(Debug)]
pub(crate) struct ShownHeaders<'company> {
    headers: &'company Headers,
    /// The forms that a header which is not among the others may be one of, where
    /// some could not be read.
    damaged: [bool; FORMS],
    /// The transactions without a header, where every one could be read that it
    /// may have had.
    absent: HashSet<TransactionId>,
}

impl<'company> ShownHeaders<'company> {
    pub(crate) fn new(headers: &'company Headers) -> Self {
        Self {
            headers,
            damaged: [false; FORMS],
            absent: HashSet::new(),
        }
    }

    /// The header of a transaction of `kind`.
    pub(crate) fn of_kind(
        &mut self,
        id: TransactionId,
        kind: TransactionKind,
    ) -> Option<&'company Header> {
        self.find(id, |form| form.can_hold(kind))
    }

    /// The header of a transaction of which nothing says what kind it is.
    pub(crate) fn of_any_kind(&mut self, id: TransactionId) -> Option<&'company Header> {
        self.find(id, |_| true)
    }

    fn find(
        &mut self,
        id: TransactionId,
        can_hold: impl Fn(Form) -> bool,
    ) -> Option<&'company Header> {
        let header = self.headers.get(id);
        if header.is_none() {
            let forms = Form::all().filter(|&form| can_hold(form));
            let mut is_explained = false;
            for form in forms.filter(|&form| self.headers.unread(form).next().is_some()) {
                is_explained = true;
                self.damaged[form.place()] = true;
            }
            if !is_explained {
                self.absent.insert(id);
            }
        }
        header
    }

    /// Adds the headers that `other` has to do without.
    pub(crate) fn join(&mut self, other: Self) {
        for (damaged, other) in self.damaged.iter_mut().zip(other.damaged) {
            *damaged |= other;
        }
        self.absent.extend(other.absent);
    }

    /// How much could not be read of the headers that were looked for. One that
    /// is not among the others can be any of those that could not be read.
    pub(crate) fn left_out(&self) -> LeftOut {
        let forms = Form::all().filter(|form| self.damaged[form.place()]);
        let unread = forms.flat_map(|form| self.headers.unread(form));
        let absent = self.absent.iter().map(|_| 1).sum();
        LeftOut::of_places(unread.map(Damage::place)).with(LeftOut::of_rows(absent))
    }
}

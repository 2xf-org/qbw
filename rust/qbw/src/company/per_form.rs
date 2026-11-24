//! A piece for each form.

use std::array;

use crate::company::pieces::Piece;
use crate::tables::{FORMS, Form};

/// A piece for each form of transaction.
#[derive(Debug)]
pub(super) struct PerForm<T>([Piece<T>; FORMS]);

impl<T> PerForm<T> {
    /// The piece for `form`.
    pub(super) fn of(&self, form: Form) -> &Piece<T> {
        &self.0[form.place()]
    }
}

impl<T> Default for PerForm<T> {
    fn default() -> Self {
        Self(array::from_fn(|_| Piece::default()))
    }
}

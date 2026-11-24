//! Rows, read into structs.

use std::slice;

use sa17::{Row, Value, ValueProblem};

use crate::damage::Problem;
use crate::tables::Field;

/// A row of a table, or of several tables of one shape, with the columns that
/// this crate reads. `stored_row!` declares one.
pub(crate) trait StoredRow: Sized {
    /// The columns that every such table has.
    const REQUIRED: &'static [&'static str];

    /// The columns that some such tables, or those of some releases, lack.
    const OPTIONAL: &'static [&'static str];

    /// Reads the values of [`Self::REQUIRED`] and then of [`Self::OPTIONAL`].
    ///
    /// # Errors
    ///
    /// Returns the first column that does not hold what it has to.
    fn read(fields: &mut Fields<'_>) -> Result<Self, Problem>;
}

/// The values of a row, one after the other.
pub(crate) struct Fields<'row> {
    values: slice::Iter<'row, Value>,
}

impl<'row> Fields<'row> {
    pub(crate) fn new(row: &'row Row) -> Self {
        Self {
            values: row.values().iter(),
        }
    }

    /// The next value, which is the one of `column`.
    ///
    /// # Errors
    ///
    /// Returns how the value differs from what the type takes.
    pub(crate) fn next<T: Field>(&mut self, column: &'static str) -> Result<T, Problem> {
        let value = self.values.next().ok_or(ValueProblem::Missing);
        let read = value.and_then(T::read);
        read.map_err(|found| Problem::Column { column, found })
    }
}

/// Declares a row: a struct whose fields name the columns they are read from.
///
/// Every table of such rows has the columns of `required`. A column of
/// `optional` reads as nothing where the table lacks it.
macro_rules! stored_row {
    (
        $(#[$meta:meta])*
        pub(crate) struct $name:ident {
            required {
                $($(#[$required_meta:meta])* $required:ident: $required_type:ty = $required_column:literal,)*
            }
            optional {
                $($(#[$optional_meta:meta])* $optional:ident: $optional_type:ty = $optional_column:literal,)*
            }
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        pub(crate) struct $name {
            $($(#[$required_meta])* pub(crate) $required: $required_type,)*
            $($(#[$optional_meta])* pub(crate) $optional: $optional_type,)*
        }

        impl $crate::tables::StoredRow for $name {
            const REQUIRED: &'static [&'static str] = &[$($required_column),*];
            const OPTIONAL: &'static [&'static str] = &[$($optional_column),*];

            fn read(
                fields: &mut $crate::tables::Fields<'_>,
            ) -> ::std::result::Result<Self, $crate::damage::Problem> {
                // The order of the fields is the order of the columns above.
                Ok(Self {
                    $($required: fields.next($required_column)?,)*
                    $($optional: fields.next($optional_column)?,)*
                })
            }
        }
    };
}

pub(crate) use stored_row;

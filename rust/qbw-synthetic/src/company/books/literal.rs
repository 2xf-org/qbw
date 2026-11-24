//! Values as they are typed into the books.

/// A value that is typed into a row. What it stands for depends on the type
/// of its column: a text is a number in a `numeric` column and a day in a
/// `date` column.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Literal {
    Null,
    Whole(i32),
    Flag(bool),
    Text(&'static str),
    /// The bytes of a text that is not in the file's encoding.
    Bytes(&'static [u8]),
}

impl From<i32> for Literal {
    fn from(whole: i32) -> Self {
        Self::Whole(whole)
    }
}

impl From<bool> for Literal {
    fn from(flag: bool) -> Self {
        Self::Flag(flag)
    }
}

impl From<&'static str> for Literal {
    fn from(text: &'static str) -> Self {
        Self::Text(text)
    }
}

/// The values of a row by the names of their columns. A column that is not
/// among them has its default.
pub(crate) type Values = Vec<(&'static str, Literal)>;

/// Writes [`Values`] down: `row! { "name" => "Checking", "type" => 0 }`.
macro_rules! row {
    ($($column:literal => $value:expr),* $(,)?) => {
        vec![$(($column, $crate::company::books::literal::Literal::from($value))),*]
    };
}

pub(crate) use row;

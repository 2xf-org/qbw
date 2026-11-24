//! Enums over the numbers that the file stores for a choice.

use crate::case;

/// Declares an enum for a choice that the file stores as a number.
///
/// Each variant has the number that stands for it. A number without a variant
/// is kept in `Other`, so that a file of a later release reads and the number
/// is never lost.
macro_rules! codes {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $($(#[$variant_meta:meta])* $variant:ident = $code:literal,)+
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        pub enum $name {
            $($(#[$variant_meta])* $variant,)+
            /// A choice that this version has no name for, as the file stores it.
            Other(i64),
        }

        impl $name {
            /// The choice that the file stores as `code`.
            #[must_use]
            pub const fn from_code(code: i64) -> Self {
                match code {
                    $($code => Self::$variant,)+
                    code => Self::Other(code),
                }
            }

            /// The number that the file stores for the choice.
            #[must_use]
            pub const fn code(self) -> i64 {
                match self {
                    $(Self::$variant => $code,)+
                    Self::Other(code) => code,
                }
            }
        }

        impl $crate::number::FromNumber for $name {
            fn from_number(number: i64) -> Option<Self> {
                Some(Self::from_code(number))
            }
        }
    };
}

/// Gives the choices of an enum of [`codes`] the words that they are printed in,
/// with a list of them and a way back from the words.
macro_rules! words {
    (
        $(#[$meta:meta])*
        $name:ident, a $what:literal, or else $other:literal {
            $($variant:ident => $words:literal,)+
        }
    ) => {
        impl $name {
            /// Every choice that has a name, in the order of the enum.
            pub const ALL: &'static [Self] = &[$(Self::$variant,)+];
        }

        impl std::fmt::Display for $name {
            $(#[$meta])*
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.pad(match self {
                    $(Self::$variant => $words,)+
                    Self::Other(code) => return write!(formatter, "{} {code}", $other),
                })
            }
        }

        impl std::str::FromStr for $name {
            type Err = $crate::code::UnknownWords;

            /// The choice that prints itself as `text`, ignoring case and what
            /// stands between the words. The one of a number without a name is
            /// among them.
            fn from_str(text: &str) -> Result<Self, Self::Err> {
                let mut named = Self::ALL.iter().copied();
                let named = named.find(|named| {
                    $crate::code::are_same_words(&named.to_string(), text)
                });
                let numbered = || $crate::code::number_after($other, text);
                named
                    .or_else(|| numbered().map(Self::from_code))
                    .ok_or_else(|| $crate::code::UnknownWords::new($what, text))
            }
        }
    };
}

pub(crate) use {codes, words};

/// Words that no choice prints itself in, such as `Receipt` for the kind of a
/// transaction.
/// [`TransactionKind::ALL`](crate::transactions::TransactionKind::ALL) and its
/// like have the choices.
///
/// ```
/// use qbw::transactions::TransactionKind;
///
/// let found: TransactionKind = "bill-pmt-check".parse()?;
/// assert_eq!(found, TransactionKind::BillPaymentCheck);
/// assert_eq!(found.to_string(), "Bill Pmt -Check");
///
/// let unknown = "Receipt".parse::<TransactionKind>();
/// let says = unknown.map_err(|unknown| unknown.to_string());
/// let expected = "there is no kind of transaction called \"Receipt\"";
/// assert_eq!(says, Err(expected.to_owned()));
/// # Ok::<(), qbw::UnknownWords>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("there is no {what} called {words:?}")]
#[non_exhaustive]
pub struct UnknownWords {
    /// What was asked for, such as `kind of transaction`.
    pub what: &'static str,
    /// The words.
    pub words: String,
}

impl UnknownWords {
    pub(crate) fn new(what: &'static str, words: &str) -> Self {
        Self {
            what,
            words: words.to_owned(),
        }
    }
}

/// Whether two texts are of the same words, ignoring case and what stands between
/// them.
pub(crate) fn are_same_words(left: &str, right: &str) -> bool {
    let [left, right] = [left, right].map(case::folded);
    words_of(&left).eq(words_of(&right))
}

/// The number in a text such as `Transaction 99`, where `name` is what goes
/// before it.
pub(crate) fn number_after(name: &str, text: &str) -> Option<i64> {
    let (before, number) = text.trim().rsplit_once(char::is_whitespace)?;
    let number = number.parse().ok()?;
    are_same_words(before, name).then_some(number)
}

fn words_of(text: &str) -> impl Iterator<Item = &str> {
    let words = text.split(|letter: char| !letter.is_alphanumeric());
    words.filter(|word| !word.is_empty())
}

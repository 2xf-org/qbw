//! The ids that the file stores as text.

use std::fmt;

/// The longest key that the file has room for.
const LONGEST: usize = 17;

/// A short text, kept in place.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Key {
    // Zero past the text, so that equal keys have equal bytes.
    bytes: [u8; LONGEST],
    length: u8,
}

impl Key {
    fn try_new(text: &str) -> Option<Self> {
        let mut bytes = [0; LONGEST];
        bytes
            .get_mut(..text.len())?
            .copy_from_slice(text.as_bytes());
        let length = u8::try_from(text.len()).ok().filter(|&length| length > 0)?;
        Some(Self { bytes, length })
    }

    fn as_str(&self) -> &str {
        let text = self.bytes.get(..self.length.into());
        // The bytes are those of a text.
        text.and_then(|text| str::from_utf8(text).ok())
            .unwrap_or_default()
    }
}

macro_rules! text_ids {
    ($($(#[$doc:meta])* $name:ident,)+) => {$(
        $(#[$doc])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(Key);

        impl $name {
            /// The id that the file stores as `text`, or `None` for a text
            /// that is empty or longer than the file has room for.
            #[must_use]
            pub fn try_new(text: &str) -> Option<Self> {
                Key::try_new(text).map(Self)
            }

            /// The text that the file stores.
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.pad(self.as_str())
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.debug_tuple(stringify!($name)).field(&self.as_str()).finish()
            }
        }
    )+};
}

text_ids! {
    /// An address that goods are shipped to, as it was when a form used it.
    AddressId,
    /// A name's address and phone numbers, as they were when a form used them.
    ContactId,
    /// A record that custom fields belong to: a name, an item, a transaction
    /// or a line.
    OwnerKey,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_id_keeps_the_text_that_the_file_stores() {
        let key = OwnerKey::try_new("0000000000000001").unwrap();
        assert_eq!(key.as_str(), "0000000000000001");
        assert_eq!(
            format!("{key}|{key:?}"),
            "0000000000000001|OwnerKey(\"0000000000000001\")"
        );
        assert_eq!(OwnerKey::try_new("0000000000000001"), Some(key));
        assert_ne!(OwnerKey::try_new("000000000000001"), Some(key));
    }

    #[test]
    fn an_id_is_neither_empty_nor_longer_than_its_column() {
        assert_eq!(ContactId::try_new(""), None);
        assert!(ContactId::try_new("12345678901234567").is_some());
        assert_eq!(ContactId::try_new("123456789012345678"), None);
        assert_eq!(ContactId::try_new("\u{e9}").unwrap().as_str(), "\u{e9}");
    }
}

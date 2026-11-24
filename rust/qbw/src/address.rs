//! Postal addresses.

/// A postal address.
///
/// QuickBooks keeps an address twice: as the lines that print on a form, which
/// start with a name, and in its parts. Either can be empty.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Address {
    /// The lines as they print on a form.
    pub printed: Vec<String>,
    /// The lines of the street.
    pub street: Vec<String>,
    /// The city.
    pub city: Option<String>,
    /// The state or province.
    pub state: Option<String>,
    /// The postal code.
    pub postal_code: Option<String>,
    /// The country.
    pub country: Option<String>,
}

impl Address {
    /// Whether nothing is known of the address.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// The lines that have something, in order.
    pub(crate) fn lines<const LINES: usize>(lines: [Option<String>; LINES]) -> Vec<String> {
        lines.into_iter().flatten().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_address_keeps_the_lines_that_have_something() {
        let line = |text: &str| Some(text.to_owned());
        let lines = Address::lines([line("Dock 2"), None, line("Northport")]);
        assert_eq!(lines, ["Dock 2", "Northport"]);
        assert!(Address::default().is_empty());
        let address = Address {
            street: lines,
            ..Address::default()
        };
        assert!(!address.is_empty());
    }
}

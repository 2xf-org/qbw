//! Comparing texts whatever their case.

/// A text without its case, to compare it with another.
pub(crate) fn folded(text: &str) -> String {
    text.to_lowercase()
}

/// Whether two texts are the same but for their case.
pub(crate) fn is_same(left: &str, right: &str) -> bool {
    folded(left) == folded(right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_of_every_alphabet_have_two_cases() {
        assert!(is_same("Caf\u{e9}", "CAF\u{c9}"));
        assert!(!is_same("Cafe", "Caf\u{e9}"));
    }
}

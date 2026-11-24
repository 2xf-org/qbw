//! The order of numbers that are texts.

use std::cmp::Ordering;

use crate::case;

/// A text that is ordered by the numbers in it, as people order the numbers of
/// their invoices: `A-9` comes before `A-10`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Natural<'text>(pub(super) &'text str);

impl Ord for Natural<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        let (mut left, mut right) = (self.0, other.0);
        while !left.is_empty() && !right.is_empty() {
            let (left_run, left_rest) = split_run(left);
            let (right_run, right_rest) = split_run(right);
            let ordering = compare_runs(left_run, right_run);
            if ordering.is_ne() {
                return ordering;
            }
            (left, right) = (left_rest, right_rest);
        }
        // Texts that differ only in case or in zeros have an order, too.
        left.cmp(right).then_with(|| self.0.cmp(other.0))
    }
}

impl PartialOrd for Natural<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The digits that `text` starts with, or else what comes before its first
/// digit, and the rest.
fn split_run(text: &str) -> (&str, &str) {
    let is_number = starts_with_digit(text);
    let end = text.find(|character: char| character.is_ascii_digit() != is_number);
    text.split_at(end.unwrap_or(text.len()))
}

fn compare_runs(left: &str, right: &str) -> Ordering {
    match (starts_with_digit(left), starts_with_digit(right)) {
        (true, true) => {
            let (left, right) = (without_zeros(left), without_zeros(right));
            left.len().cmp(&right.len()).then_with(|| left.cmp(right))
        }
        (false, false) => {
            let lowercase = case::folded;
            lowercase(left).cmp(&lowercase(right))
        }
        (left_is_number, right_is_number) => right_is_number.cmp(&left_is_number),
    }
}

fn starts_with_digit(text: &str) -> bool {
    text.starts_with(|character: char| character.is_ascii_digit())
}

fn without_zeros(digits: &str) -> &str {
    digits.trim_start_matches('0')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_in_texts_count_as_numbers() {
        let mut numbers = [
            "A-10", "b-2", "A-9", "1042", "99", "B-1", "A", "", "A-9b", "007", "7",
        ];
        numbers.sort_unstable_by_key(|&number| Natural(number));
        assert_eq!(
            numbers,
            [
                "", "007", "7", "99", "1042", "A", "A-9", "A-9b", "A-10", "B-1", "b-2"
            ]
        );
    }

    #[test]
    fn a_number_may_be_longer_than_any_integer() {
        let long = "9".repeat(60);
        let longer = format!("1{}", "0".repeat(60));
        assert!(Natural(&long) < Natural(&longer));
    }

    #[test]
    fn texts_that_differ_are_never_equal() {
        for (left, right) in [("a", "A"), ("01", "1"), ("\u{e9}1", "\u{c9}1")] {
            assert_ne!(Natural(left).cmp(&Natural(right)), Ordering::Equal);
        }
    }
}

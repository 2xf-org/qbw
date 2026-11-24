//! Which end of an order comes first.

use std::cmp::Ordering;

/// Which end of an order comes first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Direction {
    /// The oldest, the smallest, or the first of the alphabet.
    Ascending,
    /// The newest, the largest, or the last of the alphabet.
    Descending,
}

impl Direction {
    pub(super) const fn apply(self, ordering: Ordering) -> Ordering {
        match self {
            Self::Ascending => ordering,
            Self::Descending => ordering.reverse(),
        }
    }

    /// Orders two values of which either may be missing. What is missing comes
    /// last in both directions.
    pub(super) fn compare<T: Ord>(self, left: Option<T>, right: Option<T>) -> Ordering {
        match (left, right) {
            (Some(left), Some(right)) => self.apply(left.cmp(&right)),
            (left, right) => right.is_some().cmp(&left.is_some()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_is_missing_is_last_in_both_directions() {
        for (direction, expected) in [
            (Direction::Ascending, [Some(1), Some(2), None]),
            (Direction::Descending, [Some(2), Some(1), None]),
        ] {
            let mut numbers = [None, Some(2), Some(1)];
            numbers.sort_by(|left, right| direction.compare(*left, *right));
            assert_eq!(numbers, expected);
        }
    }
}

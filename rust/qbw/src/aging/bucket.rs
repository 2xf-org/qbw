//! A span of ages.

/// A span of ages in days, both ends included. The first bucket has everything
/// that is not late, and the last everything later than the others.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Bucket {
    /// The lowest age. `None` in the first bucket.
    pub first: Option<i64>,
    /// The highest age. `None` in the last bucket.
    pub last: Option<i64>,
}

impl Bucket {
    /// The buckets around `count` of `days` days each.
    pub(super) fn all(count: u16, days: u16) -> Vec<Self> {
        let (count, days) = (i64::from(count), i64::from(days));
        let current = Self {
            first: None,
            last: Some(0),
        };
        let between = (0..count).map(|place| Self {
            first: Some(place * days + 1),
            last: Some((place + 1) * days),
        });
        let late = Self {
            first: Some(count * days + 1),
            last: None,
        };
        let mut buckets = vec![current];
        buckets.extend(between);
        buckets.push(late);
        buckets
    }

    /// Whether it is the bucket of an age.
    #[must_use]
    pub fn contains(self, age: i64) -> bool {
        self.first.is_none_or(|first| first <= age) && self.last.is_none_or(|last| age <= last)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_age_is_in_one_bucket() {
        let buckets = Bucket::all(3, 30);
        assert_eq!(buckets.len(), 5);
        let place = |age| {
            let found = (0..)
                .zip(&buckets)
                .filter(|(_, bucket)| bucket.contains(age));
            let places: Vec<i32> = found.map(|(place, _)| place).collect();
            places
        };
        let ages = [
            (i64::MIN, 0),
            (0, 0),
            (1, 1),
            (30, 1),
            (31, 2),
            (90, 3),
            (91, 4),
            (i64::MAX, 4),
        ];
        for (age, expected) in ages {
            assert_eq!(place(age), [expected], "{age} days");
        }
    }

    #[test]
    fn there_can_be_no_bucket_between() {
        let buckets = Bucket::all(0, 30);
        let ends: Vec<_> = buckets
            .iter()
            .map(|bucket| (bucket.first, bucket.last))
            .collect();
        assert_eq!(ends, [(None, Some(0)), (Some(1), None)]);
    }
}

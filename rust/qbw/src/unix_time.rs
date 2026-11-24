//! Moments, as the file stores them.

use std::fmt;

use chrono::{DateTime, Utc};

use crate::number::FromNumber;

/// A moment, as seconds since 1970 in UTC. The file keeps when an entry or a
/// transaction was made and last changed this way.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnixTime(i64);

impl UnixTime {
    /// The moment `seconds` after the start of 1970.
    #[must_use]
    pub const fn new(seconds: i64) -> Self {
        Self(seconds)
    }

    /// The seconds since the start of 1970.
    #[must_use]
    pub const fn seconds(self) -> i64 {
        self.0
    }

    /// The moment as a date and a time in UTC, or `None` when it is outside
    /// the calendar.
    #[must_use]
    pub const fn to_utc(self) -> Option<DateTime<Utc>> {
        DateTime::from_timestamp(self.0, 0)
    }
}

impl FromNumber for UnixTime {
    fn from_number(number: i64) -> Option<Self> {
        Some(Self(number))
    }
}

impl fmt::Display for UnixTime {
    /// Writes the date and the time in UTC, such as `2024-05-01 09:30:00 UTC`.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_utc() {
            Some(time) => time.fmt(formatter),
            None => self.0.fmt(formatter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_moment_reads_in_utc() {
        let moment = UnixTime::new(1_714_555_800);
        assert_eq!(moment.to_string(), "2024-05-01 09:30:00 UTC");
        assert_eq!(UnixTime::from_number(1_714_555_800), Some(moment));
        assert_eq!(UnixTime::new(i64::MAX).to_utc(), None);
        assert_eq!(UnixTime::new(i64::MAX).to_string(), i64::MAX.to_string());
    }
}

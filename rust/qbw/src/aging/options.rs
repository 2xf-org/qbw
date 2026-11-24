//! How to age what is open.

use crate::as_of::AsOf;
use crate::settings::AgingBasis;

/// How to age what is open on a day: in how many buckets of how many days, from
/// which date, and with a row for what.
///
/// ```
/// use qbw::AsOf;
/// use qbw::aging::{AgingBy, AgingOptions};
/// use qbw::settings::AgingBasis;
///
/// let options = AgingOptions::new(AsOf::Latest)
///     .basis(AgingBasis::TransactionDate)
///     .try_buckets(4, 15)?
///     .by(AgingBy::Items);
/// assert!(options.try_buckets(4, 0).is_err());
/// # Ok::<(), qbw::aging::BucketsError>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AgingOptions {
    pub(crate) as_of: AsOf,
    pub(super) basis: Option<AgingBasis>,
    pub(super) buckets: u16,
    pub(super) days: u16,
    pub(super) by: AgingBy,
}

/// Buckets that [`AgingOptions::try_buckets`] refuses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error(
    "an aging has at most {} buckets of at least one day, not {count} of {days}",
    AgingOptions::MOST_BUCKETS
)]
#[non_exhaustive]
pub struct BucketsError {
    /// How many buckets were asked for.
    pub count: u16,
    /// How many days each was to have.
    pub days: u16,
}

/// What an aging has a row for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AgingBy {
    /// Each customer or vendor, as in the summary.
    Names,
    /// Each open item, as in the detail.
    Items,
}

impl AgingOptions {
    /// The most buckets an aging has between its first and its last.
    pub const MOST_BUCKETS: u16 = 366;

    /// What is open at `as_of`, aged as QuickBooks does unless it is told
    /// otherwise: from the date in the company's preferences, in three buckets of
    /// thirty days, with a row for each name.
    #[must_use]
    pub fn new(as_of: impl Into<AsOf>) -> Self {
        Self {
            as_of: as_of.into(),
            basis: None,
            buckets: 3,
            days: 30,
            by: AgingBy::Names,
        }
    }

    /// Ages from another date than the company's preferences say.
    #[must_use]
    pub const fn basis(self, basis: AgingBasis) -> Self {
        Self {
            basis: Some(basis),
            ..self
        }
    }

    /// Ages in `count` buckets of `days` days each, between what is not late
    /// and what is later than all of them.
    ///
    /// # Errors
    ///
    /// Returns an error when there are more buckets than [`Self::MOST_BUCKETS`],
    /// or they have no days.
    pub const fn try_buckets(self, count: u16, days: u16) -> Result<Self, BucketsError> {
        if count > Self::MOST_BUCKETS || days == 0 {
            return Err(BucketsError { count, days });
        }
        Ok(Self {
            buckets: count,
            days,
            ..self
        })
    }

    /// Gives a row for something else than each name.
    #[must_use]
    pub const fn by(self, by: AgingBy) -> Self {
        Self { by, ..self }
    }
}

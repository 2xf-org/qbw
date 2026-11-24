//! What a currency was worth.

use chrono::NaiveDate;

use crate::origin::Origin;
use crate::tables::lists::ExchangeRateRow;

/// What a currency was worth from a day on.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct ExchangeRate {
    /// The day.
    pub as_of: NaiveDate,
    /// What one of it was worth in the company's currency.
    pub rate: f32,
    /// The stored row.
    pub origin: Origin,
}

impl ExchangeRate {
    /// The rate of `row`, unless it lacks a day or a rate.
    pub(super) fn new(origin: Origin, row: &ExchangeRateRow) -> Option<Self> {
        Some(Self {
            as_of: row.as_of?,
            rate: row.rate?,
            origin,
        })
    }
}

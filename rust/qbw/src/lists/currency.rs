//! Currencies.

use std::collections::HashMap;

use chrono::NaiveDate;
use sa17::{Source, TextEncoding};

use crate::error::Result;
use crate::ids::CurrencyId;
use crate::list::{List, ListEntry, sealed};
use crate::lists::{ExchangeRate, reading};
use crate::origin::Origin;
use crate::tables::lists::{CURRENCIES, CurrencyRow, EXCHANGE_RATES};

/// A currency. The list has every one that QuickBooks knows, and those that
/// the company uses are active.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Currency {
    /// What the currency is known by.
    pub id: CurrencyId,
    /// Its name.
    pub name: String,
    /// Its code of ISO 4217, such as `EUR`.
    pub code: String,
    /// Its symbol, such as `$`.
    pub symbol: Option<String>,
    /// The digits after the point that its amounts are written with.
    pub decimal_places: Option<i32>,
    /// What it was worth, ordered by date.
    pub rates: Vec<ExchangeRate>,
    /// Whether the company uses it.
    pub is_active: bool,
    /// The stored row.
    pub origin: Origin,
}

impl Currency {
    /// What it was worth on `date`: the rate of that day, or of the last day
    /// before it that has one.
    #[must_use]
    pub fn rate_on(&self, date: NaiveDate) -> Option<&ExchangeRate> {
        let later = self.rates.partition_point(|rate| rate.as_of <= date);
        self.rates.get(..later)?.last()
    }
}

impl sealed::Sealed for Currency {}

impl ListEntry for Currency {
    type Id = CurrencyId;

    fn id(&self) -> CurrencyId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn origin(&self) -> Origin {
        self.origin
    }
}

/// Reads the currencies and what they were worth.
///
/// # Errors
///
/// Returns an error when a table of them lacks a column.
pub(crate) fn read(source: &dyn Source) -> Result<List<Currency>> {
    let mut damage = Vec::new();
    let mut rates: HashMap<CurrencyId, Vec<ExchangeRate>> = HashMap::new();
    for (origin, row) in reading::rows(source, &EXCHANGE_RATES, &mut damage)? {
        if let (false, Some(currency)) = (row.is_deleted, row.currency) {
            let rate = ExchangeRate::new(origin, &row);
            rates.entry(currency).or_default().extend(rate);
        }
    }
    let encoding = source.text_encoding();
    let rows = reading::rows(source, &CURRENCIES, &mut damage)?.into_iter();
    let currencies = rows
        .filter(|(_, row)| !row.is_deleted)
        .map(|(origin, row)| {
            let rates = rates.remove(&row.id).unwrap_or_default();
            currency(origin, row, rates, encoding)
        });
    Ok(reading::sorted_by_name(currencies.collect(), damage))
}

fn currency(
    origin: Origin,
    row: CurrencyRow,
    mut rates: Vec<ExchangeRate>,
    encoding: TextEncoding,
) -> Currency {
    rates.sort_by_key(|rate| rate.as_of);
    Currency {
        id: row.id,
        name: row.name.unwrap_or_default(),
        code: row.code.unwrap_or_default(),
        symbol: row.symbol.map(|symbol| from_utf8(symbol, encoding)),
        decimal_places: row.decimal_places,
        rates,
        is_active: !row.is_hidden,
        origin,
    }
}

/// The text whose bytes are UTF-8, of a column that is stored as `encoding`.
/// It stays as it is when they are not.
fn from_utf8(text: String, encoding: TextEncoding) -> String {
    let bytes = encoding.encode(&text);
    let repaired = bytes.and_then(|bytes| String::from_utf8(bytes).ok());
    repaired.unwrap_or(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(bytes: &[u8]) -> String {
        let encoding = TextEncoding::Windows1252;
        from_utf8(encoding.decode(bytes), encoding)
    }

    #[test]
    fn a_symbol_in_utf_8_is_read_as_such() {
        assert_eq!(read(b"\xe2\x82\xac"), "\u{20ac}");
        assert_eq!(read(b"\xc2\xa5"), "\u{a5}");
        assert_eq!(read(b"$"), "$");
    }

    #[test]
    fn a_symbol_that_is_not_in_utf_8_stays() {
        assert_eq!(read(b"\xa3"), "\u{a3}");
        assert_eq!(read(b"K\x10\xe8"), "K\u{10}\u{e8}");
        // A text in UTF-8 has nothing to repair.
        assert_eq!(
            from_utf8("\u{20ac}".to_owned(), TextEncoding::Utf8),
            "\u{20ac}"
        );
    }
}

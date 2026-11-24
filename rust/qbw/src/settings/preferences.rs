//! The company's preferences.

use std::collections::HashMap;

use chrono::Month;
use sa17::{Source, ValueProblem};

use crate::damage::{Damage, Problem};
use crate::error::{Error, Result};
use crate::ids::{CurrencyId, ItemId};
use crate::left_out::LeftOut;
use crate::origin::Origin;
use crate::settings::{AgingBasis, Preference};
use crate::tables::settings::{PREFERENCE_VALUE, PREFERENCE_VALUES, PREFERENCES, preference};

/// The preferences of the whole company.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Preferences {
    /// The currency the books are kept in. [`CurrencyId::COMPANY`] in a
    /// company of one currency.
    pub home_currency: CurrencyId,
    /// Whether the company uses more than one currency.
    pub uses_multicurrency: bool,
    /// The month that its years start with. January where the file does not say.
    pub fiscal_year_start: Month,
    /// The month that its years for income tax start with, where the file
    /// says.
    pub tax_year_start: Option<Month>,
    /// What open invoices and bills are aged from.
    pub aging_basis: AgingBasis,
    /// Whether accounts are shown with their numbers.
    pub uses_account_numbers: bool,
    /// Whether lines are given classes.
    pub uses_classes: bool,
    /// Whether the company keeps stock.
    pub uses_inventory: bool,
    /// Whether it collects sales tax.
    pub uses_sales_tax: bool,
    /// Whether it takes sales orders.
    pub uses_sales_orders: bool,
    /// Whether it keeps stock at several sites.
    pub uses_sites: bool,
    /// Whether the sites have bins.
    pub uses_bins: bool,
    /// Whether it follows serial numbers.
    pub uses_serial_numbers: bool,
    /// Whether what is on sales orders is not available.
    pub available_less_sales_orders: bool,
    /// Whether what pending builds need is not available.
    pub available_less_builds: bool,
    /// The sales tax that a new customer gets.
    pub default_tax_item: Option<ItemId>,
    /// The date that the books are closed at, as the file keeps it: day and
    /// month in the order of the computer it was set on, which is not known.
    pub closing_date: Option<String>,
    /// How much could not be read.
    pub left_out: LeftOut,
    stored: Vec<Preference>,
}

impl Preferences {
    /// Every preference of the whole company that has a value, as the file
    /// keeps it. A name can come up more than once.
    ///
    /// Some hold secrets, and the names of some hold a person's e-mail address.
    #[must_use]
    pub fn stored(&self) -> &[Preference] {
        &self.stored
    }
}

/// Reads the preferences.
///
/// # Errors
///
/// Returns [`Error::Damaged`] when a preference that every figure depends on
/// cannot be understood, or may be one that cannot be read.
pub(crate) fn read(source: &dyn Source) -> Result<Preferences> {
    let stored = Stored::read(source)?;
    Ok(Preferences {
        home_currency: stored.home_currency()?,
        uses_multicurrency: stored.is_set(preference::MULTICURRENCY),
        fiscal_year_start: stored.fiscal_year_start()?,
        tax_year_start: stored.get(preference::TAX_YEAR_START).and_then(month),
        aging_basis: if stored.is_set(preference::AGE_FROM_TRANSACTION_DATE) {
            AgingBasis::TransactionDate
        } else {
            AgingBasis::DueDate
        },
        uses_account_numbers: stored.is_set(preference::ACCOUNT_NUMBERS),
        uses_classes: stored.is_set(preference::CLASSES),
        uses_inventory: stored.is_set(preference::INVENTORY),
        uses_sales_tax: stored.is_set(preference::SALES_TAX),
        uses_sales_orders: stored.is_set(preference::SALES_ORDERS),
        uses_sites: stored.is_set(preference::SITES),
        uses_bins: stored.is_set(preference::BINS),
        uses_serial_numbers: stored.is_set(preference::SERIAL_NUMBERS),
        available_less_sales_orders: stored.is_set(preference::AVAILABLE_LESS_SALES_ORDERS),
        available_less_builds: stored.is_set(preference::AVAILABLE_LESS_BUILDS),
        default_tax_item: stored.default_tax_item(),
        closing_date: stored.value(preference::CLOSING_DATE).map(str::to_owned),
        left_out: LeftOut::of(&stored.damage),
        stored: stored.values,
    })
}

/// The values of the whole company, and what could not be read of them.
struct Stored {
    values: Vec<Preference>,
    damage: Vec<Damage>,
}

impl Stored {
    fn read(source: &dyn Source) -> Result<Self> {
        let mut damage = Vec::new();
        let names: HashMap<i32, String> = PREFERENCES
            .open(source)?
            .scan(&mut damage)
            .map(|(_, preference)| (preference.id, preference.name))
            .collect();
        let values = PREFERENCE_VALUES
            .open(source)?
            .scan(&mut damage)
            .filter(|(_, row)| row.user.is_none())
            .filter_map(|(origin, row)| {
                Some(Preference {
                    name: names.get(&row.preference)?.clone(),
                    value: row.value?,
                    origin,
                })
            })
            .collect();
        Ok(Self { values, damage })
    }

    fn get(&self, name: &str) -> Option<&Preference> {
        let mut values = self.values.iter();
        values.find(|preference| preference.name == name)
    }

    fn value(&self, name: &str) -> Option<&str> {
        self.get(name).map(|preference| preference.value.as_str())
    }

    /// Whether a preference that is on or off is on. The file writes `1` for
    /// the older ones and `T` for the newer.
    fn is_set(&self, name: &str) -> bool {
        matches!(self.value(name), Some("1" | "T"))
    }

    /// The preference called `name`, when every figure depends on it.
    ///
    /// # Errors
    ///
    /// Returns what could not be read, when the preference may be part of it.
    fn decisive(&self, name: &str) -> Result<Option<&Preference>> {
        match (self.get(name), self.damage.first()) {
            (None, Some(damage)) => Err(Error::Damaged(damage.clone())),
            (found, _) => Ok(found),
        }
    }

    fn home_currency(&self) -> Result<CurrencyId> {
        let Some(preference) = self.decisive(preference::HOME_CURRENCY)? else {
            return Ok(CurrencyId::COMPANY);
        };
        let number = preference.value.parse();
        number
            .map(CurrencyId::new)
            .map_err(|_| misunderstood(preference, "the number of a currency"))
    }

    fn fiscal_year_start(&self) -> Result<Month> {
        let Some(preference) = self.decisive(preference::FISCAL_YEAR_START)? else {
            return Ok(Month::January);
        };
        month(preference).ok_or_else(|| misunderstood(preference, "the number of a month"))
    }

    fn default_tax_item(&self) -> Option<ItemId> {
        let number = self.value(preference::DEFAULT_TAX_ITEM)?.parse().ok()?;
        (number != 0).then(|| ItemId::new(number))
    }
}

/// The month of a preference that counts months from 1.
fn month(preference: &Preference) -> Option<Month> {
    let number: u8 = preference.value.parse().ok()?;
    Month::try_from(number).ok()
}

/// The error for a preference that is not what it has to be.
fn misunderstood(preference: &Preference, expected: &'static str) -> Error {
    let Origin { table, row } = preference.origin;
    Error::Damaged(Damage::new(
        table,
        Some(row),
        Problem::Column {
            column: PREFERENCE_VALUE,
            found: ValueProblem::Type {
                expected,
                found: "another text",
            },
        },
    ))
}

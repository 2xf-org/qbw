//! Who the company is.

use chrono::NaiveDate;
use sa17::Source;

use crate::address::Address;
use crate::error::Result;
use crate::left_out::LeftOut;
use crate::settings::product_version;
use crate::settings::{Preference, Preferences, ProductVersion};
use crate::tables::information::company_preference;
use crate::unix_time::UnixTime;

/// Who the company is, and which QuickBooks wrote its file.
///
/// The numbers that it pays tax under are encrypted in the file, and are not
/// read.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct CompanyInfo {
    /// What the company calls itself.
    pub name: String,
    /// Its legal name.
    pub legal_name: Option<String>,
    /// Its legal address, in parts.
    pub legal_address: Address,
    /// The lines of its address as they print on forms, in the order the file
    /// keeps them. In some files that is not the order they print in.
    pub address_lines: Vec<String>,
    /// Its phone number.
    pub phone: Option<String>,
    /// The number of its fax.
    pub fax: Option<String>,
    /// Its e-mail address.
    pub email: Option<String>,
    /// Its website.
    pub website: Option<String>,
    /// The day that its books start, as it was set up.
    pub started: Option<NaiveDate>,
    /// When the file was made.
    pub created: Option<UnixTime>,
    /// The QuickBooks that the file is of.
    pub version: ProductVersion,
    /// How much could not be read, of the preferences and of what the file says
    /// about itself.
    pub left_out: LeftOut,
}

/// Reads who the company is, most of it from `preferences`.
///
/// # Errors
///
/// Returns an error when a table of it lacks a column.
pub(crate) fn read(source: &dyn Source, preferences: &Preferences) -> Result<CompanyInfo> {
    let mut damage = Vec::new();
    let version = product_version::read(source, &mut damage)?;
    let text = |name| first(preferences, name).map(|preference| preference.value.clone());
    Ok(CompanyInfo {
        name: text(company_preference::NAME).unwrap_or_default(),
        legal_name: text(company_preference::LEGAL_NAME),
        legal_address: Address {
            printed: Vec::new(),
            street: Address::lines(company_preference::LEGAL_STREET.map(text)),
            city: text(company_preference::LEGAL_CITY),
            state: text(company_preference::LEGAL_STATE),
            postal_code: text(company_preference::LEGAL_POSTAL_CODE),
            country: text(company_preference::LEGAL_COUNTRY),
        },
        address_lines: called(preferences, company_preference::ADDRESS_LINE)
            .map(|line| line.value.clone())
            .collect(),
        phone: text(company_preference::PHONE),
        fax: text(company_preference::FAX),
        email: text(company_preference::EMAIL),
        website: text(company_preference::WEBSITE),
        started: first(preferences, company_preference::STARTED).and_then(day),
        created: first(preferences, company_preference::CREATED).and_then(moment),
        version,
        left_out: preferences.left_out.with(LeftOut::of(&damage)),
    })
}

/// The preferences called `name`.
fn called<'stored>(
    preferences: &'stored Preferences,
    name: &'static str,
) -> impl Iterator<Item = &'stored Preference> {
    let stored = preferences.stored().iter();
    stored.filter(move |preference| preference.name == name)
}

fn first<'stored>(
    preferences: &'stored Preferences,
    name: &'static str,
) -> Option<&'stored Preference> {
    called(preferences, name).next()
}

/// The day of a preference that is written as in `2023/01/01`.
fn day(preference: &Preference) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(&preference.value, "%Y/%m/%d").ok()
}

/// The moment of a preference that counts seconds since 1970.
fn moment(preference: &Preference) -> Option<UnixTime> {
    preference.value.parse().ok().map(UnixTime::new)
}

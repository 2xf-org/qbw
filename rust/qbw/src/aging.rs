//! How late what is owed is.
//!
//! [`Company::aging`] sorts the [open items](crate::open_items) of a day into
//! buckets by their age, with a row for each name or each item.
//!
//! An invoice or a bill ages from the date it is due, or from its own date where
//! the company's preferences or the [`AgingOptions`] say so. One without a due
//! date ages from its own, and so does what takes off what is owed, such as a
//! payment that is not used up.
//!
//! ```
//! use qbw::aging::AgingOptions;
//! use qbw::chrono::NaiveDate;
//! use qbw::{Company, Side};
//!
//! # let file = qbw_synthetic::company::books().build();
//! let company = Company::from_bytes(file)?;
//! let day: NaiveDate = "2024-02-15".parse()?;
//! let options = AgingOptions::new(day);
//! let aging = company.aging(Side::Receivable, &options)?;
//! // Invoice 1041 was 62 days late, with 46 open.
//! assert_eq!(aging.totals.amounts[3].to_string(), "46.00");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # How sure it is
//!
//! The totals are those of the open items, which are proven. The buckets are
//! those that QuickBooks starts with, and that a company ages from the date of
//! the transaction when its preferences say so was in none of the files. No
//! report that QuickBooks printed was compared.
//!
//! [`Company::aging`]: crate::Company::aging

mod bucket;
mod options;
mod read;
mod report;
mod sums;

pub use bucket::Bucket;
pub use options::{AgingBy, AgingOptions, BucketsError};
pub use report::{Aged, AgedItem, Aging, AgingRow};

pub(crate) use read::AgingReader;

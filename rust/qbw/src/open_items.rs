//! What is owed: the invoices and bills that are not paid, and the payments and
//! credits that are not used up.
//!
//! [`Company::open_items`] gives all of one [`Side`], now or at the end of a
//! day. [`Company::owed_by`] is what one customer owes on a day and what of it
//! is overdue, and [`Company::owed_to`] what one vendor is owed.
//! [`Company::names_owed`] is that of every name in one pass.
//!
//! ```
//! use qbw::Company;
//! use qbw::chrono::NaiveDate;
//!
//! # let file = qbw_synthetic::company::books().build();
//! let company = Company::from_bytes(file)?;
//! let names = company.names()?;
//! let customer = names.customers().named("Ada Stores").ok_or("no such customer")?;
//! let day: NaiveDate = "2024-06-30".parse()?;
//! let owed = company.owed_by(customer.id, day)?.ok_or("no such customer")?;
//! assert_eq!(owed.home_total.to_string(), "45.00");
//! assert_eq!(owed.home_overdue.to_string(), "45.00");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Signs
//!
//! An amount is above zero for what is owed, on either side. An invoice and a
//! bill are, and a payment that is not used up, a credit memo and the credit of
//! a vendor are below zero. The items of an account come to its balance on
//! [`Side::Receivable`], and to its balance with the sign turned on
//! [`Side::Payable`].
//!
//! # How sure it is
//!
//! In the files of three companies, what the file keeps as open is the amount
//! less what the links say was settled, for 99,041 of 99,046 transactions. Five
//! contradict themselves. At five dates over ten years, the open items came to
//! the balance of every account for what is owed, to the cent: 51 of 51.
//!
//! QuickBooks can show what was open on a past day in more than one way. Here a
//! payment counts when both it and what it pays are dated up to the day, which
//! is the only way that comes to the accounts' balances. No report that
//! QuickBooks printed was compared.
//!
//! An item in another currency is worth what its line posted, less what was
//! settled of it at the rate of the transaction. What a change of rates added to
//! an account is in no item: it is [`OpenItems::adjustments`]. In the company's
//! currency, items and adjustments come to the balance of the accounts but for
//! two thousandths of it at most, which is what items leave that were paid bit
//! by bit at rounded amounts. On the first days of the quarters of twenty years,
//! 6,001 of 13,768 names in another currency come to their balance to the cent.
//!
//! [`Company::names_owed`]: crate::Company::names_owed
//! [`Company::open_items`]: crate::Company::open_items
//! [`Company::owed_by`]: crate::Company::owed_by
//! [`Company::owed_to`]: crate::Company::owed_to
//! [`Side`]: crate::Side
//! [`Side::Payable`]: crate::Side::Payable
//! [`Side::Receivable`]: crate::Side::Receivable

mod items;
mod names_owed;
mod open_item;
mod owed;
mod part;
mod read;

pub use items::OpenItems;
pub use names_owed::{NameOwed, NamesOwed};
pub use open_item::OpenItem;
pub use owed::Owed;

pub(crate) use open_item::OpenSums;
pub(crate) use part::Settling;
pub(crate) use read::{Moment, OpenItemReader};

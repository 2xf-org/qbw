//! Reads QuickBooks Desktop company files (`.qbw`) as what a bookkeeper knows
//! in them, such as accounts with their types and balances, and gives every
//! amount as an exact number of one currency.
//!
//! Everything is asked of a [`Company`]:
//!
//! ```
//! use qbw::Company;
//!
//! # let file = qbw_synthetic::company::books().build();
//! let company = Company::from_bytes(file)?;
//! let accounts = company.accounts()?;
//! let parts = accounts.named("Sales:Parts").ok_or("no such account")?;
//! let balance = parts.stored.ok_or("no balance")?;
//! assert_eq!(balance.amount.to_string(), "-100.00");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # What to ask
//!
//! | Question | Method of [`Company`] |
//! | --- | --- |
//! | Which accounts, names, items and other entries there are | [`accounts`](Company::accounts), [`names`](Company::names), [`items`](Company::items) and the [lists] |
//! | What a transaction says, line by line | [`transaction`](Company::transaction) |
//! | Which transactions there are | [`find`](Company::find), [`transactions`](Company::transactions), [`transaction_summary`](Company::transaction_summary) |
//! | What an account's balance is, and what was posted to it | [`ledger`](Company::ledger), [`register`](Company::register) |
//! | What the company earned, has and owes | [`profit_and_loss`](Company::profit_and_loss), [`balance_sheet`](Company::balance_sheet) and the other [reports] |
//! | Who owes what, and how late it is | [`owed_by`](Company::owed_by), [`owed_to`](Company::owed_to), [`names_owed`](Company::names_owed), [`open_items`](Company::open_items), [`aging`](Company::aging) |
//! | What paid what | [`paid_by`](Company::paid_by), [`applied_to`](Company::applied_to), [`links`](Company::links) |
//! | What is in stock, and what is ordered | [`stock`](Company::stock), [`valuation`](Company::valuation), [`movements`](Company::movements), [`open_orders`](Company::open_orders) |
//! | What a person sees on selecting an entry | [`overview`](Company::overview), [`name_overview`](Company::name_overview), [`item_overview`](Company::item_overview), [`account_overview`](Company::account_overview) |
//! | How far the figures can be trusted | [`check`](Company::check) |
//!
//! # Three layers
//!
//! A program can stop at any of three layers, and go down one for the same
//! thing:
//!
//! - The records of this crate, such as an [`Account`](accounts::Account).
//!   They name what a bookkeeper names, with ids, codes and amounts as types.
//! - The stored row below. A record that a row of the file stands for has an
//!   [`Origin`], and [`Company::stored_row`] gives that row, with every column
//!   of the table. What a record has from other rows is not in it, such as the
//!   stored balance of an account, the order of a list, or the contact of a
//!   name. A [`User`](settings::User) comes from many rows and has no origin.
//! - The database. [`Company::source`] is it, with all the tables that the
//!   file has, also those that this crate does not read. Its crate is
//!   [`sa17`].
//!
//! ```
//! # use qbw::Company;
//! # let company = Company::from_bytes(qbw_synthetic::company::books().build())?;
//! # let accounts = company.accounts()?;
//! let parts = accounts.named("Sales:Parts").ok_or("no such account")?;
//! let row = company.stored_row(parts.origin)?.ok_or("no such row")?;
//! assert_eq!(row.get("sublevel").and_then(|value| value.as_i64()), Some(1));
//!
//! let currencies = company.source().table("abmc_currency")?;
//! assert_eq!(currencies.rows().count(), 3);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! A company can be read from anything that answers in a file's place: a
//! [`Company`] is made of any [`Source`](sa17::Source), such as a file
//! with changes laid over it.
//!
//! # Amounts
//!
//! An amount is [`Money`]: an exact number and its currency. Where there are
//! two, the one with the plain name is in the currency of the account, the name
//! or the transaction, and the one called `home` is in the company's: `amount`
//! and `home_amount`, `total` and `home_total`. A company that keeps its books
//! in one currency has every amount in [`CurrencyId::COMPANY`], and [`Totals`]
//! adds amounts of any currencies without mixing them.
//!
//! In the books a debit is positive and a credit negative, so the balance of an
//! account that grows by credits, such as one for sales, is below zero when it
//! is in order. Postings, balances, registers, the trial balance, the general
//! ledger, the journal and the account listing have that sign, and
//! [`AccountType::as_kept`](accounts::AccountType::as_kept) turns an amount or a
//! balance of one of them, so that it is above zero on the side that its account
//! grows on. [What is owed](open_items#signs) is above zero as it is, and so are
//! the amounts of [forms](transactions#signs) and the sections of
//! [statements](reports#signs).
//!
//! A quantity is a [`Decimal`](sa17::Decimal) without zeros at the end of
//! its fraction, so it prints as a person writes it: `5`, or `2.5`. A sum of two
//! can have such zeros, which `Decimal::normalize` takes away.
//!
//! Nothing here reads the clock. What is of a day takes the day, and `as_of` is
//! the end of it. [`AsOf::Latest`] is everything in the file.
//!
//! # Figures
//!
//! Balances, [registers] and the statements of [`reports`] are worked out from
//! the postings, which [`Company::ledger`] has all of. What is open on a day is
//! worked out from the postings and the [links]. What is open now, stock and
//! its value are kept in the file as figures, and are read as they are. The
//! documentation of each module says how sure its rules are, and
//! [`Company::check`] compares the two kinds of figures.
//!
//! ```
//! # use qbw::Company;
//! # let company = Company::from_bytes(qbw_synthetic::company::books().build())?;
//! let day = "2024-06-30".parse()?;
//! let sheet = company.balance_sheet(day)?;
//! assert_eq!(sheet.assets.total.to_string(), "1262.00");
//! assert_eq!(sheet.liabilities_and_equity, sheet.assets.total);
//!
//! let year = company.profit_and_loss(company.fiscal_year(day)?)?;
//! assert_eq!(year.net_income, sheet.equity.net_income);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # What cannot be read
//!
//! A damaged row does not stop the reading. What is read goes on without it: a
//! list, a sum and a report say how much they lack with a [`LeftOut`], which
//! counts the rows of everything that decides them, and a list, the ledger and
//! the links have what is wrong as [`Damage`]. A transaction whose header
//! cannot be read keeps what its lines post, and an answer that shows it without
//! its number counts the header, once. A row with a key that another row has is
//! damage too. An [`enum@Error`] is for what leaves no answer, such as a file
//! that is no company's, or one that lacks a table that every file has. A table
//! or a column that only newer files have reads as nothing in an older one.
//!
//! An id that the file does not have is no error: what is asked about it is
//! `None`, and a walk over what it has is empty.
//!
//! # What prints itself
//!
//! Every public type has `Debug`. A record prints itself whole, with the names,
//! addresses, memos and amounts of the business, so it belongs in a log as
//! little as the file does. What can hold a login or a secret prints without
//! it: a [`User`](settings::User), a [`Fact`](settings::Fact), a
//! [`Preference`](settings::Preference). A [`Company`] and a
//! [`Ledger`](ledger::Ledger) print only how large they are.
//!
//! # What it does not promise
//!
//! - QuickBooks' maker publishes no description of the file. What the tables
//!   mean was worked out from the files of three companies, and an item that
//!   says unchecked is of a kind that none of them has.
//! - A code without a known meaning is kept as `Other`, with its number.
//! - Bank account numbers, tax ids and passwords are encrypted in the file,
//!   and are not read.
//! - Reading never changes the file, and nothing here writes one.

pub mod accounts;
pub mod aging;
pub mod check;
pub mod custom_fields;
pub mod inventory;
pub mod items;
pub mod ledger;
pub mod links;
pub mod lists;
pub mod names;
pub mod open_items;
pub mod orders;
pub mod overviews;
pub mod query;
pub mod registers;
pub mod reports;
pub mod settings;
pub mod transactions;

mod address;
mod amount;
mod as_of;
mod case;
mod chain;
mod code;
mod codes;
mod company;
mod currencies;
mod damage;
mod error;
mod ids;
mod left_out;
mod list;
mod money;
mod money_error;
mod number;
mod origin;
mod other_side;
mod page;
mod party;
mod period;
mod posting_state;
mod price;
mod rest;
mod side;
mod tables;
mod totals;
mod unix_time;

pub use chrono;
pub use sa17;

pub use address::Address;
pub use as_of::AsOf;
pub use code::UnknownWords;
pub use company::Company;
pub use damage::{Damage, Problem};
pub use error::{Error, Result};
pub use ids::{
    AccountId, AddressId, CategoryId, ClassId, ContactId, CurrencyId, CustomFieldId,
    CustomerTypeId, ItemId, JobTypeId, LineId, MemorizedId, MessageId, NameId, OwnerKey,
    PaymentMethodId, PriceLevelId, SalesRepId, SalesTaxCodeId, ShipMethodId, SiteId, TemplateId,
    TermsId, TransactionId, UserId, VendorTypeId,
};
pub use left_out::LeftOut;
pub use list::{List, ListEntry, ListId};
pub use money::Money;
pub use money_error::MoneyError;
pub use origin::Origin;
pub use other_side::OtherSide;
pub use page::Page;
pub use period::Period;
pub use price::Price;
pub use side::Side;
pub use totals::Totals;
pub use unix_time::UnixTime;

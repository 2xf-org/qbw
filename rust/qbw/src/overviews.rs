//! What a person sees on selecting an entry, put together.
//!
//! Each overview is of one page of QuickBooks, and gives what there is about an
//! entry at the end of a day, with the latest of its transactions. Nothing in
//! one has a rule of its own: every figure is another answer of this crate, whose
//! documentation says how sure it is.
//!
//! | Answer | QuickBooks' page |
//! | --- | --- |
//! | [`Company::overview`] | The Home page with its account balances, and what the Company Snapshot and the trackers of income and bills sum up |
//! | [`Company::name_overview`] | The information and the transactions of the Customer, Vendor and Employee Centers |
//! | [`Company::item_overview`] | The information and the transactions of the Inventory Center, and the report that the Item List opens |
//! | [`Company::account_overview`] | A line of the Chart of Accounts, and the register or the report that it opens |
//!
//! QuickBooks shows these pages with everything in the file. What is dated after
//! the day is left out here, so that the balances, what is owed and the
//! transactions agree. An item is as it is now, since orders have no past.
//!
//! Its centers show a transaction's type, number, date, account and amount, which
//! a [`TransactionSummary`](crate::query::TransactionSummary) has. That the amount
//! is what the own side posts, with a debit as positive, is inferred, and so is
//! that a customer's list has the transactions of its jobs.
//!
//! An overview has the latest transactions only. The rest, or those of one kind,
//! are a [query](crate::query) or a [register](crate::registers) away:
//!
//! ```
//! use qbw::chrono::NaiveDate;
//! use qbw::query::TransactionQuery;
//! use qbw::transactions::TransactionKind;
//! use qbw::{Company, NameId, Page};
//!
//! # let file = qbw_synthetic::company::books().build();
//! let company = Company::from_bytes(file)?;
//! let customer = NameId::new(101);
//! let day: NaiveDate = "2024-06-30".parse()?;
//! let overview = company.name_overview(customer, day, 5)?;
//! let overview = overview.ok_or("no such name")?;
//! assert_eq!(overview.total.map(|total| total.to_string()).as_deref(), Some("45.00"));
//! assert_eq!(overview.transactions.kinds[0].kind, TransactionKind::Invoice);
//!
//! let query = TransactionQuery::new().name(customer).with_jobs();
//! let invoices = query.kinds([TransactionKind::Invoice]);
//! let invoices = company.transactions(&invoices, Page::all())?;
//! assert_eq!(invoices.row_count, overview.transactions.kinds[0].count);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! [`Company::account_overview`]: crate::Company::account_overview
//! [`Company::item_overview`]: crate::Company::item_overview
//! [`Company::name_overview`]: crate::Company::name_overview
//! [`Company::overview`]: crate::Company::overview

mod account_overview;
mod company_overview;
mod item_overview;
mod name_overview;

pub use account_overview::AccountOverview;
pub use company_overview::CompanyOverview;
pub use item_overview::{ItemOverview, ItemRow};
pub use name_overview::NameOverview;

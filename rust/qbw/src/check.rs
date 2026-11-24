//! How far the figures can be trusted: what is worked out from the books, against
//! what QuickBooks keeps and against the rest of the books.
//!
//! [`Company::check`] works everything out in two ways where the file has two,
//! and gives a [`Check`]: how often figures agree, for each [`CheckKind`], and a
//! [`Finding`] for each that do not.
//!
//! ```
//! use qbw::Company;
//! use qbw::check::CheckKind;
//!
//! # let file = qbw_synthetic::company::books().build();
//! let company = Company::from_bytes(file)?;
//! let check = company.check()?;
//! let balances = check.get(CheckKind::AccountBalance).ok_or("no balances")?;
//! assert_eq!((balances.agreed, balances.looked_at), (21, 22));
//!
//! let found = check.unexplained().next().ok_or("everything agrees")?;
//! let causes: Vec<String> = found.causes.iter().map(ToString::to_string).collect();
//! assert_eq!(causes, ["maybe QuickBooks' total is out of date"]);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # What a finding means
//!
//! Two figures differ, and one of them is wrong. A [`Cause`] with an amount was
//! worked out apart from both of them, and a finding whose causes come to the
//! difference [is explained](Finding::is_explained). Every other cause is only
//! what may be behind it. A total that QuickBooks has not brought up to date
//! looks the same as a transaction that is missing here, so such a finding stays
//! unexplained, and somebody has to look. [`Check::unexplained`] has all of
//! these. A file whose books are in order can have some.
//!
//! What the file lacks a table for is in [`Check::not_compared`]. A check reads
//! the whole file, so it takes the longest of all questions. The accounts, names
//! and items that it looks at are the listed ones. What differs is never an error.
//!
//! # What it gave for three companies
//!
//! Some kinds compare with a figure that QuickBooks keeps. The others compare
//! the books with themselves: they would differ if lines were lost, but they
//! cannot show that a rule of this crate is the one QuickBooks follows.
//!
//! | Kind | Compared with | Agree | The others |
//! | --- | --- | --- | --- |
//! | [`TransactionBalance`](CheckKind::TransactionBalance) | the books | 265,141 of 265,141 | |
//! | [`LedgerBalance`](CheckKind::LedgerBalance) | the books | 3 of 3 | |
//! | [`HeaderAndLines`](CheckKind::HeaderAndLines) | the books | 289,829 of 289,829 | |
//! | [`LineChain`](CheckKind::LineChain) | the books | 289,829 of 289,829 | |
//! | [`FormTotal`](CheckKind::FormTotal) | the books | 273,019 of 273,020 | maybe rounding |
//! | [`AccountBalance`](CheckKind::AccountBalance) | QuickBooks | 773 of 777 | maybe out of date |
//! | [`CustomerBalance`](CheckKind::CustomerBalance) | QuickBooks | 2,913 of 2,914 | maybe out of date |
//! | [`VendorBalance`](CheckKind::VendorBalance) | QuickBooks | 3,476 of 3,476 | |
//! | [`OpenForm`](CheckKind::OpenForm) | QuickBooks | 99,039 of 99,044 | no cause is known |
//! | [`OpenReceivable`](CheckKind::OpenReceivable) | QuickBooks | 67,520 of 67,525 | the same five |
//! | [`OpenPayable`](CheckKind::OpenPayable) | QuickBooks | 31,521 of 31,521 | |
//! | [`ReceivableAccount`](CheckKind::ReceivableAccount) | QuickBooks | 4 of 5 | explained by the same five |
//! | [`PayableAccount`](CheckKind::PayableAccount) | QuickBooks | 6 of 6 | |
//! | [`ItemQuantity`](CheckKind::ItemQuantity) | QuickBooks | 2,454 of 2,454 | |
//! | [`ItemCount`](CheckKind::ItemCount) | QuickBooks | 2,454 of 2,454 | |
//! | [`ItemValue`](CheckKind::ItemValue) | QuickBooks | 2,454 of 2,454 | |
//! | [`ItemPostings`](CheckKind::ItemPostings) | QuickBooks | 2,449 of 2,454 | maybe costs of builds |
//! | [`SiteQuantity`](CheckKind::SiteQuantity) | QuickBooks | 5,499 of 5,511 | maybe sites that do not add up |
//! | [`OnOrder`](CheckKind::OnOrder) | QuickBooks | 2,454 of 2,454 | |
//! | [`OnSalesOrders`](CheckKind::OnSalesOrders) | QuickBooks | 2,453 of 2,454 | maybe out of date |
//! | [`ToBeBuilt`](CheckKind::ToBeBuilt) | QuickBooks | 2,454 of 2,454 | |
//! | [`RequiredForBuilds`](CheckKind::RequiredForBuilds) | QuickBooks | 2,454 of 2,454 | |
//! | [`InventoryAsset`](CheckKind::InventoryAsset) | QuickBooks | 0 of 5 | four explained by other postings |
//! | [`YearlyFigure`](CheckKind::YearlyFigure) | QuickBooks | 6 of 9 | maybe of years that went on |
//!
//! Of 1,348,247 figures, 1,348,204 agree. Five of the other 43 are explained and
//! 38 are not, and for ten of these no cause is known.
//!
//! [`Company::check`]: crate::Company::check

mod balances;
mod cause;
mod facts;
mod finding;
mod forms;
mod kind;
mod owed;
mod report;
mod stock;
mod tally;

pub use cause::Cause;
pub use finding::{Figure, Finding, Subject};
pub use kind::CheckKind;
pub use report::{Check, Compared, NotCompared};

pub(crate) use balances::{accounts, ledger, names};
pub(crate) use facts::facts;
pub(crate) use forms::FormCheck;
pub(crate) use owed::Open;
pub(crate) use stock::StockCheck;
pub(crate) use tally::Tally;

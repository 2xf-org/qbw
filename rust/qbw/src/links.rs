//! What paid, credited, deposited, received or invoiced what.
//!
//! A payment does not name the invoices it pays, nor a deposit the payments in
//! it. The file keeps links between them, and [`Company::links`] gives those of
//! a transaction, seen from it: an invoice is [paid by](Relation::PaidBy) a
//! payment, and the payment is [applied to](Relation::AppliedTo) the invoice.
//!
//! ```
//! use qbw::Company;
//! use qbw::transactions::TransactionKind;
//!
//! # let file = qbw_synthetic::company::books().build();
//! let company = Company::from_bytes(file)?;
//! let invoice = company.find(TransactionKind::Invoice, "1041")?.ok_or("no such invoice")?;
//! let paid = company.paid_by(invoice)?.ok_or("no such invoice")?;
//! for link in &paid {
//!     println!("{} {} {:?}", link.other.date, link.other.kind, link.amount);
//! }
//! assert_eq!(paid.total.to_string(), "157.00");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # How sure it is
//!
//! In the files of three companies, what settled each of 24,844 paid invoices
//! and 16,365 paid bills comes to their totals. A payment or a credit is what it
//! was used for and what is left of it, for 50,413 of 50,417 of them, and the
//! payments in each of 30,995 deposits are its lines. The file keeps every link
//! twice, once for each end, and the two say the same.
//!
//! A link of a code that none of them has is [`Relation::LinkedTo`]. It may
//! settle something, so what adds up what was settled counts it as left out.
//!
//! Not settled is what the codes mean that only tie a transaction to itself, and
//! why some links are for information only. QuickBooks shows neither, and
//! neither is here.
//!
//! [`Company::links`]: crate::Company::links

mod index;
mod line_link;
mod link;
mod linked_transaction;
mod read;
mod relation;
mod stored_link;
mod transaction_links;

pub use line_link::{LineLink, LineLinks};
pub use link::Link;
pub use linked_transaction::LinkedTransaction;
pub use relation::Relation;
pub use transaction_links::{Links, Settled};

pub use crate::codes::LinkCode;

pub(crate) use index::{LinkDates, LinkIndex};
pub(crate) use read::LinkReader;
pub(crate) use stored_link::StoredLink;

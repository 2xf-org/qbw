//! What happened in the company: sales, purchases, payments and the rest.
//!
//! Every form of QuickBooks is a [`Transaction`], whatever its [kind]. What
//! only some kinds have is in its [`Details`].
//!
//! ```
//! use qbw::Company;
//! use qbw::transactions::TransactionKind;
//!
//! # let file = qbw_synthetic::company::books().build();
//! let company = Company::from_bytes(file)?;
//! let id = company.find(TransactionKind::Invoice, "1041")?;
//! let invoice = company.transaction(id.ok_or("no such invoice")?)?;
//! let invoice = invoice.ok_or("no such invoice")?;
//! for line in invoice.form_lines() {
//!     println!("{:?} {:?}", line.description, line.amount);
//! }
//! let open = invoice.open().ok_or("it can be open")?;
//! assert_eq!(open.to_string(), "45.00");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Its own side and its lines
//!
//! A form is of an account: a check of the bank's, an invoice of the one for
//! what customers owe. That is the transaction's own side, [`OwnLine`], and
//! the rest are its [lines](Line), in the order of the form. A [`LineRole`]
//! says what a line is for. Some lines repeat the sum of others, and
//! [`LineRole::adds_to_total`] says which do not.
//!
//! # Signs
//!
//! The books tell debits from credits, and a form does not: it shows its total
//! and its lines as positive, and a discount or what comes back below zero.
//! [`Transaction::total`], [`Line::amount`] and [`Line::quantity`] are what
//! the form shows. [`OwnLine::amount`] and [`Line::posted`] are what the books
//! get, with a debit as positive. Journal entries and changes of the stock are
//! on neither side, so they have no total and show their lines as they are
//! posted. [`Line::debit`] and [`Line::credit`] are the two columns of a journal
//! entry.
//!
//! In the files of three companies, the lines that add to the total of a form
//! add up to it in the transaction's currency on 273,019 of 273,020 forms. The
//! other is stored a cent off. The columns of all 13,382 journal entries come to
//! the same.
//!
//! # What is left
//!
//! [`OpenState`] is what is left to pay of an invoice or a bill, or to use of a
//! payment or a credit, as the file keeps it, and a credit that is not used up
//! is above zero as on its form. [`OrderProgress`] is how far an order is filled.
//!
//! A line of an order that does not say how many is of one, and what its links
//! filled agrees for 64,579 of 64,580 lines. A discount is the exception: it has
//! no quantity.
//!
//! [kind]: TransactionKind

mod details;
mod family;
mod form_side;
mod forms;
mod header;
mod headers;
mod left;
mod line;
mod line_index;
mod line_role;
mod open_state;
mod order_progress;
mod own_line;
mod own_lines;
mod own_side;
mod posted;
mod reader;
mod shown_headers;
mod stored_line;
mod transaction;

pub use details::Details;
pub use forms::{
    Bill, BillPayment, Build, CheckForm, InventoryAdjustment, JournalForm, Payment, PurchaseOrder,
    Refund, Sale, Transfer,
};
pub use line::Line;
pub use line_role::LineRole;
pub use open_state::OpenState;
pub use order_progress::OrderProgress;
pub use own_line::OwnLine;
pub use posted::Posted;
pub use transaction::Transaction;

pub use crate::codes::{Cleared, LastAction, TransactionKind};
pub use crate::posting_state::PostingState;

pub(crate) use form_side::FormSide;
pub(crate) use header::Header;
pub(crate) use headers::Headers;
pub(crate) use left::Left;
pub(crate) use line_index::LineIndex;
pub(crate) use order_progress::OrderLine;
pub(crate) use own_lines::OwnLines;
pub(crate) use own_side::OwnSide;
pub(crate) use reader::Reader;
pub(crate) use shown_headers::ShownHeaders;

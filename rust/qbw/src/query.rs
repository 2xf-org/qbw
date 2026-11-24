//! Finding transactions.
//!
//! A [`TransactionQuery`] says which transactions to list and in which order,
//! and [`Company::transactions`](crate::Company::transactions) gives a [page]
//! of them. A row of a page is a [`TransactionSummary`], and
//! [`Company::transaction`](crate::Company::transaction) gives the transaction
//! with its lines.
//!
//! The first answer reads the headers of every transaction. One that looks at
//! what is on the own lines, such as whether an invoice is paid, also reads
//! the lines of the kinds it is about. Both are kept.
//!
//! [page]: TransactionPage

mod direction;
mod memos;
mod natural;
mod paid_state;
mod posting_filter;
mod search;
mod transaction_order;
mod transaction_page;
mod transaction_query;
mod transaction_summary;

pub use direction::Direction;
pub use paid_state::PaidState;
pub use posting_filter::PostingFilter;
pub use transaction_order::TransactionOrder;
pub use transaction_page::{KindCount, TransactionPage};
pub use transaction_query::TransactionQuery;
pub use transaction_summary::TransactionSummary;

pub(crate) use search::Search;

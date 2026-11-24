//! The postings of the books, and the balances they add up to.
//!
//! A posting is an amount on an account: a line of a transaction, or what a
//! line of an item in stock posts besides. [`Ledger`] holds all of them.
//! Nothing that QuickBooks keeps as a total goes into it, so it can be checked
//! against those.
//!
//! # What posts
//!
//! Every line that has an account, unless its transaction does not post:
//! estimates, orders, pending forms and memorized transactions do not. A line
//! of an item in stock can post two amounts more, the change of the stock's
//! value and the cost of what was sold. An amount is a debit where it is
//! positive.
//!
//! In the files of three companies the postings of each of 265,141
//! transactions add up to zero, and the lifetime balance of 826 of 831 accounts
//! is the one that QuickBooks keeps. We take the other five for totals that
//! QuickBooks had not brought up to date. What customers owe agrees for 3,047 of
//! 3,048 of them, and what vendors are owed for all 3,546.
//!
//! # Two amounts
//!
//! [`Posting::amount`] is in the account's currency and
//! [`Posting::home_amount`] in the company's. They differ on an account in
//! another currency. A line that only settles an exchange difference has an
//! amount of zero there.
//!
//! A customer or a vendor trades in one currency, and what they owe or are owed
//! is in that one, whatever the accounts are kept in.
//!
//! # Order
//!
//! Postings are ordered by date and within a day by line. The order in which
//! QuickBooks shows the transactions of one day is not known, so a running
//! balance is certain at the end of each day and not between.

mod account_postings;
mod balance;
mod book;
mod build;
mod claim;
mod entry;
mod places;
mod posting;
mod posting_source;
mod postings;
mod subledgers;
mod sums;

pub use balance::Balance;
pub use book::Ledger;
pub use posting::Posting;
pub use posting_source::PostingSource;
pub use postings::Postings;

pub(crate) use build::build;
pub(crate) use claim::Claim;
pub(crate) use entry::Entry;
pub(crate) use sums::Sums;

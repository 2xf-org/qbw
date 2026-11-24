//! Statements and reports, worked out from the postings.
//!
//! Every figure is a sum over the [`Ledger`](crate::ledger::Ledger), in the
//! company's currency. Nothing that QuickBooks keeps as a total goes into one.
//! All of it is on the accrual basis: a sale counts on the date of its invoice,
//! not when it is paid.
//!
//! # Signs
//!
//! A balance has a debit as positive, in the [`TrialBalance`], the
//! [`GeneralLedger`] and the [`AccountListing`]. A [`Section`] of the other two
//! statements turns it to the sign that its accounts grow by, so that income,
//! debts and equity are positive where they are in order. An account that went
//! the other way, such as a credit card that was paid too much, is below zero.
//!
//! # Years
//!
//! An account of the balance sheet goes on from year to year. One of profit and
//! loss starts again with each fiscal year, whose first month is in the
//! company's preferences. QuickBooks posts nothing at the end of a year, and
//! neither does a statement: the profit of earlier years is worked out, and
//! stands with what is posted to Retained Earnings. Net income is the profit of
//! the year so far.
//!
//! # Sections
//!
//! The [type](crate::accounts::AccountType) of an account decides its section.
//! An account stands below its parent when both are of one type, and a parent's
//! total is its own amount and those of the accounts below it. Accounts are in
//! the order of the list.
//!
//! What is posted to an account that cannot be read, or whose type this version
//! does not know, is in no section. It is [`BalanceSheet::unplaced`] and
//! [`ProfitAndLoss::unplaced`], and it is in the trial balance and the general
//! ledger.
//!
//! # How sure it is
//!
//! The files of three companies were checked, with 265,141 transactions in 57
//! fiscal years. What follows held without an exception unless it says so.
//!
//! - The lifetime balance of an account is the one that QuickBooks keeps: 826
//!   of 831. We take the other five for totals that QuickBooks had not brought
//!   up to date.
//! - Net income of the last fiscal year is the figure that QuickBooks worked
//!   out itself, which every file has.
//! - Every transaction comes to zero, and so does the whole ledger.
//! - Assets are liabilities and equity, and debits are credits, at the end of
//!   each fiscal year and on 216 days within, and the general ledger opens and
//!   closes at zero in every fiscal year. That follows from the point before,
//!   and says that no posting is lost. It says nothing about the year that a
//!   profit belongs to.
//!
//! The rest is how QuickBooks shows these statements, and is taken from its
//! documentation and from what is left to choose. No file says so, and no
//! printed report was compared:
//!
//! - which accounts a statement leaves out, and in which order sections stand;
//! - that the trial balance has this year's amounts of profit and loss, with
//!   the profit of earlier years in Retained Earnings. With the amounts of all
//!   years and without that profit, debits would be credits, too;
//! - that an account of profit and loss opens the general ledger with the year
//!   so far, and that nothing is closed within its period;
//! - that the journal is in the order in which transactions were entered;
//! - that the account listing has no balance for accounts of profit and loss.
//!
//! All three companies have years that start in January, so another first month
//! was tested on made-up books only.

mod account_listing;
mod balance_sheet;
mod books;
mod general_ledger;
mod journal;
mod name_balances;
mod name_ledger;
mod owed_balance;
mod profit_and_loss;
mod section;
mod section_tree;
mod trial_balance;

pub use account_listing::{AccountListing, AccountListingRow};
pub use balance_sheet::{Assets, BalanceSheet, EarlierProfit, Equity, Liabilities};
pub use general_ledger::{AccountLedger, GeneralLedger, LedgerRow};
pub use journal::{Journal, JournalEntry};
pub use name_balances::{NameBalanceRow, NameBalances};
pub use name_ledger::{NameLedger, NameLedgerRow};
pub use owed_balance::OwedBalance;
pub use profit_and_loss::ProfitAndLoss;
pub use section::{Section, SectionAccount};
pub use trial_balance::{TrialBalance, TrialBalanceRow};

pub(crate) use account_listing::account_listing;
pub(crate) use balance_sheet::balance_sheet;
pub(crate) use books::Books;
pub(crate) use general_ledger::general_ledger;
pub(crate) use name_balances::{name_balance, name_balances};
pub(crate) use name_ledger::name_ledger;
pub(crate) use profit_and_loss::profit_and_loss;
pub(crate) use trial_balance::trial_balance;

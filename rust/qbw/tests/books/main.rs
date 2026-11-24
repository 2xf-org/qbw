//! Reads the synthetic company through the crate's public interface. What is
//! expected is typed in, as the company's own values are.

// Every function here runs inside a test, where a failed unwrap is a failed test.
#![allow(clippy::unwrap_used)]

mod accounts;
mod aging;
mod broken_transactions;
mod changed;
mod check;
mod company;
mod counted;
mod custom_fields;
mod damaged_transactions;
mod fixture;
mod forms;
mod information;
mod inventory;
mod items;
mod ledger;
mod left_out;
mod links;
mod lists;
mod lost_rows;
mod name_reports;
mod names;
mod odd_accounts;
mod odd_forms;
mod odd_lines;
mod odd_transactions;
mod one_currency;
mod open_items;
mod orders;
mod overviews;
mod owing_books;
mod queries;
mod registers;
mod reports;
mod settings;
mod settlement;
mod small_books;
mod small_lists;
mod small_owing;
mod small_stock;
mod support;
mod transaction_states;
mod transactions;
mod unused_payments;
mod years;

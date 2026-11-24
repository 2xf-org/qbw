//! An open company file.

// The documentation lists the methods in the order of the modules.
mod handle;

mod accounts;
mod check;
mod custom_fields;
mod information;
mod inventory;
mod items;
mod ledger;
mod links;
mod lists;
mod names;
mod open_items;
mod overviews;
mod per_form;
mod pieces;
mod registers;
mod reports;
mod settings;
mod transactions;

pub use handle::Company;

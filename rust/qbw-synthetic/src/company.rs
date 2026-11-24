//! Companies to try qbw on. Everything about them is made up.
//!
//! [`sample`] is a few tables of no program in particular. [`books`] is a
//! company laid out as QuickBooks Desktop lays one out, and
//! [`one_currency_books`] a small one that uses no other currency.

mod books;
mod sample;

pub use books::{books, damaged_books, one_currency_books};
pub use sample::sample;

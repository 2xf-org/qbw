//! What there is in stock, what it is worth, and what happened to it.
//!
//! [`Company::stock`] is the stock now, with what is ordered and planned.
//! [`Company::valuation`] is what there was at the end of a day and what it
//! was worth, and [`Company::movements`] every line that changed an item:
//!
//! ```
//! use qbw::Company;
//! use qbw::chrono::NaiveDate;
//!
//! # let file = qbw_synthetic::company::books().build();
//! let company = Company::from_bytes(file)?;
//! let items = company.items()?;
//! let valve = items.named("Valve").ok_or("no such item")?;
//! let stock = company.stock()?;
//! let row = stock.get(valve.id).ok_or("not in stock")?;
//! assert_eq!((row.value.to_string(), row.needs_reorder()), ("160.00".to_owned(), true));
//!
//! let year_end: NaiveDate = "2023-12-31".parse()?;
//! let year_end = company.valuation(year_end)?;
//! assert_eq!(year_end.value.to_string(), "140.00");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Where the figures come from
//!
//! QuickBooks keeps a history of every item: how much there was after each
//! line, what that was worth and the average cost. Quantities on hand, values
//! and costs are read from it, now and on any day.
//!
//! An order and a build that is pending have no past: the file keeps how much
//! is left of them now. So what is on order, on sales orders, to be built and
//! needed for builds is added up from their lines, and a [`Valuation`] of a day
//! has none of it. The history has such figures for every row, too, and they
//! are those of now.
//!
//! What is at a site or in a bin is added up from the lines of transactions, and
//! so is [`StockRow::counted`], to check the history against. A line that does
//! not say how many is of one.
//!
//! # How sure it is
//!
//! The files of three companies have 2,454 items in stock, and something
//! happened to 2,243 of them.
//!
//! - The history ends with the quantity and the value that QuickBooks keeps
//!   with the item, for all of them, and with the average cost for all that
//!   something happened to.
//! - The lines add up to the quantity for all of them, and to the one in the
//!   history for all of them on five days over ten years. Each of 163,537
//!   movements has the quantity of its line.
//! - What is left of orders and what pending builds add and take agrees for all
//!   but one, of which QuickBooks keeps a sales order that is done.
//! - One company has sites. Of 5,324 quantities at a site or in a bin that
//!   QuickBooks keeps, 5,312 agree. The others are of six items whose kept
//!   quantities do not add up to what there is of the item.
//!
//! [`StockRow::available`] and [`StockRow::needs_reorder`] are inferred.
//!
//! [`Company::movements`]: crate::Company::movements
//! [`Company::stock`]: crate::Company::stock
//! [`Company::valuation`]: crate::Company::valuation

mod history;
mod item_lines;
mod movements;
mod quantity;
mod read;
mod site_stock;
mod stock;
mod stock_movement;
mod stock_row;
mod valuation;
mod valuation_row;

pub use movements::Movements;
pub use site_stock::SiteStock;
pub use stock::Stock;
pub use stock_movement::StockMovement;
pub use stock_row::StockRow;
pub use valuation::Valuation;
pub use valuation_row::ValuationRow;

pub(crate) use history::History;
pub(crate) use item_lines::{ItemLines, OpenLine};
pub(crate) use read::{StockReader, movements, valuation};

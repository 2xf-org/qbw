//! Orders that are not filled yet.
//!
//! [`Company::open_orders`] gives the lines of sales orders or of purchase
//! orders that something is left of:
//!
//! ```
//! use qbw::{Company, Side};
//!
//! # let file = qbw_synthetic::company::books().build();
//! let company = Company::from_bytes(file)?;
//! let orders = company.open_orders(Side::Payable)?;
//! let line = orders.lines.first().ok_or("nothing is on order")?;
//! let quantities = [line.ordered, line.filled, line.open];
//! assert_eq!(quantities.map(|quantity| quantity.to_string()), ["5", "2", "3"]);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! [`Company::open_orders`]: crate::Company::open_orders

mod open_order_line;
mod open_orders;

pub use open_order_line::OpenOrderLine;
pub use open_orders::OpenOrders;

pub(crate) use open_orders::OpenLines;

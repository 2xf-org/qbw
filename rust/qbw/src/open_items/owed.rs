//! What is owed, and what of it is overdue.

use chrono::NaiveDate;

use crate::as_of::AsOf;
use crate::ids::NameId;
use crate::left_out::LeftOut;
use crate::money::Money;
use crate::open_items::items;
use crate::open_items::{OpenItem, OpenItems};
use crate::side::Side;
use crate::totals::Totals;

/// What one customer owes or one vendor is owed at a moment, or all of them
/// together, and what of it is overdue.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Owed {
    /// The customer or the vendor. `None` for all of them together.
    pub name: Option<NameId>,
    /// Receivable or payable.
    pub side: Side,
    /// The moment.
    pub as_of: AsOf,
    /// The day that what is overdue was due before: that of the moment.
    pub day: NaiveDate,
    /// What is open, ordered by date. Those of a customer's jobs are among them.
    pub items: Vec<OpenItem>,
    /// What is owed, less payments and credits that are not used up. QuickBooks
    /// gives a name one currency, so [`Totals::single`] is the whole of it for
    /// one.
    pub total: Totals,
    /// What was due before the day, with nothing taken off.
    pub overdue: Totals,
    /// What is owed, in the company's currency.
    pub home_total: Money,
    /// What was due before the day, in the company's currency.
    pub home_overdue: Money,
    /// What changes of exchange rates added to the accounts for it, in the
    /// company's currency. It belongs to no item: with [`Self::home_total`] it
    /// comes to what is posted to the accounts.
    pub adjustments: Money,
    /// How much could not be read.
    pub left_out: LeftOut,
}

impl Owed {
    /// What `open` has, which is open with `name`. What was due before `day` is
    /// overdue.
    pub(crate) fn new(name: Option<NameId>, day: NaiveDate, open: OpenItems) -> Self {
        let home = open.adjustments.currency();
        let overdue = || open.overdue(day);
        Self {
            name,
            side: open.side,
            as_of: open.as_of,
            day,
            total: open.totals(),
            overdue: items::totals(overdue()),
            home_total: open.home_total(),
            home_overdue: items::home_total(overdue(), home),
            adjustments: open.adjustments,
            left_out: open.left_out,
            items: open.items,
        }
    }
}

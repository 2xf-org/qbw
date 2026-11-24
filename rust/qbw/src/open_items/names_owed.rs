//! What each name owes or is owed.

use std::collections::{BTreeMap, HashMap};

use chrono::NaiveDate;

use crate::amount::Total;
use crate::as_of::AsOf;
use crate::ids::{CurrencyId, NameId};
use crate::left_out::LeftOut;
use crate::money::Money;
use crate::open_items::{OpenItem, OpenItems};
use crate::side::Side;
use crate::totals::Totals;

/// What each customer owes or each vendor is owed, and what of it is overdue, as
/// the list of a center shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct NamesOwed {
    /// Receivable or payable.
    pub side: Side,
    /// The moment.
    pub as_of: AsOf,
    /// The day that what is overdue was due before: that of the moment.
    pub day: NaiveDate,
    /// The names that something is open with, or with a job of theirs, by id. An
    /// item that is with nobody is in no row.
    pub rows: Vec<NameOwed>,
    /// How much could not be read.
    pub left_out: LeftOut,
    /// The currency that the books are kept in.
    home: CurrencyId,
    /// The currency of each name of the side.
    currencies: BTreeMap<NameId, CurrencyId>,
}

/// What a customer with its jobs owes, or a vendor is owed. The figures are those
/// of [`Owed`](crate::open_items::Owed).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct NameOwed {
    /// The customer, the job or the vendor.
    pub name: NameId,
    /// What is owed, less payments and credits that are not used up.
    pub total: Totals,
    /// What was due before the day, with nothing taken off.
    pub overdue: Totals,
    /// What is owed, in the company's currency.
    pub home_total: Money,
    /// What was due before the day, in the company's currency.
    pub home_overdue: Money,
    /// What changes of exchange rates added, in the company's currency.
    pub adjustments: Money,
}

impl NamesOwed {
    /// What a name owes or is owed with its jobs. `None` when nothing is open.
    #[must_use]
    pub fn get(&self, name: NameId) -> Option<&NameOwed> {
        let place = self.rows.binary_search_by_key(&name, |row| row.name);
        self.rows.get(place.ok()?)
    }

    /// What a name owes or is owed with its jobs, also when nothing is open: then
    /// it is nothing in the currency of the name, and nothing in the company's.
    /// `None` for a name that is no customer, or no vendor.
    ///
    /// ```
    /// use qbw::{AsOf, Company, CurrencyId, NameId, Side};
    ///
    /// # let file = qbw_synthetic::company::books().build();
    /// let company = Company::from_bytes(file)?;
    /// let owed = company.names_owed(Side::Receivable, AsOf::Latest)?;
    /// // Bell Works trades in euros, and owes nothing.
    /// let bell = NameId::new(102);
    /// assert_eq!(owed.get(bell), None);
    /// let nothing = owed.owed(bell).ok_or("no such customer")?;
    /// let total = nothing.total.single().ok_or("one currency")?;
    /// assert_eq!((total.to_string(), total.currency()), ("0.00".to_owned(), CurrencyId::new(2)));
    /// assert_eq!(nothing.home_total.currency(), CurrencyId::new(1));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn owed(&self, name: NameId) -> Option<NameOwed> {
        if let Some(owed) = self.get(name) {
            return Some(owed.clone());
        }
        let currency = *self.currencies.get(&name)?;
        let mut sums = Sums::default();
        sums.total.insert(currency, Total::default());
        sums.overdue.insert(currency, Total::default());
        Some(sums.row(name, self.home))
    }

    /// What `open` has of each name, with that of the names below it. `lineage`
    /// gives a name and those above it, and `currencies` are those of every name
    /// of the side.
    pub(crate) fn new(
        open: &OpenItems,
        day: NaiveDate,
        lineage: impl Fn(NameId) -> Vec<NameId>,
        currencies: BTreeMap<NameId, CurrencyId>,
    ) -> Self {
        let mut names: BTreeMap<NameId, Sums> = BTreeMap::new();
        let mut above: HashMap<NameId, Vec<NameId>> = HashMap::new();
        let mut add = |name: NameId, part: &dyn Fn(&mut Sums)| {
            let lineage = above.entry(name).or_insert_with(|| lineage(name));
            for &name in lineage.iter() {
                part(names.entry(name).or_default());
            }
        };
        for item in open {
            if let Some(name) = item.name {
                add(name, &|sums| sums.add(item, day));
            }
        }
        for (&name, &adjusted) in &open.adjusted {
            if let Some(name) = name {
                add(name, &|sums| sums.adjustments += adjusted);
            }
        }
        let home = open.adjustments.currency();
        let rows = names.into_iter().map(|(name, sums)| sums.row(name, home));
        Self {
            side: open.side,
            as_of: open.as_of,
            day,
            rows: rows.collect(),
            left_out: open.amounts_left_out,
            home,
            currencies,
        }
    }
}

/// What is open with a name.
#[derive(Default)]
struct Sums {
    total: BTreeMap<CurrencyId, Total>,
    overdue: BTreeMap<CurrencyId, Total>,
    home_total: Total,
    home_overdue: Total,
    adjustments: Total,
}

impl Sums {
    fn add(&mut self, item: &OpenItem, day: NaiveDate) {
        let (currency, sums) = (item.open.currency(), item.sums());
        *self.total.entry(currency).or_default() += sums.open;
        self.home_total += sums.home_open;
        if item.is_overdue(day) {
            *self.overdue.entry(currency).or_default() += sums.open;
            self.home_overdue += sums.home_open;
        }
    }

    fn row(self, name: NameId, home: CurrencyId) -> NameOwed {
        let totals = |sums: BTreeMap<CurrencyId, Total>| {
            let sums = sums.into_iter();
            Totals::of(sums.map(|(currency, sum)| sum.money(currency)))
        };
        NameOwed {
            name,
            total: totals(self.total),
            overdue: totals(self.overdue),
            home_total: self.home_total.money(home),
            home_overdue: self.home_overdue.money(home),
            adjustments: self.adjustments.money(home),
        }
    }
}

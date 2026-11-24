//! What is owed to a company and by it.

use std::collections::{HashMap, HashSet};

use sa17::Source;

use crate::aging::{Aging, AgingOptions, AgingReader};
use crate::as_of::AsOf;
use crate::company::Company;
use crate::error::Result;
use crate::ids::NameId;
use crate::names::Name;
use crate::open_items::{Moment, NamesOwed, OpenItemReader, OpenItems, Owed, Settling};
use crate::side::Side;

/// # Who owes what
impl<S: Source> Company<S> {
    /// Everything that is open on one side.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::ledger`] and, for a day, of
    /// [`Self::links`].
    pub fn open_items(&self, side: Side, as_of: impl Into<AsOf>) -> Result<OpenItems> {
        self.open_items_of(side, as_of.into(), None)
    }

    /// What a customer owes, and what of it is overdue. What its jobs owe is
    /// part of it. `None` when there is no such customer.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::names`] and [`Self::open_items`].
    pub fn owed_by(&self, customer: NameId, as_of: impl Into<AsOf>) -> Result<Option<Owed>> {
        let customers = self.names()?.customers();
        if customers.get(customer).is_none() {
            return Ok(None);
        }
        let names = customers.subtree(customer);
        let mut owed = self.owed(Side::Receivable, Some((customer, names)), as_of.into())?;
        // A customer that cannot be read may be a job.
        owed.left_out = owed.left_out.union(customers.left_out());
        Ok(Some(owed))
    }

    /// What a vendor is owed, and what of it is overdue. `None` when there is no
    /// such vendor.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::names`] and [`Self::open_items`].
    pub fn owed_to(&self, vendor: NameId, as_of: impl Into<AsOf>) -> Result<Option<Owed>> {
        if self.names()?.vendors().get(vendor).is_none() {
            return Ok(None);
        }
        let names = HashSet::from([vendor]);
        let owed = self.owed(Side::Payable, Some((vendor, names)), as_of.into());
        owed.map(Some)
    }

    /// What each customer owes with its jobs, or each vendor is owed, and what of
    /// it is overdue, all in one pass.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::names`] and [`Self::open_items`].
    pub fn names_owed(&self, side: Side, as_of: impl Into<AsOf>) -> Result<NamesOwed> {
        let as_of = as_of.into();
        let names = self.names()?;
        let customers = names.customers();
        let lineage = |name| match side {
            Side::Receivable => customers.lineage(name),
            Side::Payable => vec![name],
        };
        let currencies = match side {
            Side::Receivable => {
                let customers = customers.iter();
                customers
                    .map(|customer| (customer.id, customer.currency))
                    .collect()
            }
            Side::Payable => {
                let vendors = names.vendors().iter();
                vendors.map(|vendor| (vendor.id, vendor.currency)).collect()
            }
        };
        let open = self.open_items(side, as_of)?;
        let mut owed = NamesOwed::new(&open, self.day(as_of)?, lineage, currencies);
        owed.left_out = owed.left_out.union(names.left_out_on(side));
        Ok(owed)
    }

    /// What is open on one side, by how late it is.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::open_items`] and [`Self::names`].
    pub fn aging(&self, side: Side, options: &AgingOptions) -> Result<Aging> {
        let names = self.names()?;
        let mut places: HashMap<NameId, usize> = HashMap::new();
        places.extend(names.iter().map(Name::id).zip(0..));
        let reader = AgingReader {
            open: self.open_items(side, options.as_of)?,
            day: self.day(options.as_of)?,
            options,
            preferred: self.preferences()?.aging_basis,
            place: &|name| places.get(&name).copied(),
            names_left_out: names.left_out_on(side),
        };
        Ok(reader.read())
    }

    /// What is owed on one side, by or to one name with the names that count as
    /// it, or by or to all of them.
    pub(super) fn owed(
        &self,
        side: Side,
        name: Option<(NameId, HashSet<NameId>)>,
        as_of: AsOf,
    ) -> Result<Owed> {
        let (name, names) = name.unzip();
        let open = self.open_items_of(side, as_of, names)?;
        Ok(Owed::new(name, self.day(as_of)?, open))
    }

    fn open_items_of(
        &self,
        side: Side,
        as_of: AsOf,
        names: Option<HashSet<NameId>>,
    ) -> Result<OpenItems> {
        let reader = OpenItemReader {
            ledger: self.ledger()?,
            accounts: self.accounts()?,
            headers: self.header_index()?,
            side,
            moment: match as_of {
                AsOf::Latest => Moment::Latest,
                AsOf::Date(day) => Moment::Date(Settling {
                    day,
                    links: self.link_index()?,
                    dates: self.link_dates()?,
                }),
            },
            names,
        };
        Ok(reader.read())
    }
}

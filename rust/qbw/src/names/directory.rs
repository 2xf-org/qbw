//! The directory of every name.

use sa17::Source;

use crate::case;
use crate::damage::Damage;
use crate::error::Result;
use crate::ids::{CurrencyId, NameId};
use crate::left_out::LeftOut;
use crate::list::List;
use crate::names::details::DetailsByName;
use crate::names::{
    Customer, Employee, Name, OtherName, Vendor, customer, employee, other_name, vendor,
};
use crate::side::Side;

/// Every name a transaction can be with: the customers and jobs, the vendors,
/// the employees and the other names.
///
/// The four lists share one set of ids, so an id alone finds its name.
#[derive(Clone, Debug)]
pub struct Names {
    customers: List<Customer>,
    vendors: List<Vendor>,
    employees: List<Employee>,
    other_names: List<OtherName>,
    /// What could not be read of the contacts and addresses.
    damage: Vec<Damage>,
}

impl Names {
    /// The name with `id`, whichever list it is in.
    #[must_use]
    pub fn get(&self, id: NameId) -> Option<Name<'_>> {
        let customer = self.customers.get(id).map(Name::Customer);
        customer
            .or_else(|| self.vendors.get(id).map(Name::Vendor))
            .or_else(|| self.employees.get(id).map(Name::Employee))
            .or_else(|| self.other_names.get(id).map(Name::Other))
    }

    /// The name with `id` as QuickBooks writes it: a job's after the
    /// customer's, as in `Customer:Job`.
    #[must_use]
    pub fn full_name(&self, id: NameId) -> Option<&str> {
        let customer = self.customers.full_name(id);
        customer
            .or_else(|| self.vendors.full_name(id))
            .or_else(|| self.employees.full_name(id))
            .or_else(|| self.other_names.full_name(id))
    }

    /// The name with `id` itself: a job's without the customer's.
    #[must_use]
    pub fn own_name(&self, id: NameId) -> Option<&str> {
        let customer = self.customers.own_name(id);
        customer
            .or_else(|| self.vendors.own_name(id))
            .or_else(|| self.employees.own_name(id))
            .or_else(|| self.other_names.own_name(id))
    }

    /// Every name in QuickBooks' order, inactive ones among them: list after
    /// list, as they are below.
    pub fn iter(&self) -> impl Iterator<Item = Name<'_>> {
        let customers = self.customers.iter().map(Name::Customer);
        let vendors = self.vendors.iter().map(Name::Vendor);
        let employees = self.employees.iter().map(Name::Employee);
        let other_names = self.other_names.iter().map(Name::Other);
        customers.chain(vendors).chain(employees).chain(other_names)
    }

    /// The name called `name`, ignoring case, in whichever list: by its full
    /// name, or else by its own, where no other name has that.
    /// [`Self::all_named`] tells what several names are called from what none is.
    #[must_use]
    pub fn named(&self, name: &str) -> Option<Name<'_>> {
        let name = case::folded(name);
        self.with_full_name(&name).next().or_else(|| {
            let mut names = self.with_own_name(&name);
            let found = names.next();
            found.filter(|_| names.next().is_none())
        })
    }

    /// Every name that `name` can mean, ignoring case, in whichever list: the ones
    /// with that full name, or else the ones whose own name it is, in the order of
    /// [`Self::iter`].
    ///
    /// ```
    /// use qbw::Company;
    ///
    /// # let file = qbw_synthetic::company::books().build();
    /// let company = Company::from_bytes(file)?;
    /// let names = company.names()?;
    /// assert_eq!(names.all_named("ada stores").len(), 1);
    /// assert!(names.all_named("nobody").is_empty());
    /// # Ok::<(), qbw::Error>(())
    /// ```
    #[must_use]
    pub fn all_named(&self, name: &str) -> Vec<Name<'_>> {
        let name = case::folded(name);
        let mut found: Vec<Name<'_>> = self.with_full_name(&name).collect();
        if found.is_empty() {
            found.extend(self.with_own_name(&name));
        }
        found
    }

    /// The names with a full name, which is given without its case.
    fn with_full_name(&self, name: &str) -> impl Iterator<Item = Name<'_>> {
        self.iter().filter(move |found| {
            let full_name = self.full_name(found.id());
            full_name.is_some_and(|full_name| case::folded(full_name) == name)
        })
    }

    /// The names with a name of their own, which is given without its case.
    fn with_own_name(&self, name: &str) -> impl Iterator<Item = Name<'_>> {
        let names = self.iter();
        names.filter(move |found| case::folded(found.name()) == name)
    }

    /// Whether the customer or the vendor `name` is in the list of a side.
    pub(crate) fn is_on(&self, side: Side, name: NameId) -> bool {
        match side {
            Side::Receivable => self.customers.get(name).is_some(),
            Side::Payable => self.vendors.get(name).is_some(),
        }
    }

    /// How much could not be read of the list of a side: of the customers, or of
    /// the vendors.
    pub(crate) fn left_out_on(&self, side: Side) -> LeftOut {
        match side {
            Side::Receivable => self.customers.left_out(),
            Side::Payable => self.vendors.left_out(),
        }
    }

    /// The customers and their jobs. A job has the customer as its parent.
    #[must_use]
    pub fn customers(&self) -> &List<Customer> {
        &self.customers
    }

    /// The vendors.
    #[must_use]
    pub fn vendors(&self) -> &List<Vendor> {
        &self.vendors
    }

    /// The employees.
    #[must_use]
    pub fn employees(&self) -> &List<Employee> {
        &self.employees
    }

    /// The names that are none of these.
    #[must_use]
    pub fn other_names(&self) -> &List<OtherName> {
        &self.other_names
    }

    /// The number of names.
    #[must_use]
    pub fn len(&self) -> usize {
        self.iter().count()
    }

    /// Whether there are no names.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// How much could not be read, the contacts and addresses included.
    #[must_use]
    pub fn left_out(&self) -> LeftOut {
        LeftOut::of(&self.damage)
            .with(self.customers.left_out())
            .with(self.vendors.left_out())
            .with(self.employees.left_out())
            .with(self.other_names.left_out())
    }

    /// What could not be read, the contacts and addresses included.
    pub fn damage(&self) -> impl Iterator<Item = &Damage> {
        let lists = [
            self.customers.damage(),
            self.vendors.damage(),
            self.employees.damage(),
            self.other_names.damage(),
        ];
        lists.into_iter().flatten().chain(&self.damage)
    }
}

/// Reads the names of a company that keeps its books in `home`.
///
/// # Errors
///
/// Returns an error when the file lacks a table or a column of it.
pub(crate) fn read(source: &dyn Source, home: CurrencyId) -> Result<Names> {
    let mut damage = Vec::new();
    let mut details = DetailsByName::read(source, &mut damage)?;
    let customers = customer::read(source, home, &mut details)?;
    let vendors = vendor::read(source, home, &mut details)?;
    let employees = employee::read(source, &mut details)?;
    let other_names = other_name::read(source, &mut details)?;
    let links = [
        customers.links.as_slice(),
        &vendors.links,
        &employees.links,
        &other_names.links,
    ];
    let links = links.concat();
    Ok(Names {
        customers: customers.sorted(&links),
        vendors: vendors.sorted(&links),
        employees: employees.sorted(&links),
        other_names: other_names.sorted(&links),
        damage,
    })
}

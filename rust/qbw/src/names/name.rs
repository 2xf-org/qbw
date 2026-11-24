//! A name of any of the four lists.

use crate::ids::{NameId, OwnerKey};
use crate::list::ListEntry;
use crate::names::{Contact, Customer, Employee, NameKind, OtherName, Vendor};
use crate::origin::Origin;

/// A name of any of the four lists, lent by [`Names`](crate::names::Names).
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum Name<'names> {
    /// A customer or a job.
    Customer(&'names Customer),
    /// A vendor.
    Vendor(&'names Vendor),
    /// An employee.
    Employee(&'names Employee),
    /// Another name.
    Other(&'names OtherName),
}

impl<'names> Name<'names> {
    /// What the name is known by.
    #[must_use]
    pub const fn id(self) -> NameId {
        match self {
            Self::Customer(customer) => customer.id,
            Self::Vendor(vendor) => vendor.id,
            Self::Employee(employee) => employee.id,
            Self::Other(other) => other.id,
        }
    }

    /// The list it is in.
    #[must_use]
    pub const fn kind(self) -> NameKind {
        match self {
            Self::Customer(_) => NameKind::Customer,
            Self::Vendor(_) => NameKind::Vendor,
            Self::Employee(_) => NameKind::Employee,
            Self::Other(_) => NameKind::Other,
        }
    }

    /// The name itself. A job's is without the customer's.
    #[must_use]
    pub fn name(self) -> &'names str {
        match self {
            Self::Customer(customer) => &customer.name,
            Self::Vendor(vendor) => &vendor.name,
            Self::Employee(employee) => &employee.name,
            Self::Other(other) => &other.name,
        }
    }

    /// The contact that is in use.
    #[must_use]
    pub const fn contact(self) -> Option<&'names Contact> {
        match self {
            Self::Customer(customer) => customer.contact.as_ref(),
            Self::Vendor(vendor) => vendor.contact.as_ref(),
            Self::Employee(employee) => employee.contact.as_ref(),
            Self::Other(other) => other.contact.as_ref(),
        }
    }

    /// Whether it can be used in new transactions.
    #[must_use]
    pub fn is_active(self) -> bool {
        match self {
            Self::Customer(customer) => customer.is_active(),
            Self::Vendor(vendor) => vendor.is_active(),
            Self::Employee(employee) => employee.is_active(),
            Self::Other(other) => other.is_active(),
        }
    }

    /// What its custom fields are kept by.
    #[must_use]
    pub const fn owner_key(self) -> Option<OwnerKey> {
        match self {
            Self::Customer(customer) => customer.owner_key,
            Self::Vendor(vendor) => vendor.owner_key,
            Self::Employee(employee) => employee.owner_key,
            Self::Other(other) => other.owner_key,
        }
    }

    /// The stored row.
    #[must_use]
    pub const fn origin(self) -> Origin {
        match self {
            Self::Customer(customer) => customer.origin,
            Self::Vendor(vendor) => vendor.origin,
            Self::Employee(employee) => employee.origin,
            Self::Other(other) => other.origin,
        }
    }
}

//! The small lists, all alike.

use std::fmt;

use crate::left_out::LeftOut;
use crate::list::{List, ListEntry, ListId};
use crate::origin::Origin;

/// One of the small lists.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ListKind {
    /// Categories of items.
    Categories,
    /// Classes.
    Classes,
    /// Currencies.
    Currencies,
    /// Messages to customers.
    CustomerMessages,
    /// Types of customers.
    CustomerTypes,
    /// Types of jobs.
    JobTypes,
    /// Memorized transactions.
    Memorized,
    /// Payment methods.
    PaymentMethods,
    /// Price levels.
    PriceLevels,
    /// Sales reps.
    SalesReps,
    /// Sales tax codes.
    SalesTaxCodes,
    /// Ways to ship goods.
    ShipMethods,
    /// Sites that stock is kept at.
    Sites,
    /// Templates of forms.
    Templates,
    /// Terms of payment.
    Terms,
    /// Types of vendors.
    VendorTypes,
}

impl ListKind {
    /// Every one of them.
    pub const ALL: &'static [Self] = &[
        Self::Categories,
        Self::Classes,
        Self::Currencies,
        Self::CustomerMessages,
        Self::CustomerTypes,
        Self::JobTypes,
        Self::Memorized,
        Self::PaymentMethods,
        Self::PriceLevels,
        Self::SalesReps,
        Self::SalesTaxCodes,
        Self::ShipMethods,
        Self::Sites,
        Self::Templates,
        Self::Terms,
        Self::VendorTypes,
    ];
}

impl fmt::Display for ListKind {
    /// Writes what the list is called, such as `Customer Type List`.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.pad(match self {
            Self::Categories => "Category List",
            Self::Classes => "Class List",
            Self::Currencies => "Currency List",
            Self::CustomerMessages => "Customer Message List",
            Self::CustomerTypes => "Customer Type List",
            Self::JobTypes => "Job Type List",
            Self::Memorized => "Memorized Transaction List",
            Self::PaymentMethods => "Payment Method List",
            Self::PriceLevels => "Price Level List",
            Self::SalesReps => "Sales Rep List",
            Self::SalesTaxCodes => "Sales Tax Code List",
            Self::ShipMethods => "Ship Via List",
            Self::Sites => "Inventory Site List",
            Self::Templates => "Templates",
            Self::Terms => "Terms List",
            Self::VendorTypes => "Vendor Type List",
        })
    }
}

/// A small list with what every entry has, whichever list it is.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SmallList<'company> {
    /// Which list it is.
    pub kind: ListKind,
    /// The entries as [`List::walk`] gives them.
    pub entries: Vec<SmallListEntry<'company>>,
    /// How much could not be read.
    pub left_out: LeftOut,
}

/// An entry of a [`SmallList`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SmallListEntry<'company> {
    /// The number of its id, which no other entry of the list has.
    pub number: i32,
    /// Its own name.
    pub name: &'company str,
    /// Its name after those of the entries above it.
    pub full_name: &'company str,
    /// The number of the entry above it.
    pub parent: Option<i32>,
    /// How many entries it is below.
    pub depth: usize,
    /// Whether it can be used in new transactions.
    pub is_active: bool,
    /// The stored row.
    pub origin: Origin,
}

impl<'company> SmallList<'company> {
    pub(crate) fn new<T: ListEntry>(kind: ListKind, list: &'company List<T>) -> Self {
        let entries = list.walk().map(|(depth, entry)| SmallListEntry {
            number: entry.id().number(),
            name: entry.name(),
            full_name: list.full_name(entry.id()).unwrap_or_else(|| entry.name()),
            parent: entry.parent().map(ListId::number),
            depth,
            is_active: entry.is_active(),
            origin: entry.origin(),
        });
        Self {
            kind,
            entries: entries.collect(),
            left_out: list.left_out(),
        }
    }
}

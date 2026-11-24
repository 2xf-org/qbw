//! The links of a transaction.

use std::slice;

use crate::amount::Total;
use crate::ids::{CurrencyId, TransactionId};
use crate::left_out::LeftOut;
use crate::links::{Link, Relation};
use crate::money::Money;

/// The links of a transaction, one for each other transaction and
/// [`Relation`]. They are ordered by their relation, and then by the date of
/// the other.
///
/// A link to a transaction whose header cannot be read is left out. The header
/// counts once, however many links lead to it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Links {
    /// The transaction.
    pub transaction: TransactionId,
    /// The currency of the amounts, which is the transaction's.
    pub currency: CurrencyId,
    /// The links in order.
    pub links: Vec<Link>,
    /// How much could not be read.
    pub left_out: LeftOut,
}

impl Links {
    /// The links in order.
    pub fn iter(&self) -> slice::Iter<'_, Link> {
        self.links.iter()
    }

    /// The links of one relation.
    pub fn of(&self, relation: Relation) -> impl Iterator<Item = &Link> {
        self.iter().filter(move |link| link.relation == relation)
    }

    /// The links whose relation `keeps` says yes to, with what they come to. One
    /// in a way without a name may be among them, and is left out.
    pub(crate) fn settled(mut self, keeps: impl Fn(Relation) -> bool) -> Settled {
        let unknown = self.of(Relation::LinkedTo).map(|_| 1).sum();
        self.links.retain(|link| keeps(link.relation));
        let total: Total = self.links.iter().map(|link| link.sum).sum();
        Settled {
            transaction: self.transaction,
            total: total.money(self.currency),
            links: self.links,
            left_out: self.left_out.with(LeftOut::of_rows(unknown)),
        }
    }
}

impl<'links> IntoIterator for &'links Links {
    type Item = &'links Link;
    type IntoIter = slice::Iter<'links, Link>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// What settled a transaction, or what a transaction settled. The links are in
/// the order of [`Links`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Settled {
    /// The transaction.
    pub transaction: TransactionId,
    /// The links in order.
    pub links: Vec<Link>,
    /// What they come to, in the transaction's currency.
    pub total: Money,
    /// How much could not be read.
    pub left_out: LeftOut,
}

impl Settled {
    /// The links in order.
    pub fn iter(&self) -> slice::Iter<'_, Link> {
        self.links.iter()
    }
}

impl<'links> IntoIterator for &'links Settled {
    type Item = &'links Link;
    type IntoIter = slice::Iter<'links, Link>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

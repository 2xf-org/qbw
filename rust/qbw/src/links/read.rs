//! Putting the links of a transaction together.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use sa17::Source;

use crate::amount::Total;
use crate::codes::{LinkCode, TransactionKind};
use crate::error::Result;
use crate::ids::{CurrencyId, TransactionId};
use crate::links::stored_link::{StoredLink, Worth};
use crate::links::{LineLink, LineLinks, Link, LinkIndex, LinkedTransaction, Links, Relation};
use crate::tables::links::HeaderCurrencyRow;
use crate::transactions::{Header, Headers, ShownHeaders};

/// Puts the links of transactions together.
pub(crate) struct LinkReader<'company> {
    pub(crate) source: &'company dyn Source,
    pub(crate) index: &'company LinkIndex,
    pub(crate) headers: &'company Headers,
    /// The currency that the books are kept in.
    pub(crate) home: CurrencyId,
}

impl LinkReader<'_> {
    /// The links of a transaction to others. `None` when there is no such
    /// transaction.
    ///
    /// # Errors
    ///
    /// Returns an error when the transaction's header cannot be read again.
    pub(crate) fn links(&self, transaction: TransactionId) -> Result<Option<Links>> {
        if !self.is_known(transaction) {
            return Ok(None);
        }
        let mut headers = ShownHeaders::new(self.headers);
        let own = headers.of_any_kind(transaction);
        let currency = self.currency(own)?;
        let mut groups: BTreeMap<Place, Group> = BTreeMap::new();
        for link in self.index.of(transaction) {
            let Some(other) = headers.of_any_kind(link.other) else {
                continue;
            };
            let place = Place {
                relation: relation(link, own),
                date: other.date,
                other: other.id,
            };
            groups.entry(place).or_default().add(link);
        }
        let links = groups.into_iter().filter_map(|(place, group)| {
            let other = self.headers.get(place.other)?;
            group.link(place.relation, other, currency)
        });
        Ok(Some(Links {
            transaction,
            currency,
            links: links.collect(),
            left_out: self.index.left_out().with(headers.left_out()),
        }))
    }

    /// The links of a transaction's lines to lines of others. `None` when there
    /// is no such transaction.
    ///
    /// # Errors
    ///
    /// Returns an error when the transaction's header cannot be read again.
    pub(crate) fn line_links(&self, transaction: TransactionId) -> Result<Option<LineLinks>> {
        if !self.is_known(transaction) {
            return Ok(None);
        }
        let mut headers = ShownHeaders::new(self.headers);
        let currency = self.currency(headers.of_any_kind(transaction))?;
        let links = self.index.of(transaction).iter();
        let links = links.map(|link| line_link(link, currency));
        Ok(Some(LineLinks {
            transaction,
            links: links.collect(),
            left_out: self.index.left_out().with(headers.left_out()),
        }))
    }

    /// Whether there is a transaction, be it one whose header cannot be read.
    fn is_known(&self, transaction: TransactionId) -> bool {
        self.headers.get(transaction).is_some() || !self.index.of(transaction).is_empty()
    }

    /// The currency of the transaction of `header`, or the company's when that is
    /// lost.
    fn currency(&self, header: Option<&Header>) -> Result<CurrencyId> {
        let Some(header) = header else {
            return Ok(self.home);
        };
        let row = match header.form.headers_in::<HeaderCurrencyRow>(self.source)? {
            Some(table) => table.fetch(header.row)?,
            None => None,
        };
        let currency = row.and_then(|row| row.currency);
        Ok(currency.unwrap_or(self.home))
    }
}

/// Where a group of links stands among the others.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Place {
    relation: Relation,
    date: NaiveDate,
    other: TransactionId,
}

/// The links to one transaction, of one relation.
#[derive(Default)]
struct Group {
    amount: Option<Total>,
    quantity: Option<Total>,
    /// Whether the group stands even when it is worth nothing.
    is_kept: bool,
}

impl Group {
    fn add(&mut self, link: &StoredLink) {
        match link.worth() {
            Worth::Amount(amount) => {
                *self.amount.get_or_insert_default() += amount.into();
            }
            Worth::Quantity(quantity) => {
                *self.quantity.get_or_insert_default() += quantity.into();
            }
            Worth::Nothing => {}
        }
        // An invoice of an order can be of nothing but texts.
        self.is_kept |= link.code() == LinkCode::InvoicedAmount;
    }

    /// The link, unless it comes to nothing.
    fn link(&self, relation: Relation, other: &Header, currency: CurrencyId) -> Option<Link> {
        let sum = self.amount.unwrap_or_default();
        let quantity = self.quantity.filter(|quantity| !quantity.is_zero());
        if sum.is_zero() && quantity.is_none() && !self.is_kept {
            return None;
        }
        Some(Link {
            relation,
            other: LinkedTransaction::new(other),
            amount: self.amount.map(|amount| amount.money(currency)),
            quantity: quantity.map(Total::quantity),
            sum,
        })
    }
}

/// How the transaction of `header`, which is lost where it is `None`, belongs to
/// the other end of a link.
fn relation(link: &StoredLink, header: Option<&Header>) -> Relation {
    let kind = header.map_or(TransactionKind::Other(0), |header| header.kind);
    let ends = Relation::of(link.code(), kind);
    if link.is_source {
        ends.source
    } else {
        ends.target
    }
}

fn line_link(link: &StoredLink, currency: CurrencyId) -> LineLink {
    let (amount, quantity) = match link.worth() {
        Worth::Amount(amount) => (Some(amount.money(currency)), None),
        Worth::Quantity(quantity) => (None, Some(quantity.quantity())),
        Worth::Nothing => (None, None),
    };
    LineLink {
        line: link.line(),
        other: link.other,
        other_line: link.other_line(),
        code: link.code(),
        is_source: link.is_source,
        amount,
        quantity,
    }
}

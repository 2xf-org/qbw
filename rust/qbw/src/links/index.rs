//! The links of every transaction.

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use sa17::Source;

use crate::codes::LinkCode;
use crate::damage::Damage;
use crate::error::Result;
use crate::ids::TransactionId;
use crate::left_out::LeftOut;
use crate::links::stored_link::{Ends, StoredLink, Worth};
use crate::tables::links::{LINKS, LinkRow};

/// The links of every transaction to others, without the ones that QuickBooks
/// shows nowhere: those that say again what another says, those that are for
/// information only, and those of a transaction to itself.
#[derive(Debug)]
pub(crate) struct LinkIndex {
    /// By transaction and by the other one.
    links: Box<[StoredLink]>,
    damage: Vec<Damage>,
    /// How many links have a code that this version has no name for.
    unknown: u32,
}

impl LinkIndex {
    /// Reads the links of `source`.
    ///
    /// # Errors
    ///
    /// Returns an error when the file lacks the table or a column of it.
    pub(crate) fn read(source: &dyn Source) -> Result<Self> {
        let mut damage = Vec::new();
        let mut repeated = Vec::new();
        let mut seen = HashSet::new();
        let mut links: Vec<StoredLink> = Vec::new();
        for (origin, row) in LINKS.open(source)?.scan(&mut damage) {
            if seen.insert(row.id) {
                links.extend(link(&row));
            } else {
                repeated.push(Damage::repeated(origin));
            }
        }
        damage.append(&mut repeated);
        let mut links = links.into_boxed_slice();
        links.sort_unstable_by_key(StoredLink::place);
        let is_unknown = |link: &&StoredLink| matches!(link.code(), LinkCode::Other(_));
        Ok(Self {
            unknown: links.iter().filter(is_unknown).map(|_| 1).sum(),
            links,
            damage,
        })
    }

    /// The links of a transaction, by the other one.
    pub(crate) fn of(&self, transaction: TransactionId) -> &[StoredLink] {
        let start = self
            .links
            .partition_point(|link| link.transaction < transaction);
        let end = self
            .links
            .partition_point(|link| link.transaction <= transaction);
        self.links.get(start..end).unwrap_or_default()
    }

    /// How much could not be read.
    pub(crate) fn left_out(&self) -> LeftOut {
        LeftOut::of(&self.damage)
    }

    /// What could not be read.
    pub(crate) fn damage(&self) -> &[Damage] {
        &self.damage
    }

    /// The links whose code this version has no name for. Nobody can tell whether
    /// they settle something.
    pub(crate) fn unknown(&self) -> LeftOut {
        LeftOut::of_rows(self.unknown)
    }
}

/// The dates that transactions with links post on.
#[derive(Debug)]
pub(crate) struct LinkDates {
    dates: HashMap<TransactionId, NaiveDate>,
}

impl LinkDates {
    /// The dates of the transactions of `links`, of all those of `posted`, which
    /// are by id.
    pub(crate) fn of(
        links: &LinkIndex,
        posted: impl Iterator<Item = (TransactionId, NaiveDate)>,
    ) -> Self {
        let mut posted = posted.peekable();
        let mut dates = HashMap::new();
        let by_transaction =
            |left: &StoredLink, right: &StoredLink| left.transaction == right.transaction;
        for linked in links.links.chunk_by(by_transaction) {
            let Some(StoredLink { transaction, .. }) = linked.first() else {
                continue;
            };
            while posted.next_if(|(id, _)| id < transaction).is_some() {}
            dates.extend(posted.next_if(|(id, _)| id == transaction));
        }
        Self { dates }
    }

    /// The date of a transaction. `None` when it posts nothing, and when it has no
    /// link of its own that can be read: then it is in the header.
    pub(crate) fn get(&self, transaction: TransactionId) -> Option<NaiveDate> {
        self.dates.get(&transaction).copied()
    }
}

/// The link of a row, unless QuickBooks shows it nowhere.
fn link(row: &LinkRow) -> Option<StoredLink> {
    let other = row.other.filter(|&other| other != row.transaction)?;
    if row.is_secondary || row.is_information_only {
        return None;
    }
    let ends = Ends {
        transaction: row.transaction,
        line: row.line,
        other,
        other_line: row.other_line,
        is_source: row.is_source,
    };
    let worth = match (row.amount, row.quantity) {
        (Some(amount), _) => Worth::Amount(-amount),
        (None, Some(quantity)) => Worth::Quantity(quantity),
        (None, None) => Worth::Nothing,
    };
    Some(StoredLink::new(&ends, row.code, worth))
}

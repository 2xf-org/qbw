//! Working out what is open.

use std::collections::{BTreeMap, HashSet};

use crate::accounts::Account;
use crate::amount::Total;
use crate::as_of::AsOf;
use crate::ids::NameId;
use crate::ledger::{Entry, Ledger};
use crate::left_out::LeftOut;
use crate::list::List;
use crate::open_items::part::{Part, Settling, TooLarge};
use crate::open_items::{OpenItem, OpenItems};
use crate::side::Side;
use crate::transactions::{Headers, ShownHeaders};

/// Works out what is open on one side.
pub(crate) struct OpenItemReader<'company> {
    pub(crate) ledger: &'company Ledger,
    pub(crate) accounts: &'company List<Account>,
    pub(crate) headers: &'company Headers,
    pub(crate) side: Side,
    pub(crate) moment: Moment<'company>,
    /// The only names to look at. `None` is all of them.
    pub(crate) names: Option<HashSet<NameId>>,
}

/// What is open, with how much could not be read.
struct Found {
    items: Vec<OpenItem>,
    /// How much of what the amounts rest on.
    amounts_left_out: LeftOut,
    /// How much in all.
    left_out: LeftOut,
}

/// The moment that something is open at, with what working that out takes.
#[derive(Clone, Copy)]
pub(crate) enum Moment<'company> {
    /// Now, as the file keeps it.
    Latest,
    /// The end of a day, by the links.
    Date(Settling<'company>),
}

impl<'company> OpenItemReader<'company> {
    pub(crate) fn read(&self) -> OpenItems {
        let mut adjustments: BTreeMap<Option<NameId>, Total> = BTreeMap::new();
        let mut lines = Vec::new();
        for account in self.accounts {
            if account.account_type != self.side.account_type() {
                continue;
            }
            for entry in self
                .ledger
                .entries_during(account.id, self.moment.as_of().period())
            {
                // Only the rate of the account's currency changed.
                if entry.own(account.currency, self.ledger.home()).is_zero() {
                    let name = entry.name_on(self.side);
                    if self.is_wanted(name) {
                        let adjusted = self.side.owed(Total::from(entry.home));
                        *adjustments.entry(name).or_default() += adjusted;
                    }
                } else {
                    lines.push((entry, account));
                }
            }
        }
        let Found {
            items,
            amounts_left_out,
            left_out,
        } = self.find(lines);
        OpenItems {
            side: self.side,
            as_of: self.moment.as_of(),
            items,
            adjustments: adjustments
                .values()
                .copied()
                .sum::<Total>()
                .money(self.ledger.home()),
            adjusted: adjustments,
            left_out,
            amounts_left_out,
        }
    }

    /// Whether a name is one to look at.
    fn is_wanted(&self, name: Option<NameId>) -> bool {
        match &self.names {
            Some(names) => name.is_some_and(|name| names.contains(&name)),
            None => true,
        }
    }

    /// The items of entries, each given with its account.
    fn find(&self, mut lines: Vec<(&Entry, &Account)>) -> Found {
        lines.sort_unstable_by_key(|(entry, account)| {
            let name = entry.name_on(self.side);
            let own = entry.own(account.currency, self.ledger.home());
            (
                entry.transaction,
                account.id,
                name,
                own.is_negative(),
                entry.source,
                entry.line,
            )
        });
        let mut items = Vec::new();
        let mut too_large = 0;
        let mut dated = ShownHeaders::new(self.headers);
        let mut numbered = ShownHeaders::new(self.headers);
        for lines in lines.chunk_by(|(left, _), (right, _)| left.transaction == right.transaction) {
            let parts = Part::of_transaction(lines, self.side, self.ledger);
            for (place, part) in parts.iter().enumerate() {
                if !self.is_wanted(part.name()) {
                    continue;
                }
                let rest = match self.moment {
                    Moment::Latest => part.rest(self.ledger),
                    Moment::Date(settling) => part.unsettled(place, &parts, settling, &mut dated),
                };
                match part.item(rest, self.ledger, &mut numbered) {
                    Ok(item) => items.extend(item),
                    Err(TooLarge) => too_large += 1,
                }
            }
        }
        items.sort_by_key(|item| (item.date, item.transaction));
        let left_out = self.left_out().with(LeftOut::of_rows(too_large));
        let amounts_left_out = left_out.with(dated.left_out());
        numbered.join(dated);
        Found {
            items,
            amounts_left_out,
            left_out: left_out.with(numbered.left_out()),
        }
    }

    /// How much could not be read of what the amounts of every item rest on.
    fn left_out(&self) -> LeftOut {
        let links = match self.moment {
            Moment::Latest => LeftOut::default(),
            Moment::Date(settling) => settling.links.left_out().with(settling.links.unknown()),
        };
        self.ledger.left_out().with(links)
    }
}

impl Moment<'_> {
    const fn as_of(self) -> AsOf {
        match self {
            Self::Latest => AsOf::Latest,
            Self::Date(settling) => AsOf::Date(settling.day),
        }
    }
}

//! The search for transactions.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::RangeBounds;

use sa17::Source;

use crate::accounts::Account;
use crate::amount::Total;
use crate::case;
use crate::codes::TransactionKind;
use crate::damage::{Damage, Place};
use crate::error::Result;
use crate::ids::{CurrencyId, NameId, TransactionId};
use crate::ledger::Ledger;
use crate::left_out::LeftOut;
use crate::list::List;
use crate::money::Money;
use crate::names::Names;
use crate::page::Page;
use crate::query::memos::Memos;
use crate::query::natural::Natural;
use crate::query::{
    KindCount, PaidState, TransactionOrder, TransactionPage, TransactionQuery, TransactionSummary,
};
use crate::tables::Form;
use crate::transactions::{Header, Headers, Left, OpenState, OrderProgress, OwnLines, OwnSide};

/// A search for transactions, with what it reads.
pub(crate) struct Search<'company> {
    pub(crate) query: &'company TransactionQuery,
    pub(crate) source: &'company dyn Source,
    /// The currency the books are kept in.
    pub(crate) home: CurrencyId,
    pub(crate) headers: &'company Headers,
    pub(crate) accounts: &'company List<Account>,
    /// The names, for a query that is about them.
    pub(crate) names: Option<&'company Names>,
    /// The ledger, for a query that is about what is posted.
    pub(crate) ledger: Option<&'company Ledger>,
    /// Reads the own lines of a form when they are first needed.
    pub(crate) own_lines: &'company dyn Fn(Form) -> Result<&'company OwnLines>,
}

impl<'company> Search<'company> {
    /// The page `page` of what the query finds.
    ///
    /// # Errors
    ///
    /// Returns an error when a table lacks a column that it has in every file.
    pub(crate) fn page(&self, page: Page) -> Result<TransactionPage> {
        let mut memos = Memos::new(self.source);
        let found = self.found(&mut memos)?;
        let shown = found.get(page.of(found.len())).unwrap_or_default();
        let rows = shown.iter().map(|header| self.summary(header, &mut memos));
        let rows: Vec<TransactionSummary> = rows.collect::<Result<_>>()?;
        Ok(TransactionPage {
            rows,
            row_count: found.len(),
            kinds: kinds(&found),
            left_out: self.left_out(shown)?.with(LeftOut::of(&memos.damage)),
        })
    }

    /// The transaction with `id` as a list shows it, whatever the query is.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::page`].
    pub(crate) fn one(&self, id: TransactionId) -> Result<Option<TransactionSummary>> {
        let Some(header) = self.headers.get(id) else {
            return Ok(None);
        };
        let mut memos = Memos::new(self.source);
        let mut summary = self.summary(header, &mut memos)?;
        let lost_memo = LeftOut::of(&memos.damage);
        summary.left_out = summary.left_out.with(lost_memo);
        Ok(Some(summary))
    }

    /// What the query finds.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::page`].
    pub(crate) fn ids(&self) -> Result<Vec<TransactionId>> {
        let found = self.found(&mut Memos::new(self.source))?;
        Ok(found.iter().map(|header| header.id).collect())
    }

    /// The transactions that the query asks for, in order.
    fn found(&self, memos: &mut Memos<'company>) -> Result<Vec<&'company Header>> {
        let mut found = self.by_header();
        self.keep_by_postings(&mut found);
        self.keep_by_own_line(&mut found)?;
        self.keep_by_text(&mut found, memos)?;
        self.sort(&mut found)?;
        Ok(found)
    }

    /// The transactions that the query admits by their headers.
    fn by_header(&self) -> Vec<&'company Header> {
        let query = self.query;
        let names = self.names_asked_for();
        let admits = |header: &&Header| {
            (query.kinds.is_empty() || query.kinds.contains(&header.kind))
                && query.posting.admits(header.posting)
                && query.period.contains(header.date)
                && names.as_ref().is_none_or(|names| {
                    let name = header.name();
                    name.is_some_and(|name| names.contains(&name))
                })
                && query.number.as_ref().is_none_or(|number| {
                    let own = header.number.as_deref();
                    own.is_some_and(|own| case::is_same(own, number))
                })
        };
        self.headers.iter().filter(admits).collect()
    }

    /// The name that the query asks for, with its jobs when it asks for them.
    fn names_asked_for(&self) -> Option<HashSet<NameId>> {
        let asked = self.query.name?;
        let customers = self.names.map(Names::customers);
        let customers = customers.filter(|_| self.query.with_jobs);
        let names = customers.map(|customers| customers.subtree(asked));
        Some(names.unwrap_or_else(|| HashSet::from([asked])))
    }

    /// Drops what does not post to the account that the query asks for, or posts
    /// another amount to it.
    fn keep_by_postings(&self, found: &mut Vec<&'company Header>) {
        let (Some(account), Some(ledger)) = (self.query.posts_to, self.ledger) else {
            return;
        };
        let currency = ledger.currency(account);
        let mut posted: HashMap<TransactionId, Total> = HashMap::new();
        for entry in ledger.entries(account) {
            let amount = entry.own(currency, ledger.home());
            *posted.entry(entry.transaction).or_default() += amount.into();
        }
        found.retain(|header| {
            let posted = posted.get(&header.id);
            posted.is_some_and(|posted| self.admits(posted.money(currency)))
        });
    }

    /// Whether an amount is one that the query asks for.
    fn admits(&self, amount: Money) -> bool {
        self.query.amount.contains(&amount.abs().amount())
    }

    /// Drops what the query does not admit by the own lines.
    fn keep_by_own_line(&self, found: &mut Vec<&'company Header>) -> Result<()> {
        let query = self.query;
        let is_of_amount = query.is_about_amounts() && query.posts_to.is_none();
        if query.account.is_none() && query.paid.is_none() && !is_of_amount {
            return Ok(());
        }
        let mut kept = Vec::new();
        for &header in found.iter() {
            let Some(own) = self.own_side(header)? else {
                continue;
            };
            let paid = match self.open_state(header)? {
                OpenState::Paid => Some(PaidState::Paid),
                OpenState::Open(_) => Some(PaidState::Unpaid),
                OpenState::NotTracked => None,
            };
            let is_on_account = query.account.is_none() || own.account() == query.account;
            let amount = own.total().or_else(|| own.amount());
            let is_admitted = !is_of_amount || amount.is_some_and(|amount| self.admits(amount));
            if is_on_account && is_admitted && (query.paid.is_none() || paid == query.paid) {
                kept.push(header);
            }
        }
        *found = kept;
        Ok(())
    }

    /// Drops what the query does not admit by the text it asks for.
    fn keep_by_text(
        &self,
        found: &mut Vec<&'company Header>,
        memos: &mut Memos<'company>,
    ) -> Result<()> {
        let Some(text) = &self.query.text else {
            return Ok(());
        };
        let holds = |own: Option<&str>| own.is_some_and(|own| case::folded(own).contains(text));
        let mut kept = Vec::new();
        for &header in found.iter() {
            // A memo is read last, since that takes longest.
            if holds(header.number.as_deref())
                || holds(self.full_name(header))
                || holds(memos.get(header)?.as_deref())
            {
                kept.push(header);
            }
        }
        *found = kept;
        Ok(())
    }

    fn sort(&self, found: &mut Vec<&'company Header>) -> Result<()> {
        match self.query.order {
            TransactionOrder::Date => self.sort_by(found, |header| Ok(Some(header.date))),
            TransactionOrder::Entered => self.sort_by(found, |header| Ok(Some(header.id))),
            TransactionOrder::Number => {
                self.sort_by(found, |header| Ok(header.number.as_deref().map(Natural)))
            }
            TransactionOrder::Name => {
                self.sort_by(found, |header| Ok(self.full_name(header).map(case::folded)))
            }
            TransactionOrder::Total => self.sort_by(found, |header| {
                let own = self.own_side(header)?;
                let total = own.and_then(OwnSide::home_total);
                Ok(total.map(|total| total.amount()))
            }),
            TransactionOrder::DueDate => self.sort_by(found, |header| {
                let own = self.own_side(header)?;
                Ok(own.and_then(OwnSide::due_date))
            }),
        }
    }

    fn sort_by<K: Ord>(
        &self,
        found: &mut Vec<&'company Header>,
        key: impl Fn(&'company Header) -> Result<Option<K>>,
    ) -> Result<()> {
        let mut keyed: Vec<(Option<K>, &Header)> = Vec::with_capacity(found.len());
        for &header in found.iter() {
            keyed.push((key(header)?, header));
        }
        let direction = self.query.direction;
        keyed.sort_by(|(left_key, left), (right_key, right)| {
            let by_key = direction.compare(left_key.as_ref(), right_key.as_ref());
            let in_time = || (left.date, left.id).cmp(&(right.date, right.id));
            by_key.then_with(|| direction.apply(in_time()))
        });
        *found = keyed.into_iter().map(|(_, header)| header).collect();
        Ok(())
    }

    fn summary(
        &self,
        header: &'company Header,
        memos: &mut Memos<'company>,
    ) -> Result<TransactionSummary> {
        let own = self.own_side(header)?;
        let own_lines = (self.own_lines)(header.form)?;
        let left = own_lines.left(header.id);
        let line = own_lines.order_or_transfer(header.id);
        let transfer = line.filter(|_| header.kind == TransactionKind::Transfer);
        let lost_line = match own {
            Some(_) => LeftOut::default(),
            None => LeftOut::of(own_lines.damage()),
        };
        Ok(TransactionSummary {
            id: header.id,
            kind: header.kind,
            date: header.date,
            number: header.number(),
            name: header.name(),
            memo: memos.get(header)?,
            account: own.and_then(OwnSide::account),
            other_account: transfer.and_then(|line| line.other_account),
            currency: own.map_or(self.home, OwnSide::currency),
            amount: own.and_then(OwnSide::amount),
            total: own.and_then(OwnSide::total),
            home_total: own.and_then(OwnSide::home_total),
            due_date: own.and_then(OwnSide::due_date),
            open: self.open_state(header)?,
            order: line.and_then(|line| OrderProgress::of_whole(line, header.kind)),
            posting: header.posting,
            last_action: header.last_action,
            origin: header.origin(),
            left_out: self.accounts.left_out().with(lost_line),
            owed: left.map(Left::owed).unwrap_or_default(),
        })
    }

    /// What could not be read that the query may have found, or that a
    /// transaction which is `shown` may lack. What several parts lack counts once.
    fn left_out(&self, shown: &[&Header]) -> Result<LeftOut> {
        let names = self.names.into_iter().flat_map(Names::damage);
        let posted = self.ledger.map(Ledger::damage).unwrap_or_default();
        // What is left of a transaction depends on the types of its accounts.
        let lists = self.accounts.damage().iter().chain(names).chain(posted);
        let mut lost: HashSet<Place> = lists.map(Damage::place).collect();
        for form in Form::all().filter(|&form| self.can_be_in(form)) {
            lost.extend(self.headers.damage(form).iter().map(Damage::place));
            let is_shown = shown.iter().any(|header| header.form == form);
            if is_shown || self.query.is_about_own_lines() {
                let own_lines = (self.own_lines)(form)?;
                lost.extend(own_lines.damage().iter().map(Damage::place));
            }
        }
        let unread_tables = self.ledger.map(|ledger| ledger.left_out().unread_tables);
        let unread_tables = LeftOut::of_unread_tables(unread_tables.unwrap_or_default());
        Ok(LeftOut::of_places(lost).with(unread_tables))
    }

    /// Whether a transaction that the query asks for can be in `form`.
    fn can_be_in(&self, form: Form) -> bool {
        let kinds = &self.query.kinds;
        kinds.is_empty() || kinds.iter().any(|&kind| form.can_hold(kind))
    }

    fn own_side(&self, header: &'company Header) -> Result<Option<OwnSide<'company>>> {
        let own_lines = (self.own_lines)(header.form)?;
        let line = own_lines.get(header.id);
        Ok(line.map(|line| OwnSide::new(header, line, self.home)))
    }

    /// What is left of a transaction.
    fn open_state(&self, header: &'company Header) -> Result<OpenState> {
        let own = self.own_side(header)?;
        let currency = own.map_or(self.home, OwnSide::currency);
        let left = (self.own_lines)(header.form)?.left(header.id);
        Ok(left.map_or(OpenState::NotTracked, |left| left.state(header, currency)))
    }

    fn full_name(&self, header: &Header) -> Option<&'company str> {
        self.names?.full_name(header.name()?)
    }
}

/// How many of `found` are of each kind.
fn kinds(found: &[&Header]) -> Vec<KindCount> {
    let mut counts: BTreeMap<i64, usize> = BTreeMap::new();
    for header in found {
        *counts.entry(header.kind.code()).or_default() += 1;
    }
    let counts = counts.into_iter().map(|(code, count)| KindCount {
        kind: TransactionKind::from_code(code),
        count,
    });
    counts.collect()
}

//! Comparing forms with their lines, their links and the ledger.

use std::collections::HashMap;

use sa17::Decimal;

use crate::check::owed::Open;
use crate::check::tally::Figures;
use crate::check::{Cause, CheckKind, Subject, Tally};
use crate::damage::Problem;
use crate::ids::TransactionId;
use crate::ledger::Ledger;
use crate::money::{CENTS, Money};
use crate::open_items::OpenItem;
use crate::posting_state::PostingState;
use crate::side::Side;
use crate::transactions::{FormSide, OpenState, Transaction};

/// What comparing forms takes.
pub(crate) struct FormCheck<'company> {
    ledger: &'company Ledger,
    /// What the links leave open of each transaction, and how much could not be
    /// read of them. `None` when there are no links to go by.
    linked: Option<Linked<'company>>,
}

struct Linked<'company> {
    items: HashMap<TransactionId, Vec<(Side, &'company OpenItem)>>,
    causes: Vec<Cause>,
}

impl<'company> FormCheck<'company> {
    /// Compares with `ledger`, and with what the links leave on both sides.
    pub(crate) fn new(ledger: &'company Ledger, open: Option<&'company [Open; 2]>) -> Self {
        let linked = open.map(|sides| {
            let mut items: HashMap<TransactionId, Vec<(Side, &OpenItem)>> = HashMap::new();
            for open in sides {
                for item in &open.linked {
                    let of_transaction = items.entry(item.transaction).or_default();
                    of_transaction.push((open.linked.side, item));
                }
            }
            let causes = sides.iter().flat_map(Open::causes);
            Linked {
                items,
                causes: causes.collect(),
            }
        });
        Self { ledger, linked }
    }

    /// Compares one form.
    pub(crate) fn check(&self, tally: &mut Tally, form: &Transaction) {
        let subject = || Subject::Transaction(form.id);
        tally.expect(CheckKind::HeaderAndLines, subject(), self.agrees(form));
        let mut damage = form.damage.iter();
        let is_whole = !damage.any(|damage| damage.problem == Problem::LineChain);
        tally.expect(CheckKind::LineChain, subject(), is_whole);
        self.total(tally, form);
        self.left(tally, form);
    }

    /// Whether the postings of a form have its date and its kind, and are there
    /// when it posts.
    fn agrees(&self, form: &Transaction) -> bool {
        let mut entries = self.ledger.transaction_entries(form.id).peekable();
        if form.posting != PostingState::Posted {
            return entries.peek().is_none();
        }
        let has_posting_lines = form.lines.iter().any(|line| line.posted().is_some());
        if entries.peek().is_none() && has_posting_lines {
            return false;
        }
        entries.all(|entry| entry.date == form.date && entry.kind() == form.kind)
    }

    /// Compares the lines of a form with its total.
    fn total(&self, tally: &mut Tally, form: &Transaction) {
        let Some(total) = form.total else {
            return;
        };
        let lines = form.lines_total();
        let figures = Figures {
            computed: lines.map(Into::into),
            expected: Some(total.into()),
        };
        let subject = Subject::Transaction(form.id);
        tally.compare(CheckKind::FormTotal, subject, figures, || {
            let is_foreign = form.currency != self.ledger.home();
            if !form.damage.is_empty() {
                vec![Cause::Unread]
            } else if is_foreign && lines.is_some_and(|lines| is_near(lines, total)) {
                vec![Cause::Rounding]
            } else {
                Vec::new()
            }
        });
    }

    /// Compares what a form says is left of it with what its links leave.
    fn left(&self, tally: &mut Tally, form: &Transaction) {
        let Some(linked) = &self.linked else {
            return;
        };
        if form.open_state == OpenState::NotTracked {
            return;
        }
        let items = linked.items.get(&form.id).map(Vec::as_slice);
        let side = FormSide::of(form.kind);
        let shown = items.unwrap_or_default().iter().map(|(owed_on, item)| {
            let posted = owed_on.owed(item.open);
            if form.account() == Some(item.account) {
                side.total(posted).unwrap_or(posted)
            } else {
                side.shown(posted)
            }
        });
        let expected = add(shown, Money::zero(form.currency));
        let figures = Figures {
            computed: form.open().map(Into::into),
            expected: expected.map(Into::into),
        };
        let subject = Subject::Transaction(form.id);
        tally.compare(CheckKind::OpenForm, subject, figures, || {
            linked.causes.clone()
        });
    }
}

/// What amounts add up to from `first` on, unless they are of two currencies or
/// too large.
fn add(mut amounts: impl Iterator<Item = Money>, first: Money) -> Option<Money> {
    amounts.try_fold(first, Money::checked_add).ok()
}

/// Whether two amounts are a cent apart at most.
fn is_near(one: Money, other: Money) -> bool {
    let apart = one.checked_sub(other).map(Money::abs);
    let cent = Decimal::new(1, CENTS);
    cent.is_some_and(|cent| apart.is_ok_and(|apart| apart.amount() <= cent))
}

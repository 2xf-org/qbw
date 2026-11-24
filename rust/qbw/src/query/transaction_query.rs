//! Which transactions to list.

use std::ops::{Bound, RangeBounds};

use sa17::Decimal;

use crate::case;
use crate::codes::TransactionKind;
use crate::ids::{AccountId, NameId};
use crate::period::Period;
use crate::query::{Direction, PaidState, PostingFilter, TransactionOrder};

/// Which transactions to list, and in which order: all that QuickBooks lists,
/// the newest first, unless it says otherwise.
///
/// ```
/// use qbw::query::{Direction, PaidState, TransactionOrder, TransactionQuery};
/// use qbw::transactions::TransactionKind;
/// use qbw::{Company, Page};
///
/// # let company = Company::from_bytes(qbw_synthetic::company::books().build())?;
/// let unpaid = TransactionQuery::new()
///     .kinds([TransactionKind::Invoice])
///     .paid(PaidState::Unpaid)
///     .order(TransactionOrder::DueDate, Direction::Ascending);
/// let page = company.transactions(&unpaid, Page::new(0, 50))?;
/// assert_eq!(page.rows[0].number.as_deref(), Some("1041"));
/// # Ok::<(), qbw::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionQuery {
    /// Every kind when there are none.
    pub(super) kinds: Vec<TransactionKind>,
    pub(super) period: Period,
    pub(super) name: Option<NameId>,
    pub(super) with_jobs: bool,
    pub(super) account: Option<AccountId>,
    pub(super) posts_to: Option<AccountId>,
    /// The least and the greatest amount.
    pub(super) amount: (Bound<Decimal>, Bound<Decimal>),
    pub(super) number: Option<String>,
    pub(super) paid: Option<PaidState>,
    /// In lowercase.
    pub(super) text: Option<String>,
    pub(super) posting: PostingFilter,
    pub(super) order: TransactionOrder,
    pub(super) direction: Direction,
}

impl TransactionQuery {
    /// Every transaction that QuickBooks lists, the newest first.
    #[must_use]
    pub fn new() -> Self {
        Self {
            kinds: Vec::new(),
            period: Period::all(),
            name: None,
            with_jobs: false,
            account: None,
            posts_to: None,
            amount: (Bound::Unbounded, Bound::Unbounded),
            number: None,
            paid: None,
            text: None,
            posting: PostingFilter::Listed,
            order: TransactionOrder::Date,
            direction: Direction::Descending,
        }
    }

    /// Only transactions of `kinds`. None of them asks for every kind.
    #[must_use]
    pub fn kinds(self, kinds: impl IntoIterator<Item = TransactionKind>) -> Self {
        let kinds = kinds.into_iter().collect();
        Self { kinds, ..self }
    }

    /// Only transactions of the days of `period`.
    #[must_use]
    pub fn dated(self, period: impl Into<Period>) -> Self {
        let period = period.into();
        Self { period, ..self }
    }

    /// Only transactions that are with `name`. The names of lines do not count.
    #[must_use]
    pub fn name(self, name: NameId) -> Self {
        let name = Some(name);
        Self { name, ..self }
    }

    /// With those of the jobs of [`Self::name`], and of their jobs.
    #[must_use]
    pub fn with_jobs(self) -> Self {
        Self {
            with_jobs: true,
            ..self
        }
    }

    /// Only transactions whose own side is on `account`.
    #[must_use]
    pub fn account(self, account: AccountId) -> Self {
        let account = Some(account);
        Self { account, ..self }
    }

    /// Only transactions that post to `account`, with whichever line.
    #[must_use]
    pub fn posts_to(self, account: AccountId) -> Self {
        let posts_to = Some(account);
        Self { posts_to, ..self }
    }

    /// Only transactions of an amount in `range`: at least one with `least..`, at
    /// most one with `..=most`, or exactly one with `amount..=amount`.
    ///
    /// The amount is without its sign. With [`Self::posts_to`] it is what the
    /// transaction posts to the account, in the account's currency, and else the
    /// total that the form shows, or the amount of the own line of a form that
    /// shows none.
    #[must_use]
    pub fn amount(self, range: impl RangeBounds<Decimal>) -> Self {
        let amount = (range.start_bound().cloned(), range.end_bound().cloned());
        Self { amount, ..self }
    }

    /// Only transactions with `number`, ignoring case.
    #[must_use]
    pub fn number(self, number: &str) -> Self {
        let number = Some(number.to_owned());
        Self { number, ..self }
    }

    /// Only transactions that something or nothing is left of. The others can
    /// be neither paid nor used.
    #[must_use]
    pub fn paid(self, paid: PaidState) -> Self {
        let paid = Some(paid);
        Self { paid, ..self }
    }

    /// Only transactions with `text` in their number, their memo or the full
    /// name of who they are with, ignoring case.
    #[must_use]
    pub fn text(self, text: &str) -> Self {
        let text = (!text.is_empty()).then(|| case::folded(text));
        Self { text, ..self }
    }

    /// Which transactions to look at, instead of those that QuickBooks lists.
    #[must_use]
    pub fn posting(self, posting: PostingFilter) -> Self {
        Self { posting, ..self }
    }

    /// The order of the transactions, instead of the newest first.
    #[must_use]
    pub fn order(self, order: TransactionOrder, direction: Direction) -> Self {
        Self {
            order,
            direction,
            ..self
        }
    }

    /// Whether the answer depends on the names of those that transactions are
    /// with.
    pub(crate) fn is_about_names(&self) -> bool {
        self.with_jobs || self.text.is_some() || self.order == TransactionOrder::Name
    }

    /// Whether the answer depends on an amount.
    pub(super) fn is_about_amounts(&self) -> bool {
        self.amount != (Bound::Unbounded, Bound::Unbounded)
    }

    /// Whether the answer depends on what transactions post.
    pub(crate) fn is_about_postings(&self) -> bool {
        self.posts_to.is_some()
    }

    /// Whether the answer depends on the own lines of transactions.
    pub(super) fn is_about_own_lines(&self) -> bool {
        self.account.is_some()
            || (self.is_about_amounts() && self.posts_to.is_none())
            || self.paid.is_some()
            || matches!(
                self.order,
                TransactionOrder::Total | TransactionOrder::DueDate
            )
    }
}

impl Default for TransactionQuery {
    fn default() -> Self {
        Self::new()
    }
}

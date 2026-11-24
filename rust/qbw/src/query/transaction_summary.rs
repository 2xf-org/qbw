//! A transaction in a list.

use chrono::NaiveDate;

use crate::amount::Total;
use crate::codes::{LastAction, TransactionKind};
use crate::ids::{AccountId, CurrencyId, NameId, TransactionId};
use crate::left_out::LeftOut;
use crate::money::Money;
use crate::origin::Origin;
use crate::rest;
use crate::transactions::{OpenState, OrderProgress, PostingState, Transfer};

/// A transaction in a list, without its lines.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct TransactionSummary {
    /// What the transaction is known by.
    pub id: TransactionId,
    /// Its kind.
    pub kind: TransactionKind,
    /// Its date.
    pub date: NaiveDate,
    /// Its number, as the form shows it.
    pub number: Option<String>,
    /// Who it is with.
    pub name: Option<NameId>,
    /// The memo.
    pub memo: Option<String>,
    /// The account of its own side.
    pub account: Option<AccountId>,
    /// Of a transfer, the account across from its own, which is where the money
    /// goes: [`Self::transfer`] says both. `None` for any other kind.
    pub other_account: Option<AccountId>,
    /// Its currency.
    pub currency: CurrencyId,
    /// The amount of its own side in the transaction's currency. A debit is
    /// positive.
    pub amount: Option<Money>,
    /// The total that the form shows, in the transaction's currency. `None` on
    /// a kind that shows none, such as a journal entry.
    pub total: Option<Money>,
    /// The total in the company's currency.
    pub home_total: Option<Money>,
    /// When it is due.
    pub due_date: Option<NaiveDate>,
    /// What is left to pay or to use.
    pub open: OpenState,
    /// How far the whole order is filled, on an order.
    pub order: Option<OrderProgress>,
    /// Whether it is part of the books.
    pub posting: PostingState,
    /// What somebody last did to it.
    pub last_action: LastAction,
    /// The stored row of its header.
    pub origin: Origin,
    /// How much could not be read that it may lack: its own line, or the type of
    /// an account.
    pub left_out: LeftOut,
    /// What is left as something owed.
    pub(super) owed: Total,
}

impl TransactionSummary {
    /// Of a transfer, the account that the money leaves and the one that it goes
    /// to. `None` for any other kind.
    #[must_use]
    pub fn transfer(&self) -> Option<Transfer> {
        let is_transfer = self.kind == TransactionKind::Transfer;
        is_transfer.then(|| Transfer::between(self.account, self.other_account))
    }

    /// What is left to pay of it or to use, as the field of that name says it:
    /// nothing, in its own currency, when it is paid, and `None` when nothing is
    /// ever paid or used of it.
    #[must_use]
    pub fn open(&self) -> Option<Money> {
        self.open.amount(self.currency)
    }

    /// Whether it is paid or used up. `None` for a transaction that cannot be.
    #[must_use]
    pub const fn is_paid(&self) -> Option<bool> {
        self.open.is_paid()
    }

    /// Whether something is still owed of it, to the company or by it, and was
    /// due before the day `date`. A payment and a credit are never overdue.
    #[must_use]
    pub fn is_overdue(&self, date: NaiveDate) -> bool {
        self.posting == PostingState::Posted && rest::is_overdue(self.owed, self.due_date, date)
    }

    /// Whether somebody closed the order by hand.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.order.is_some_and(|order| order.is_closed)
    }

    /// Whether nothing more is to come of a purchase order: all of it was
    /// received, or somebody closed it.
    #[must_use]
    pub fn is_fully_received(&self) -> bool {
        self.kind == TransactionKind::PurchaseOrder && self.is_done()
    }

    /// Whether nothing more is to be invoiced of a sales order: all of it was,
    /// or somebody closed it.
    #[must_use]
    pub fn is_fully_invoiced(&self) -> bool {
        self.kind == TransactionKind::SalesOrder && self.is_done()
    }

    /// Whether somebody voided it. One that was changed after that says no.
    #[must_use]
    pub fn is_voided(&self) -> bool {
        self.last_action == LastAction::Voided
    }

    fn is_done(&self) -> bool {
        self.order.is_some_and(|order| order.is_done)
    }
}

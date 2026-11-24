//! A transaction.

use chrono::NaiveDate;
use sa17::Decimal;

use crate::amount::Total;
use crate::codes::{LastAction, NameColumn, TransactionKind};
use crate::damage::Damage;
use crate::ids::{
    AccountId, AddressId, ContactId, CurrencyId, ItemId, NameId, OwnerKey, PaymentMethodId,
    SalesRepId, TermsId, TransactionId, UserId,
};
use crate::left_out::LeftOut;
use crate::money::Money;
use crate::origin::Origin;
use crate::party::Party;
use crate::price::Price;
use crate::rest;
use crate::transactions::{
    Bill, Details, Line, LineRole, OpenState, OwnLine, Payment, PostingState, Sale,
};
use crate::unix_time::UnixTime;

/// A transaction: an invoice, a check, a journal entry or a form of another
/// kind, with its lines.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Transaction {
    /// What the transaction is known by.
    pub id: TransactionId,
    /// Its kind.
    pub kind: TransactionKind,
    /// Its date.
    pub date: NaiveDate,
    /// Its number, as the form shows it. Others may have the same.
    pub number: Option<String>,
    /// Who it is with.
    pub name: Option<NameId>,
    /// The memo.
    pub memo: Option<String>,
    /// The currency it is in.
    pub currency: CurrencyId,
    /// What one unit of that is worth in the company's currency, as the file
    /// keeps it. The amounts in the company's currency are kept, too, and this
    /// is too coarse to work them out.
    pub exchange_rate: f32,
    /// The total that the form shows, in the transaction's currency. `None` on
    /// a kind that shows none, such as a journal entry. Zero on a payment that
    /// moved no money.
    pub total: Option<Money>,
    /// The total in the company's currency.
    pub home_total: Option<Money>,
    /// When it is due.
    pub due_date: Option<NaiveDate>,
    /// What is left to pay of it or to use: [`Self::open`] as an amount.
    pub open_state: OpenState,
    /// Its own side, such as the bank account of a check. `None` when that
    /// cannot be read.
    pub own_line: Option<OwnLine>,
    /// Whether it is part of the books.
    pub posting: PostingState,
    /// What somebody last did to it.
    pub last_action: LastAction,
    /// The address and phone numbers of who it is with, as they were then.
    pub billing: Option<ContactId>,
    /// The name of the terms of payment, which the form shows.
    pub terms_name: Option<String>,
    /// The initials of who sold it, which the form shows.
    pub sales_rep_initials: Option<String>,
    /// What it has because of its kind.
    pub details: Details,
    /// Its lines in the order of the form, without its own side. Some are not
    /// on the form, and [`Self::form_lines`] are those that are.
    ///
    /// The form of a journal entry has its own side as the first line, and so has
    /// that of a payment that moved no money: a discount. There the first of these
    /// is its own side, as [`Line::is_own`] says.
    pub lines: Vec<Line>,
    /// When it was entered.
    pub created: Option<UnixTime>,
    /// When it was last changed.
    pub modified: Option<UnixTime>,
    /// Who last changed it.
    pub edited_by: Option<UserId>,
    /// The number that QuickBooks gave it in the order it was entered. A few
    /// repeat.
    pub sequence: Option<i32>,
    /// What its custom fields are kept by.
    pub owner_key: Option<OwnerKey>,
    /// What could not be read of its lines, and of the lists that its form takes
    /// something from which is not there.
    pub damage: Vec<Damage>,
    /// The stored row of its header.
    pub origin: Origin,
    pub(super) party: Option<Party>,
    /// What is left as something owed.
    pub(super) owed: Total,
}

impl Transaction {
    /// The lines that the form shows, in its order.
    pub fn form_lines(&self) -> impl Iterator<Item = &Line> {
        self.lines.iter().filter(|line| line.role.is_on_form())
    }

    /// The lines that are of an item.
    pub fn item_lines(&self) -> impl Iterator<Item = &Line> {
        self.in_role(LineRole::Item)
    }

    /// The lines that are on an account, as those of the expenses tab are.
    pub fn expense_lines(&self) -> impl Iterator<Item = &Line> {
        self.in_role(LineRole::Account)
    }

    /// The account of its own side.
    #[must_use]
    pub fn account(&self) -> Option<AccountId> {
        self.own_line.as_ref()?.account
    }

    /// The customer or the job it is with.
    #[must_use]
    pub fn customer(&self) -> Option<NameId> {
        self.party?.in_list(NameColumn::Customer)
    }

    /// The vendor it is with.
    #[must_use]
    pub fn vendor(&self) -> Option<NameId> {
        self.party?.in_list(NameColumn::Vendor)
    }

    /// What it has as a form that customers get.
    #[must_use]
    pub fn sale(&self) -> Option<&Sale> {
        match &self.details {
            Details::Sale(sale) => Some(sale),
            _ => None,
        }
    }

    /// What it has as a bill, a vendor credit or an item receipt.
    #[must_use]
    pub fn bill(&self) -> Option<&Bill> {
        match &self.details {
            Details::Bill(bill) => Some(bill),
            _ => None,
        }
    }

    /// What it has as a payment from a customer.
    #[must_use]
    pub fn payment(&self) -> Option<&Payment> {
        match &self.details {
            Details::Payment(payment) => Some(payment),
            _ => None,
        }
    }

    /// The terms of payment.
    #[must_use]
    pub fn terms(&self) -> Option<TermsId> {
        self.details.terms()
    }

    /// Who sold it.
    #[must_use]
    pub fn sales_rep(&self) -> Option<SalesRepId> {
        self.details.sales_rep()
    }

    /// Where the goods go.
    #[must_use]
    pub fn ship_to(&self) -> Option<AddressId> {
        self.details.ship_to()
    }

    /// How it was paid.
    #[must_use]
    pub fn payment_method(&self) -> Option<PaymentMethodId> {
        self.details.payment_method()
    }

    /// What the lines that add to the total come to, which should be
    /// [`Self::total`] on a form that shows one.
    #[must_use]
    pub fn lines_total(&self) -> Option<Money> {
        self.sum(self.lines.iter().filter(|line| line.role.adds_to_total()))
    }

    /// What the lines of a form that customers get add up to before sales tax.
    #[must_use]
    pub fn subtotal(&self) -> Option<Money> {
        self.sale()?;
        let taxed = self.lines.iter().filter(|line| line.role.adds_to_total());
        self.sum(taxed.filter(|line| line.role != LineRole::SalesTax))
    }

    /// The sales tax of a form that customers get.
    #[must_use]
    pub fn tax_total(&self) -> Option<Money> {
        self.sale()?;
        self.sum(self.in_role(LineRole::SalesTax))
    }

    /// The sales tax that was chosen for it, or the group of them.
    #[must_use]
    pub fn tax_item(&self) -> Option<ItemId> {
        let group = self.in_role(LineRole::SalesTaxGroup).next();
        let tax = group.or_else(|| self.in_role(LineRole::SalesTax).next());
        tax?.item
    }

    /// The percentage of its sales taxes together. `None` when it has none.
    #[must_use]
    pub fn tax_percent(&self) -> Option<Decimal> {
        let taxes = self.in_role(LineRole::SalesTax);
        let percentages = taxes.filter_map(|tax| match tax.rate {
            Some(Price::Percent(percent)) => Some(percent),
            _ => None,
        });
        percentages.fold(None, |sum, percent| {
            percent.checked_add(sum.unwrap_or_default())
        })
    }

    /// What is left to pay of it or to use, as [`Self::open_state`] says.
    #[must_use]
    pub fn open(&self) -> Option<Money> {
        self.open_state.amount(self.currency)
    }

    /// Whether it is paid or used up. `None` where [`Self::open`] is.
    #[must_use]
    pub fn is_paid(&self) -> Option<bool> {
        self.open_state.is_paid()
    }

    /// Whether something is still owed of it, to the company or by it, and was
    /// due before the day `date`. A payment and a credit are never overdue.
    #[must_use]
    pub fn is_overdue(&self, date: NaiveDate) -> bool {
        self.posting == PostingState::Posted && rest::is_overdue(self.owed, self.due_date, date)
    }

    /// Whether somebody voided it. One that was changed after that says no.
    #[must_use]
    pub fn is_voided(&self) -> bool {
        self.last_action == LastAction::Voided
    }

    /// Whether somebody closed the order by hand.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        let order = self.own_line.as_ref().and_then(|own| own.order);
        order.is_some_and(|order| order.is_closed)
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

    /// How much could not be read of what is in [`Self::damage`].
    #[must_use]
    pub fn left_out(&self) -> LeftOut {
        LeftOut::of(&self.damage)
    }

    fn in_role(&self, role: LineRole) -> impl Iterator<Item = &Line> {
        self.lines.iter().filter(move |line| line.role == role)
    }

    fn is_done(&self) -> bool {
        let order = self.own_line.as_ref().and_then(|own| own.order);
        order.is_some_and(|order| order.is_done)
    }

    fn sum<'line>(&self, lines: impl Iterator<Item = &'line Line>) -> Option<Money> {
        let mut amounts = lines.filter_map(|line| line.amount);
        amounts.try_fold(Money::zero(self.currency), |sum, amount| {
            sum.checked_add(amount).ok()
        })
    }
}

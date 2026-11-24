//! A line of a transaction.

use chrono::NaiveDate;
use sa17::Decimal;

use crate::codes::{Cleared, ItemKind};
use crate::currencies::Currencies;
use crate::ids::{
    AccountId, ClassId, CurrencyId, ItemId, LineId, NameId, OwnerKey, PaymentMethodId,
    SalesTaxCodeId, SiteId,
};
use crate::money::Money;
use crate::origin::Origin;
use crate::price::Price;
use crate::tables::lines::LineRow;
use crate::transactions::form_side::FormSide;
use crate::transactions::stored_line::StoredLine;
use crate::transactions::{Header, JournalForm, LineRole, OrderProgress, Posted, PostingState};

/// A line of a transaction.
///
/// Amounts and quantities are as the form shows them: positive on the lines of
/// an ordinary invoice and of an ordinary bill alike, and with a debit as
/// positive in a journal entry and where stock is counted or moved.
/// [`Self::posted`] is what the books get, with a debit as positive everywhere.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Line {
    /// What the line is known by.
    pub id: LineId,
    /// Whether it is the transaction's own line, which the form shows among the
    /// others: the first line of a journal entry.
    /// [`Transaction::own_line`](crate::transactions::Transaction::own_line) is
    /// the same line.
    pub is_own: bool,
    /// What it is for.
    pub role: LineRole,
    /// The line that the group starts with that it is a member of, or ends.
    pub group: Option<LineId>,
    /// The item.
    pub item: Option<ItemId>,
    /// The account that the line is on.
    pub account: Option<AccountId>,
    /// The line's own name, such as the job that a purchase is for, the agency
    /// that a tax is owed to or who a deposited payment is from. `None` where
    /// the line has the transaction's name.
    pub name: Option<NameId>,
    /// The class. Unchecked.
    pub class: Option<ClassId>,
    /// What the line says.
    pub description: Option<String>,
    /// How many.
    pub quantity: Option<Decimal>,
    /// The price of one, or the percentage, in the transaction's currency.
    pub rate: Option<Price>,
    /// The amount in the transaction's currency. `None` on a line without one,
    /// and on an exchange difference.
    pub amount: Option<Money>,
    /// The amount in the company's currency.
    pub home_amount: Option<Money>,
    /// Whether the line is taxed.
    pub tax_code: Option<SalesTaxCodeId>,
    /// How far the line is filled, on an order.
    pub order: Option<OrderProgress>,
    /// The site that the stock comes from or goes to.
    pub site: Option<SiteId>,
    /// The bin of the site.
    pub bin: Option<SiteId>,
    /// When the work was done.
    pub service_date: Option<NaiveDate>,
    /// How a deposited payment was made.
    pub payment_method: Option<PaymentMethodId>,
    /// The number of a deposited check.
    pub number: Option<String>,
    /// The manufacturer's number for the item.
    pub part_number: Option<String>,
    /// What was added to the cost.
    pub markup: Option<Price>,
    /// The first field that a form has for the company's own use.
    pub other1: Option<String>,
    /// The second of them.
    pub other2: Option<String>,
    /// Whether the line was found on the statement of its account.
    pub cleared: Cleared,
    /// The date of that statement, on some of the lines that were.
    pub reconciled_on: Option<NaiveDate>,
    /// What the file keeps of charging the line to a customer. What its
    /// numbers mean is not settled.
    pub reimbursed_code: Option<i32>,
    /// What its custom fields are kept by.
    pub owner_key: Option<OwnerKey>,
    /// The stored row.
    pub origin: Origin,
    posted: Option<Posted>,
    /// What the form of a journal entry has in its columns for debits and credits.
    /// A debit is positive.
    entered: Option<Money>,
}

impl Line {
    /// What the line puts into the books. `None` for a line without an account
    /// or an amount, for every line of a transaction that does not post, and for
    /// an own line, which is counted as
    /// [`Transaction::own_line`](crate::transactions::Transaction::own_line).
    #[must_use]
    pub fn posted(&self) -> Option<Posted> {
        self.posted
    }

    /// The amount of a line of a journal entry, when it is a debit, as the form
    /// has it: in the currency that
    /// [`JournalForm::is_shown_in_home_currency`](crate::transactions::JournalForm::is_shown_in_home_currency)
    /// tells. `None` on the lines of every other kind of transaction.
    #[must_use]
    pub fn debit(&self) -> Option<Money> {
        self.entered.filter(|amount| !amount.is_negative())
    }

    /// The amount of a line of a journal entry without its sign, when it is a
    /// credit, as the form has it. `None` where [`Self::debit`] is always.
    #[must_use]
    pub fn credit(&self) -> Option<Money> {
        let credit = self.entered.filter(|amount| amount.is_negative());
        credit.map(Money::abs)
    }

    /// The line `stored` of the transaction of `header`, with an item of kind
    /// `item` and in `group`, in the books of a company that keeps them in
    /// `home`.
    pub(super) fn new(
        stored: StoredLine,
        header: &Header,
        home: CurrencyId,
        item: Option<ItemKind>,
        group: Option<LineId>,
    ) -> Self {
        let StoredLine { row, origin } = stored;
        let role = LineRole::of(&row, item);
        let currencies = Currencies::of_line(row.currency, home);
        let side = FormSide::of(header.kind);
        let amount = currencies.own_amount(row.home_amount, row.foreign_amount);
        let home_amount = currencies.home_amount(row.home_amount);
        let order = OrderProgress::of_line(&row, header.kind, item);
        Self {
            id: row.id,
            is_own: row.is_own,
            role,
            group,
            item: row.item,
            account: row.account,
            name: row.names().of_line(row.name_column).map(|party| party.name),
            class: row.class,
            quantity: row.quantity.map(|quantity| side.shown(quantity).quantity()),
            rate: rate(&row, currencies.own()),
            amount: amount.map(|amount| side.shown(amount)),
            home_amount: home_amount.map(|amount| side.shown(amount)),
            tax_code: row.tax_code,
            order: order.filter(|_| role == LineRole::Item),
            site: row.site,
            bin: row.bin,
            service_date: row.service_date,
            payment_method: row.payment_method,
            markup: markup(&row, currencies.own()),
            cleared: row.cleared.unwrap_or_default(),
            reconciled_on: row.reconciled_on,
            reimbursed_code: row.reimbursed_code,
            owner_key: row.owner_key,
            // The own line posts by itself, also where it is shown as a line.
            posted: match header.posting {
                PostingState::Posted if !row.is_own => posted(&row, currencies),
                _ => None,
            },
            description: row.memo,
            number: row.number,
            part_number: row.part_number,
            other1: row.other1,
            other2: row.other2,
            origin,
            entered: None,
        }
    }

    /// The same as a line of a journal entry, which has the form `journal`.
    pub(super) fn of_journal(self, journal: &JournalForm) -> Self {
        let entered = if journal.is_shown_in_home_currency {
            self.home_amount
        } else {
            self.amount
        };
        Self { entered, ..self }
    }
}

fn rate(row: &LineRow, currency: CurrencyId) -> Option<Price> {
    let price = row.price.or(row.exact_price);
    let gross_price = row.gross_price.or(row.exact_gross_price);
    let each = |price| Price::Each(Money::new(price, currency));
    let percent = || row.price_percent.map(Price::Percent);
    price
        .map(each)
        .or_else(percent)
        .or_else(|| gross_price.map(each))
}

fn markup(row: &LineRow, currency: CurrencyId) -> Option<Price> {
    let each = row
        .markup
        .map(|markup| Price::Each(Money::new(markup, currency)));
    each.or_else(|| row.markup_percent.map(Price::Percent))
}

fn posted(row: &LineRow, currencies: Currencies) -> Option<Posted> {
    let amount = currencies.own_amount(row.home_amount, row.foreign_amount);
    Some(Posted {
        account: row.account?,
        amount: amount.unwrap_or_else(|| Money::zero(currencies.own())),
        home_amount: currencies.home_amount(row.home_amount)?,
    })
}

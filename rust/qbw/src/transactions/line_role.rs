//! What a line of a transaction is for.

use std::fmt;

use crate::codes::ItemKind;
use crate::tables::lines::LineRow;

/// What a line of a transaction is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LineRole {
    /// An item that is sold, bought, ordered, counted or built in, or a
    /// discount or a charge that the company keeps as an item.
    Item,
    /// The line that a group of items starts with. The group's items follow
    /// it, up to its [`Self::GroupEnd`].
    GroupStart,
    /// The line that a group of items ends with. It repeats their sum.
    GroupEnd,
    /// The sum of the lines above it.
    Subtotal,
    /// A sales tax.
    SalesTax,
    /// Several sales taxes that were chosen as one. It repeats their sum.
    SalesTaxGroup,
    /// An amount on an account: a line of the expenses tab, the other side of
    /// a payment, or a line of a deposit or a journal entry.
    Account,
    /// What a discount takes off the receivable or the payable.
    DiscountTaken,
    /// The account that a discount goes to: the other side of
    /// [`Self::DiscountTaken`].
    DiscountAccount,
    /// What was kept in cash out of a deposit.
    CashBack,
    /// Words, without an amount.
    Text,
    /// A row of the form that was left blank.
    Empty,
    /// What exchange rates gave or took since the invoice or the bill. The
    /// form does not show it.
    ExchangeDifference,
}

impl LineRole {
    /// What a line other than the own one is for, whose item is of kind `item`.
    /// A line can pass several of the tests, and the first one counts.
    pub(super) fn of(row: &LineRow, item: Option<ItemKind>) -> Self {
        if row.is_exchange_difference {
            return Self::ExchangeDifference;
        }
        if row.is_group_end {
            return Self::GroupEnd;
        }
        if row.is_tax {
            return match item {
                Some(ItemKind::SalesTaxGroup) => Self::SalesTaxGroup,
                _ => Self::SalesTax,
            };
        }
        if row.is_discount {
            return Self::DiscountTaken;
        }
        if row.is_discount_account {
            return Self::DiscountAccount;
        }
        if row.is_cash_back && row.account.is_some() {
            return Self::CashBack;
        }
        if row.item.is_some() {
            return match item {
                Some(ItemKind::Group) => Self::GroupStart,
                Some(ItemKind::Subtotal) => Self::Subtotal,
                _ => Self::Item,
            };
        }
        let says_something = row.memo.is_some();
        match row.account {
            Some(_) if says_something || row.home_amount.is_some() => Self::Account,
            _ if says_something => Self::Text,
            _ => Self::Empty,
        }
    }

    /// Whether the amount of such a line is part of the form's total. The
    /// others repeat amounts, or have none in the transaction's currency.
    #[must_use]
    pub const fn adds_to_total(self) -> bool {
        match self {
            Self::Item
            | Self::SalesTax
            | Self::Account
            | Self::DiscountTaken
            | Self::DiscountAccount
            | Self::CashBack => true,
            Self::GroupStart
            | Self::GroupEnd
            | Self::Subtotal
            | Self::SalesTaxGroup
            | Self::Text
            | Self::Empty
            | Self::ExchangeDifference => false,
        }
    }

    /// Whether the form shows such a line.
    #[must_use]
    pub const fn is_on_form(self) -> bool {
        !matches!(self, Self::Empty | Self::ExchangeDifference)
    }
}

impl fmt::Display for LineRole {
    /// Writes what the line is for, such as `Sales tax`.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.pad(match self {
            Self::Item => "Item",
            Self::GroupStart => "Group",
            Self::GroupEnd => "End of group",
            Self::Subtotal => "Subtotal",
            Self::SalesTax => "Sales tax",
            Self::SalesTaxGroup => "Sales tax group",
            Self::Account => "Account",
            Self::DiscountTaken => "Discount",
            Self::DiscountAccount => "Discount account",
            Self::CashBack => "Cash back",
            Self::Text => "Text",
            Self::Empty => "Empty",
            Self::ExchangeDifference => "Exchange difference",
        })
    }
}

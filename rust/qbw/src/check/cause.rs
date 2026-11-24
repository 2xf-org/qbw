//! Why figures differ.

use std::fmt;

use crate::left_out::LeftOut;
use crate::money::Money;

/// Why figures differ.
///
/// A cause with an amount explains as much of the difference: the amount is
/// worked out apart from the two figures. The others are what may be the cause.
/// They explain nothing, and somebody has to look.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Cause {
    /// Some transactions are kept as open against what their links say, each of
    /// which is a finding of its own. They come to so much.
    KeptAsOpen(Money),
    /// Somebody posted so much to the account without an item in stock, or a
    /// sale did with one.
    OtherPostings(Money),
    /// May be: rows of the file cannot be read, and what is worked out lacks
    /// them.
    Unread,
    /// May be: QuickBooks has not brought its own total up to date, which a
    /// rebuild of the file does. It is as likely that something is missing here.
    OutOfDate,
    /// May be: the quantities that QuickBooks keeps for the sites of the item do
    /// not add up to the one that it keeps for the item.
    SitesDisagree,
    /// May be: the amounts of a form in another currency were rounded to the
    /// cent.
    Rounding,
    /// May be: QuickBooks sent its figure, and the books of the year went on.
    LaterChanges,
    /// May be: a cost was changed after the assembly was built, and the build
    /// posted another one than the history of the item has.
    BuildCosts,
}

impl Cause {
    /// How much of the difference it explains: of what is expected less what is
    /// worked out. `None` for what only may be the cause.
    #[must_use]
    pub const fn amount(self) -> Option<Money> {
        match self {
            Self::KeptAsOpen(amount) | Self::OtherPostings(amount) => Some(amount),
            Self::Unread
            | Self::OutOfDate
            | Self::SitesDisagree
            | Self::Rounding
            | Self::LaterChanges
            | Self::BuildCosts => None,
        }
    }

    /// The same cause with another amount, when it has one.
    pub(super) fn map_amount(self, amount_of: impl Fn(Money) -> Money) -> Self {
        match self {
            Self::KeptAsOpen(amount) => Self::KeptAsOpen(amount_of(amount)),
            Self::OtherPostings(amount) => Self::OtherPostings(amount_of(amount)),
            cause => cause,
        }
    }

    /// Why something may be missing, with what could not be read.
    pub(super) fn of_missing(left_out: LeftOut) -> Vec<Self> {
        let is_whole = left_out.is_nothing();
        (!is_whole).then_some(Self::Unread).into_iter().collect()
    }

    /// Why QuickBooks may keep another figure than what it rests on comes to,
    /// with what could not be read of that.
    pub(super) fn of_kept(left_out: LeftOut) -> Vec<Self> {
        if left_out.is_nothing() {
            vec![Self::OutOfDate]
        } else {
            vec![Self::Unread]
        }
    }
}

impl fmt::Display for Cause {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let may_be = match self {
            Self::KeptAsOpen(amount) => {
                return write!(formatter, "{amount} is kept as open against the links");
            }
            Self::OtherPostings(amount) => {
                return write!(formatter, "{amount} was posted without the stock");
            }
            Self::Unread => "rows of the file cannot be read",
            Self::OutOfDate => "QuickBooks' total is out of date",
            Self::SitesDisagree => "the sites do not add up to the item's quantity",
            Self::Rounding => "the amounts were rounded to the cent",
            Self::LaterChanges => "the year went on after it was worked out",
            Self::BuildCosts => "a cost was changed after a build",
        };
        write!(formatter, "maybe {may_be}")
    }
}

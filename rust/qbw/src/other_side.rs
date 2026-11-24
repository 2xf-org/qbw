//! The account on the other side of a line.

use crate::ids::AccountId;

/// The account on the other side of a line, as the register of its account shows
/// it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum OtherSide {
    /// One account.
    Account(AccountId),
    /// More than one, which the register calls a split.
    Split,
    /// None: the transaction's own side has no account.
    None,
}

/// Stands for [`OtherSide::Split`] where an id is kept. No account has it.
const SPLIT: i32 = i32::MIN;

/// Stands for [`OtherSide::None`] where an id is kept, as it does in the file.
const NONE: i32 = 0;

impl OtherSide {
    /// The other side of a transaction's own line, whose first other line is on
    /// `account`. The file says itself whether there are several.
    pub(crate) fn of_own_line(is_split: bool, account: Option<AccountId>) -> Self {
        if is_split {
            Self::Split
        } else {
            account.into()
        }
    }

    pub(crate) fn packed(self) -> i32 {
        match self {
            Self::Account(account) => account.number(),
            Self::Split => SPLIT,
            Self::None => NONE,
        }
    }

    pub(crate) const fn unpacked(number: i32) -> Self {
        match number {
            SPLIT => Self::Split,
            NONE => Self::None,
            account => Self::Account(AccountId::new(account)),
        }
    }
}

impl From<Option<AccountId>> for OtherSide {
    fn from(account: Option<AccountId>) -> Self {
        account.map_or(Self::None, Self::Account)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_side_packs_into_a_number() {
        let account = OtherSide::Account(AccountId::new(7));
        for side in [account, OtherSide::Split, OtherSide::None] {
            assert_eq!(OtherSide::unpacked(side.packed()), side);
        }
    }
}

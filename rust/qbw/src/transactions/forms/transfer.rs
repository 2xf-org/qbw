//! What a transfer has.

use crate::ids::AccountId;
use crate::tables::lines::LineRow;

/// The two accounts of a transfer. A transfer is of the account that the money
/// leaves: in the files of three companies, its own line is the credit of all of
/// 9,724 transfers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Transfer {
    /// The account that the money leaves.
    pub from: Option<AccountId>,
    /// The account that it goes to.
    pub to: Option<AccountId>,
}

impl Transfer {
    pub(crate) fn new(own: Option<&LineRow>, other: Option<&LineRow>) -> Self {
        let account = |line: Option<&LineRow>| line.and_then(|line| line.account);
        Self::between(account(own), account(other))
    }

    /// A transfer between its own account and the other one. The own line is the
    /// credit, so the money leaves its account.
    pub(crate) const fn between(own: Option<AccountId>, other: Option<AccountId>) -> Self {
        Self {
            from: own,
            to: other,
        }
    }
}

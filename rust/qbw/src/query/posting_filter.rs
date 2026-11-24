//! Which transactions a query looks at.

use crate::transactions::PostingState;

/// Which transactions a query looks at.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PostingFilter {
    /// Those that QuickBooks lists: all but the memorized ones.
    #[default]
    Listed,
    /// Every one.
    All,
    /// Those in one state only.
    Only(PostingState),
}

impl PostingFilter {
    pub(super) fn admits(self, state: PostingState) -> bool {
        match self {
            Self::Listed => state != PostingState::Memorized,
            Self::All => true,
            Self::Only(only) => state == only,
        }
    }
}

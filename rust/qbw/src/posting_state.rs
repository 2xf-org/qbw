//! Whether a transaction counts.

use crate::codes::TransactionKind;

/// Whether a transaction is part of the books.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PostingState {
    /// It is: its lines are in the balances of their accounts.
    Posted,
    /// It never is. Estimates and orders only say what may happen.
    NonPosting,
    /// It is not, until somebody marks it as final.
    Pending,
    /// It is not. QuickBooks keeps it as a form to start others from.
    Memorized,
}

/// What the file keeps of whether a transaction counts. Its header has it, and
/// every line of it once more.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PostingFlags {
    pub(crate) kind: TransactionKind,
    pub(crate) is_memorized: bool,
    pub(crate) is_non_posting: bool,
}

impl PostingFlags {
    /// Whether the transaction counts. One of a kind that never does, does not
    /// whatever its flag says.
    pub(crate) const fn state(self) -> PostingState {
        if self.is_memorized {
            PostingState::Memorized
        } else if self.kind.never_posts() {
            PostingState::NonPosting
        } else if self.is_non_posting {
            PostingState::Pending
        } else {
            PostingState::Posted
        }
    }

    pub(crate) const fn is_posted(self) -> bool {
        matches!(self.state(), PostingState::Posted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FINAL: PostingFlags = PostingFlags {
        kind: TransactionKind::Invoice,
        is_memorized: false,
        is_non_posting: false,
    };

    #[test]
    fn a_kind_that_never_posts_does_not_whatever_its_flag_says() {
        let of = |kind| {
            let flags = PostingFlags { kind, ..FINAL };
            let pending = PostingFlags {
                is_non_posting: true,
                ..flags
            };
            [flags.state(), pending.state()]
        };
        let expected = [PostingState::Posted, PostingState::Pending];
        assert_eq!(of(TransactionKind::Invoice), expected);
        for kind in [
            TransactionKind::Estimate,
            TransactionKind::PurchaseOrder,
            TransactionKind::SalesOrder,
        ] {
            assert_eq!(of(kind), [PostingState::NonPosting; 2]);
        }
        let memorized = PostingFlags {
            is_memorized: true,
            ..FINAL
        };
        assert_eq!(memorized.state(), PostingState::Memorized);
    }
}

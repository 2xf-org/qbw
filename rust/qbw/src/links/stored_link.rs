//! A link as the index keeps it.

use crate::amount::Amount;
use crate::codes::LinkCode;
use crate::ids::{LineId, TransactionId};

/// A link of a transaction to another, packed, since a company has hundreds of
/// thousands of them. An id is the number that the file stores.
#[derive(Clone, Copy, Debug)]
pub(crate) struct StoredLink {
    pub(crate) transaction: TransactionId,
    pub(crate) other: TransactionId,
    line: i32,
    other_line: i32,
    value: Amount,
    code: i16,
    measure: Measure,
    pub(crate) is_source: bool,
}

/// What a link is worth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Worth {
    /// So much money, in the currency of both transactions. Above zero for what
    /// is paid or credited.
    Amount(Amount),
    /// So many of an item.
    Quantity(Amount),
    /// It only ties the two together.
    Nothing,
}

/// What the number of a link is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Measure {
    Amount,
    Quantity,
    Nothing,
}

/// The two ends of a link.
pub(super) struct Ends {
    pub(super) transaction: TransactionId,
    pub(super) line: Option<LineId>,
    pub(super) other: TransactionId,
    pub(super) other_line: Option<LineId>,
    pub(super) is_source: bool,
}

impl StoredLink {
    pub(super) fn new(ends: &Ends, code: LinkCode, worth: Worth) -> Self {
        let line = |line: Option<LineId>| line.map_or(0, LineId::number);
        let (measure, value) = match worth {
            Worth::Amount(amount) => (Measure::Amount, amount),
            Worth::Quantity(quantity) => (Measure::Quantity, quantity),
            Worth::Nothing => (Measure::Nothing, Amount::default()),
        };
        Self {
            transaction: ends.transaction,
            other: ends.other,
            line: line(ends.line),
            other_line: line(ends.other_line),
            value,
            // The file stores the code in a SMALLINT.
            code: i16::try_from(code.code()).unwrap_or(i16::MAX),
            measure,
            is_source: ends.is_source,
        }
    }

    /// What the links are ordered by: their ends, the transactions first.
    pub(super) fn place(&self) -> (TransactionId, TransactionId, i32, i32, i16) {
        let Self {
            transaction,
            other,
            line,
            other_line,
            code,
            ..
        } = *self;
        (transaction, other, line, other_line, code)
    }

    pub(crate) fn code(&self) -> LinkCode {
        LinkCode::from_code(self.code.into())
    }

    /// The line of the transaction.
    pub(crate) fn line(&self) -> Option<LineId> {
        (self.line != 0).then(|| LineId::new(self.line))
    }

    /// The line of the other transaction.
    pub(crate) fn other_line(&self) -> Option<LineId> {
        (self.other_line != 0).then(|| LineId::new(self.other_line))
    }

    pub(crate) fn worth(&self) -> Worth {
        match self.measure {
            Measure::Amount => Worth::Amount(self.value),
            Measure::Quantity => Worth::Quantity(self.value),
            Measure::Nothing => Worth::Nothing,
        }
    }

    /// The amount, or zero for a link that is not worth one.
    pub(crate) fn amount(&self) -> Amount {
        match self.worth() {
            Worth::Amount(amount) => amount,
            Worth::Quantity(_) | Worth::Nothing => Amount::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;

    #[test]
    fn a_link_is_small() {
        assert!(mem::size_of::<StoredLink>() <= 32);
    }

    #[test]
    fn a_link_keeps_what_it_is_worth() {
        let ends = Ends {
            transaction: TransactionId::new(1),
            line: Some(LineId::new(2)),
            other: TransactionId::new(3),
            other_line: None,
            is_source: true,
        };
        let five = Amount::new("5".parse().unwrap()).unwrap();
        for worth in [Worth::Amount(five), Worth::Quantity(five), Worth::Nothing] {
            let link = StoredLink::new(&ends, LinkCode::Other(12), worth);
            assert_eq!((link.worth(), link.code()), (worth, LinkCode::Other(12)));
            assert_eq!(
                (link.line(), link.other_line()),
                (Some(LineId::new(2)), None)
            );
        }
    }
}

//! What is left of a transaction.

use crate::amount::Total;
use crate::currencies::Currencies;
use crate::ids::CurrencyId;
use crate::posting_state::PostingState;
use crate::rest::Rest;
use crate::side::Side;
use crate::tables::lines::SummaryLineRow;
use crate::transactions::form_side::FormSide;
use crate::transactions::{Header, OpenState};

/// What is left of a transaction: of its lines on accounts for what customers
/// owe or what vendors are owed. That is the own line of an invoice and of a
/// bill, and another line of a payment.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Left {
    /// Whether something is left of one of them.
    is_open: bool,
    /// What is left of the own line, and of the others. A debit is positive.
    own: Total,
    others: Total,
    /// What is left as something owed.
    owed: Total,
}

impl Left {
    /// What is left of a line on an account of `side`.
    pub(super) fn of_line(line: &SummaryLineRow, side: Side, home: CurrencyId) -> Self {
        let currencies = Currencies::of_line(line.currency, home);
        let whole = currencies.own_of(line.home_amount, line.foreign_amount);
        let rest = Rest::of(line.is_paid, line.open_balance).of_line(whole.unwrap_or_default());
        let nothing = Total::default();
        let (own, others) = if line.is_own {
            (rest.into(), nothing)
        } else {
            (nothing, rest.into())
        };
        Self {
            is_open: !rest.is_zero(),
            own,
            others,
            owed: side.owed(rest.into()),
        }
    }

    /// What is left of this and of another line of the transaction.
    pub(super) fn with(self, other: Self) -> Self {
        Self {
            is_open: self.is_open || other.is_open,
            own: self.own + other.own,
            others: self.others + other.others,
            owed: self.owed + other.owed,
        }
    }

    /// What is left of the transaction of `header`, which is in `currency`.
    pub(crate) fn state(self, header: &Header, currency: CurrencyId) -> OpenState {
        if header.posting != PostingState::Posted {
            return OpenState::NotTracked;
        }
        if !self.is_open {
            return OpenState::Paid;
        }
        let side = FormSide::of(header.kind);
        // A journal entry shows its amounts as they are.
        let own = side.total(self.own).unwrap_or(self.own);
        let shown = own + side.shown(self.others);
        OpenState::Open(shown.money(currency))
    }

    /// What is left as something owed, to the company or by it.
    pub(crate) const fn owed(self) -> Total {
        self.owed
    }
}

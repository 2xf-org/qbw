//! Adding up postings.

use crate::amount::Total;
use crate::ids::CurrencyId;
use crate::ledger::{Balance, Entry};
use crate::left_out::LeftOut;

/// A balance that is being worked out. A debit is positive.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Sums {
    pub(crate) own_currency: CurrencyId,
    home_currency: CurrencyId,
    pub(crate) own: Total,
    pub(crate) home: Total,
}

impl Sums {
    /// The balance of nothing, in the company's currency and in the one of the
    /// account or the name.
    pub(crate) fn new(home_currency: CurrencyId, own_currency: CurrencyId) -> Self {
        Self {
            own_currency,
            home_currency,
            own: Total::default(),
            home: Total::default(),
        }
    }

    /// The same, with an amount that both currencies are alike for.
    pub(crate) fn with(self, amount: Total) -> Self {
        Self {
            own: self.own + amount,
            home: self.home + amount,
            ..self
        }
    }

    pub(crate) fn add_all(&mut self, entries: &[Entry]) {
        for entry in entries {
            self.add(entry);
        }
    }

    pub(crate) fn add(&mut self, entry: &Entry) {
        self.own += entry.own(self.own_currency, self.home_currency).into();
        self.home += entry.home.into();
    }

    pub(crate) fn balance(&self, left_out: LeftOut) -> Balance {
        Balance {
            amount: self.own.money(self.own_currency),
            home_amount: self.home.money(self.home_currency),
            left_out,
        }
    }
}

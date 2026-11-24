//! Comparing the years with the figures that QuickBooks worked out itself.

use std::collections::BTreeSet;

use sa17::Decimal;

use crate::check::tally::Figures;
use crate::check::{Cause, CheckKind, Subject, Tally};
use crate::money::Money;
use crate::period::Period;
use crate::reports;
use crate::reports::{Books, ProfitAndLoss};
use crate::settings::Facts;
use crate::tables::information::fact;

/// A figure that QuickBooks worked out when it last sent such figures.
struct YearlyFact {
    key: &'static str,
    figure: fn(&ProfitAndLoss) -> Money,
    /// Whether it is of the year that was going on.
    is_current: bool,
}

const YEARLY_FACTS: &[YearlyFact] = &[
    YearlyFact {
        key: fact::LAST_YEAR_NET_INCOME,
        figure: |year| year.net_income,
        is_current: false,
    },
    YearlyFact {
        key: fact::THIS_YEAR_NET_INCOME,
        figure: |year| year.net_income,
        is_current: true,
    },
    YearlyFact {
        key: fact::THIS_YEAR_INCOME,
        figure: |year| year.income.total,
        is_current: true,
    },
];

/// Whether the figures that QuickBooks worked out are those of fiscal years. The
/// file does not say of which, nor when it worked them out.
pub(crate) fn facts(tally: &mut Tally, facts: &Facts, books: Books<'_>) {
    let kept = YEARLY_FACTS.iter().filter_map(|yearly| {
        let fact = facts.get(yearly.key)?;
        let number: Decimal = fact.value.trim().parse().ok()?;
        Some((yearly, Money::new(number, books.ledger.home())))
    });
    let kept: Vec<_> = kept.collect();
    if kept.is_empty() {
        return;
    }
    let years = years(books);
    for (yearly, kept) in kept {
        let mut figures = years.iter().map(yearly.figure);
        let found = figures.find(|figure| *figure == kept);
        let subject = Subject::Fact(yearly.key.to_owned());
        let figures = Figures {
            computed: found.map(Into::into),
            expected: Some(kept.into()),
        };
        tally.compare(CheckKind::YearlyFigure, subject, figures, || {
            let cause = yearly.is_current.then_some(Cause::LaterChanges);
            cause.into_iter().collect()
        });
    }
}

/// The fiscal years that something is posted in.
fn years(books: Books<'_>) -> Vec<ProfitAndLoss> {
    let entries = books.ledger.entries_by_transaction();
    let starts = entries.filter_map(|entry| Period::year_of(entry.date, books.year_start).first());
    let starts: BTreeSet<_> = starts.collect();
    let years = starts.into_iter().map(|start| {
        let year = Period::year_of(start, books.year_start);
        reports::profit_and_loss(books, year)
    });
    years.collect()
}

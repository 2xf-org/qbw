//! The statements against each other and against QuickBooks' own figures.

use qbw::chrono::NaiveDate;
use qbw::reports::{BalanceSheet, ProfitAndLoss};
use qbw::sa17::Decimal;
use qbw::{Company, Money, Period};

use crate::agreement::Agreement;
use crate::{agreement, support};

/// A fiscal year with its statements.
struct Year {
    period: Period,
    last_day: NaiveDate,
    profit_and_loss: ProfitAndLoss,
    balance_sheet: BalanceSheet,
}

/// Every fiscal year from the first posting to the last.
fn years(company: &Company) -> Vec<Year> {
    let ledger = company.ledger().unwrap();
    let first = ledger.postings().map(|posting| posting.date).min().unwrap();
    let last = ledger.postings().map(|posting| posting.date).max().unwrap();
    let mut years = Vec::new();
    let mut day = first;
    while day <= last {
        let period = company.fiscal_year(day).unwrap();
        let last_day = period.last().unwrap();
        years.push(Year {
            period,
            last_day,
            profit_and_loss: company.profit_and_loss(period).unwrap(),
            balance_sheet: company.balance_sheet(last_day).unwrap(),
        });
        day = last_day.succ_opt().unwrap();
    }
    years
}

fn is_balanced(sheet: &BalanceSheet) -> bool {
    sheet.assets.total == sheet.liabilities_and_equity && sheet.unplaced.is_zero()
}

/// Whether amounts of one currency come to zero.
fn is_zero(amounts: impl Iterator<Item = Money>) -> bool {
    amounts.reduce(add).is_none_or(Money::is_zero)
}

fn add(left: Money, right: Money) -> Money {
    left.checked_add(right).unwrap()
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn the_balance_sheet_balances_at_every_year_end() {
    for company in support::samples() {
        let mut year_ends = Agreement::default();
        for year in years(&company) {
            year_ends.add(is_balanced(&year.balance_sheet));
        }
        year_ends.check("year ends that balance");
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn the_trial_balance_balances_at_every_year_end() {
    for company in support::samples() {
        let mut year_ends = Agreement::default();
        for year in years(&company) {
            let balance = company.trial_balance(year.last_day).unwrap();
            year_ends.add(balance.debits == balance.credits);
        }
        year_ends.check("trial balances that balance");
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn every_profit_goes_on_to_the_years_after() {
    for company in support::samples() {
        let mut agreement = Agreement::default();
        let mut earlier: Option<Money> = None;
        for year in years(&company) {
            let equity = year.balance_sheet.equity;
            let net_income = year.profit_and_loss.net_income;
            let earlier_profit = equity.earlier_profit.amount();
            let is_earlier = earlier.is_none_or(|earlier| earlier == earlier_profit);
            agreement.add(is_earlier && equity.net_income == net_income);
            earlier = Some(add(earlier_profit, net_income));
        }
        agreement.check("years whose profit goes on");
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn the_general_ledger_opens_and_closes_at_zero_every_year() {
    for company in support::samples() {
        let mut agreement = Agreement::default();
        for year in years(&company) {
            let ledger = company.general_ledger(year.period).unwrap();
            let opening = ledger.accounts.iter().map(|account| account.opening);
            let closing = ledger.accounts.iter().map(|account| account.closing);
            agreement.add(is_zero(opening) && is_zero(closing));
        }
        agreement.check("general ledgers that open and close at zero");
    }
}

/// A figure of a year.
type Figure = fn(&ProfitAndLoss) -> Money;

/// The figures that QuickBooks worked out itself when it last sent them, each
/// with what it is of a year.
const FACTS: &[(&str, Figure)] = &[
    ("QBF_NetIncomePnLAccrualInLastFiscalYear", |year| {
        year.net_income
    }),
    ("QBF_NetProfitPnLAccrualInCurrentFiscalYear", |year| {
        year.net_income
    }),
    ("QBF_RevenuePnLAccrualInCurrentFiscalYear", |year| {
        year.income.total
    }),
];

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn last_year_s_net_income_is_the_one_that_quickbooks_worked_out() {
    for company in support::samples() {
        let years = years(&company);
        let facts = agreement::rows(&company, "abmc_cf_facts", &["fact_key", "fact_value"]);
        for (key, figure) in FACTS {
            let fact = facts
                .iter()
                .find(|row| row.read::<&str>(0).unwrap() == *key);
            let fact = fact.unwrap().read::<Option<&str>>(1).unwrap();
            let fact: Decimal = fact.unwrap().trim().parse().unwrap();
            let mut figures = years.iter().map(|year| figure(&year.profit_and_loss));
            let found = figures.any(|figure| figure.amount() == fact);
            println!("{key} is the figure of a year: {found}");
            // What is true of the current year changes after it was sent.
            assert!(found || key.contains("Current"));
        }
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn the_statements_balance_on_days_within_the_year() {
    for company in support::samples() {
        let mut agreement = Agreement::default();
        let years = years(&company);
        let (first, last) = (years.first().unwrap(), years.last().unwrap());
        let days = first.period.first().unwrap().iter_days();
        // Not a multiple of a week, a month or a year.
        for day in days.step_by(97).take_while(|day| *day <= last.last_day) {
            let balance = company.trial_balance(day).unwrap();
            let sheet = company.balance_sheet(day).unwrap();
            agreement.add(is_balanced(&sheet) && balance.debits == balance.credits);
        }
        agreement.check("days that balance");
    }
}

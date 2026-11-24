//! Profit and loss.

use crate::codes::AccountType;
use crate::left_out::LeftOut;
use crate::money::Money;
use crate::period::Period;
use crate::reports::section_tree::{AccountAmount, Summed};
use crate::reports::{Books, Section, section_tree};

/// What the company earned and spent in a period.
///
/// The accounts of a section are those that something is posted to in the
/// period, even where it comes to zero. What is posted to accounts that are on
/// no statement is in none: it is [`Self::unplaced`], and zero in a file that is
/// whole.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ProfitAndLoss {
    /// The period.
    pub period: Period,
    /// What the company earned in its trade.
    pub income: Section,
    /// What the goods it sold had cost.
    pub cost_of_goods_sold: Section,
    /// Income less the cost of goods sold.
    pub gross_profit: Money,
    /// What it spent in its trade.
    pub expense: Section,
    /// Gross profit less expense.
    pub net_ordinary_income: Money,
    /// What it earned outside its trade.
    pub other_income: Section,
    /// What it spent outside its trade.
    pub other_expense: Section,
    /// Other income less other expense.
    pub net_other_income: Money,
    /// The profit: net ordinary income and net other income. A loss is below
    /// zero.
    pub net_income: Money,
    /// What is posted in the period to accounts that cannot be read, that are of
    /// a type this version does not know, or that are not part of the books. A
    /// debit is positive. Some of it may belong to the profit.
    pub unplaced: Money,
    /// How much could not be read.
    pub left_out: LeftOut,
}

impl ProfitAndLoss {
    /// The five sections, in the order of the statement.
    #[must_use]
    pub const fn sections(&self) -> [&Section; 5] {
        [
            &self.income,
            &self.cost_of_goods_sold,
            &self.expense,
            &self.other_income,
            &self.other_expense,
        ]
    }
}

pub(crate) fn profit_and_loss(books: Books<'_>, period: Period) -> ProfitAndLoss {
    let section = |account_type| -> Summed {
        section_tree::of(books, account_type, |account| {
            let entries = books.ledger.entries_during(account.id, period);
            AccountAmount {
                total: entries.iter().map(|entry| entry.home).sum(),
                is_shown: !entries.is_empty(),
            }
        })
    };
    let income = section(AccountType::Income);
    let cost = section(AccountType::CostOfGoodsSold);
    let expense = section(AccountType::Expense);
    let other_income = section(AccountType::OtherIncome);
    let other_expense = section(AccountType::OtherExpense);

    let gross = income.total - cost.total;
    let ordinary = gross - expense.total;
    let other = other_income.total - other_expense.total;
    ProfitAndLoss {
        period,
        income: income.section,
        cost_of_goods_sold: cost.section,
        gross_profit: books.money(gross),
        expense: expense.section,
        net_ordinary_income: books.money(ordinary),
        other_income: other_income.section,
        other_expense: other_expense.section,
        net_other_income: books.money(other),
        net_income: books.money(ordinary + other),
        unplaced: books.money(books.unplaced(period)),
        left_out: books.left_out(),
    }
}

//! The balance sheet.

use chrono::NaiveDate;

use crate::accounts::Account;
use crate::amount::Total;
use crate::codes::AccountType;
use crate::ids::AccountId;
use crate::left_out::LeftOut;
use crate::money::Money;
use crate::period::Period;
use crate::reports::section_tree::{AccountAmount, Summed};
use crate::reports::{Books, Section, section_tree};

/// What the company has and what it owes at the end of a day.
///
/// The accounts of a section are those whose balance is not zero. What is
/// posted to accounts that are on no statement is in none: it is
/// [`Self::unplaced`], and zero in a file that is whole.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BalanceSheet {
    /// The day. Of everything in the file, it is the last that something is
    /// posted on.
    pub as_of: NaiveDate,
    /// What the company has.
    pub assets: Assets,
    /// What it owes.
    pub liabilities: Liabilities,
    /// What the owners have in it.
    pub equity: Equity,
    /// Liabilities and equity together. They come to the same as the assets,
    /// with what is unplaced.
    pub liabilities_and_equity: Money,
    /// What is posted to accounts that cannot be read, that are of a type this
    /// version does not know, or that are not part of the books. A debit is
    /// positive.
    pub unplaced: Money,
    /// How much could not be read. Where that is not nothing, the two sides
    /// can differ.
    pub left_out: LeftOut,
}

/// What the company has.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Assets {
    /// Money in banks.
    pub bank: Section,
    /// What customers owe.
    pub accounts_receivable: Section,
    /// Other assets that turn into money within a year, such as stock.
    pub other_current_assets: Section,
    /// The three sections above together.
    pub current_assets: Money,
    /// What is owned for years, such as vehicles.
    pub fixed_assets: Section,
    /// Any other assets.
    pub other_assets: Section,
    /// All assets.
    pub total: Money,
}

/// What the company owes.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Liabilities {
    /// What vendors are owed.
    pub accounts_payable: Section,
    /// Credit cards.
    pub credit_cards: Section,
    /// Other debts due within a year, such as sales tax.
    pub other_current_liabilities: Section,
    /// The three sections above together.
    pub current_liabilities: Money,
    /// Debts due later.
    pub long_term_liabilities: Section,
    /// All liabilities.
    pub total: Money,
}

/// What the owners have in the company.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Equity {
    /// The accounts of equity. Retained Earnings has the profit of earlier
    /// years, besides what is posted to it.
    pub accounts: Section,
    /// The profit of every fiscal year before the one of the day.
    pub earlier_profit: EarlierProfit,
    /// The profit of the fiscal year up to the day.
    pub net_income: Money,
    /// All equity.
    pub total: Money,
}

/// The profit of earlier fiscal years, which QuickBooks posts nowhere, and where
/// it stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EarlierProfit {
    /// It is in the figure of an account of equity already, besides what is
    /// posted to it.
    InAccount {
        /// Retained Earnings, whatever the account is called.
        account: AccountId,
        /// The profit.
        amount: Money,
    },
    /// It stands beside the accounts, and counts into the total of equity: the
    /// file marks no account for it.
    Alone(Money),
}

impl EarlierProfit {
    /// The profit, wherever it stands.
    #[must_use]
    pub const fn amount(self) -> Money {
        match self {
            Self::InAccount { amount, .. } | Self::Alone(amount) => amount,
        }
    }
}

impl BalanceSheet {
    /// The ten sections, in the order of the statement: five of assets, four of
    /// liabilities and one of equity.
    #[must_use]
    pub const fn sections(&self) -> [&Section; 10] {
        let (assets, liabilities) = (&self.assets, &self.liabilities);
        [
            &assets.bank,
            &assets.accounts_receivable,
            &assets.other_current_assets,
            &assets.fixed_assets,
            &assets.other_assets,
            &liabilities.accounts_payable,
            &liabilities.credit_cards,
            &liabilities.other_current_liabilities,
            &liabilities.long_term_liabilities,
            &self.equity.accounts,
        ]
    }
}

pub(crate) fn balance_sheet(books: Books<'_>, as_of: NaiveDate) -> BalanceSheet {
    let sheet = Sheet {
        books,
        as_of,
        earlier_profit: books.earlier_profit(as_of),
    };
    let (liabilities, liabilities_total) = sheet.liabilities();
    let (equity, equity_total) = sheet.equity();
    BalanceSheet {
        as_of,
        assets: sheet.assets(),
        liabilities,
        equity,
        liabilities_and_equity: books.money(liabilities_total + equity_total),
        unplaced: books.money(books.unplaced(Period::until(as_of))),
        left_out: books.left_out(),
    }
}

/// A balance sheet that is being worked out.
struct Sheet<'company> {
    books: Books<'company>,
    as_of: NaiveDate,
    earlier_profit: Total,
}

impl Sheet<'_> {
    fn assets(&self) -> Assets {
        let bank = self.section(AccountType::Bank);
        let receivable = self.section(AccountType::AccountsReceivable);
        let other_current = self.section(AccountType::OtherCurrentAsset);
        let fixed = self.section(AccountType::FixedAsset);
        let other = self.section(AccountType::OtherAsset);
        let current = bank.total + receivable.total + other_current.total;
        Assets {
            bank: bank.section,
            accounts_receivable: receivable.section,
            other_current_assets: other_current.section,
            current_assets: self.books.money(current),
            fixed_assets: fixed.section,
            other_assets: other.section,
            total: self.books.money(current + fixed.total + other.total),
        }
    }

    fn liabilities(&self) -> (Liabilities, Total) {
        let payable = self.section(AccountType::AccountsPayable);
        let cards = self.section(AccountType::CreditCard);
        let other_current = self.section(AccountType::OtherCurrentLiability);
        let long_term = self.section(AccountType::LongTermLiability);
        let current = payable.total + cards.total + other_current.total;
        let total = current + long_term.total;
        let liabilities = Liabilities {
            accounts_payable: payable.section,
            credit_cards: cards.section,
            other_current_liabilities: other_current.section,
            current_liabilities: self.books.money(current),
            long_term_liabilities: long_term.section,
            total: self.books.money(total),
        };
        (liabilities, total)
    }

    fn equity(&self) -> (Equity, Total) {
        let accounts = self.section(AccountType::Equity);
        let net_income = self.books.profit(self.books.year_to(self.as_of));
        let amount = self.books.money(self.earlier_profit);
        let (earlier_profit, beside) = match self.books.retained_earnings() {
            Some(account) => (
                EarlierProfit::InAccount { account, amount },
                Total::default(),
            ),
            None => (EarlierProfit::Alone(amount), self.earlier_profit),
        };
        let total = accounts.total + beside + net_income;
        let equity = Equity {
            accounts: accounts.section,
            earlier_profit,
            net_income: self.books.money(net_income),
            total: self.books.money(total),
        };
        (equity, total)
    }

    fn section(&self, account_type: AccountType) -> Summed {
        section_tree::of(self.books, account_type, |account| {
            let total = self.balance(account);
            AccountAmount {
                total,
                is_shown: !total.is_zero(),
            }
        })
    }

    /// The balance of an account, a debit where it is positive.
    fn balance(&self, account: &Account) -> Total {
        let balance = self
            .books
            .ledger
            .total(account.id, Period::until(self.as_of));
        if self.books.retained_earnings() == Some(account.id) {
            balance - self.earlier_profit
        } else {
            balance
        }
    }
}

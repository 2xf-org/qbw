//! The statements, the general ledger, the journal and the account listing.

use qbw::accounts::{AccountType, SpecialAccount};
use qbw::chrono::NaiveDate;
use qbw::reports::{EarlierProfit, Section, TrialBalance};
use qbw::transactions::TransactionKind;
use qbw::{AccountId, AsOf, Company, Money, Page, Period};

use crate::changed::{Changed, whole};
use crate::support;

fn day(text: &str) -> NaiveDate {
    text.parse().unwrap()
}

fn typed(amounts: &[(i32, &str)]) -> Vec<(i32, String)> {
    let amounts = amounts.iter();
    amounts
        .map(|&(account, amount)| (account, amount.to_owned()))
        .collect()
}

fn rows(balance: &TrialBalance) -> Vec<(i32, String)> {
    let rows = balance.rows.iter();
    rows.map(|row| {
        (
            row.account.map_or(0, AccountId::number),
            row.balance.to_string(),
        )
    })
    .collect()
}

/// The accounts of a section as lines: each with its depth, its own amount and
/// its total.
fn lines(section: &Section) -> Vec<(usize, i32, String, String)> {
    let lines = section.walk().map(|(depth, account)| {
        let (own, total) = (account.amount.to_string(), account.total.to_string());
        (depth, account.account.number(), own, total)
    });
    lines.collect()
}

fn line(depth: usize, account: i32, own: &str, total: &str) -> (usize, i32, String, String) {
    (depth, account, own.to_owned(), total.to_owned())
}

fn texts<const N: usize>(amounts: [Money; N]) -> [String; N] {
    amounts.map(|amount| amount.to_string())
}

#[test]
fn a_trial_balance_has_this_year_s_profit_and_loss() {
    let company = support::books();
    let balance = company.trial_balance(day("2024-06-30")).unwrap();
    assert_eq!(
        rows(&balance),
        typed(&[
            (1, "837.00"),
            (2, "180.00"),
            (3, "45.00"),
            (6, "200.00"),
            (7, "-20.00"),
            (8, "-25.00"),
            (10, "-1000.00"),
            // Nothing is posted to Retained Earnings: the profit of 2023.
            (11, "-130.00"),
            (12, "-110.00"),
            (13, "50.00"),
            (14, "-20.00"),
            (15, "-20.00"),
            (16, "25.00"),
            (18, "3.00"),
            (17, "-15.00"),
        ])
    );
    assert_eq!(
        texts([balance.debits, balance.credits]),
        ["1340.00", "1340.00"]
    );
    let checking = balance.rows[0];
    assert_eq!(
        (checking.debit(), checking.credit()),
        (Some(checking.balance), None)
    );
    let payable = balance.rows[4];
    assert_eq!(
        payable.credit().map(|credit| credit.to_string()),
        Some("20.00".to_owned())
    );
    assert_eq!(payable.debit(), None);
}

#[test]
fn a_trial_balance_of_the_first_year_has_no_retained_earnings() {
    let company = support::books();
    let balance = company.trial_balance(day("2023-12-31")).unwrap();
    assert_eq!(
        rows(&balance),
        typed(&[
            (1, "800.00"),
            (3, "202.00"),
            (6, "140.00"),
            (9, "-12.00"),
            (10, "-1000.00"),
            (12, "-40.00"),
            (13, "-150.00"),
            (14, "60.00"),
        ])
    );
    assert_eq!(
        texts([balance.debits, balance.credits]),
        ["1202.00", "1202.00"]
    );
}

#[test]
fn profit_and_loss_nests_accounts_and_turns_credits() {
    let company = support::books();
    let year = company.fiscal_year(day("2024-06-30")).unwrap();
    assert_eq!(year, Period::between(day("2024-01-01"), day("2024-12-31")));
    let statement = company.profit_and_loss(year).unwrap();
    let income = [
        line(0, 12, "110.00", "60.00"),
        line(1, 13, "-50.00", "-50.00"),
    ];
    assert_eq!(lines(&statement.income), income);
    assert_eq!(
        lines(&statement.cost_of_goods_sold),
        [line(0, 14, "-20.00", "-20.00")]
    );
    let expense = [
        line(0, 15, "-20.00", "5.00"),
        line(1, 16, "25.00", "25.00"),
        line(0, 18, "3.00", "3.00"),
    ];
    assert_eq!(lines(&statement.expense), expense);
    assert_eq!(lines(&statement.other_income), []);
    assert_eq!(
        lines(&statement.other_expense),
        [line(0, 17, "-15.00", "-15.00")]
    );
    let totals = texts([
        statement.income.total,
        statement.gross_profit,
        statement.expense.total,
        statement.net_ordinary_income,
        statement.net_other_income,
        statement.net_income,
    ]);
    assert_eq!(
        totals,
        ["60.00", "80.00", "8.00", "72.00", "15.00", "87.00"]
    );
}

#[test]
fn profit_and_loss_has_only_the_accounts_of_its_period() {
    let company = support::books();
    let period = day("2023-01-01")..=day("2023-12-31");
    let statement = company.profit_and_loss(period).unwrap();
    let income = [
        line(0, 12, "40.00", "190.00"),
        line(1, 13, "150.00", "150.00"),
    ];
    assert_eq!(lines(&statement.income), income);
    assert_eq!(lines(&statement.expense), []);
    assert_eq!(
        texts([statement.gross_profit, statement.net_income]),
        ["130.00", "130.00"]
    );
    // An account that only a voided check is posted to has a line.
    let voided = company
        .profit_and_loss(day("2024-04-20")..=day("2024-04-20"))
        .unwrap();
    assert_eq!(lines(&voided.expense), [line(0, 15, "0.00", "0.00")]);
}

#[test]
fn a_balance_sheet_balances() {
    let company = support::books();
    let sheet = company.balance_sheet(day("2024-06-30")).unwrap();
    let bank = [
        line(0, 1, "837.00", "837.00"),
        line(0, 2, "180.00", "180.00"),
    ];
    assert_eq!(lines(&sheet.assets.bank), bank);
    assert_eq!(
        lines(&sheet.assets.accounts_receivable),
        [line(0, 3, "45.00", "45.00")]
    );
    assert_eq!(
        lines(&sheet.assets.other_current_assets),
        [line(0, 6, "200.00", "200.00")]
    );
    assert_eq!(
        texts([sheet.assets.current_assets, sheet.assets.total]),
        ["1262.00", "1262.00"]
    );

    let liabilities = &sheet.liabilities;
    assert_eq!(
        lines(&liabilities.accounts_payable),
        [line(0, 7, "20.00", "20.00")]
    );
    assert_eq!(
        lines(&liabilities.credit_cards),
        [line(0, 8, "25.00", "25.00")]
    );
    assert_eq!(lines(&liabilities.other_current_liabilities), []);
    assert_eq!(
        texts([liabilities.current_liabilities, liabilities.total]),
        ["45.00", "45.00"]
    );

    let equity = &sheet.equity;
    let accounts = [
        line(0, 10, "1000.00", "1000.00"),
        line(0, 11, "130.00", "130.00"),
    ];
    assert_eq!(lines(&equity.accounts), accounts);
    // It is in the figure of Retained Earnings already.
    let EarlierProfit::InAccount { account, amount } = equity.earlier_profit else {
        panic!("the file marks an account: {:?}", equity.earlier_profit);
    };
    assert_eq!(account, AccountId::new(11));
    let figures = texts([amount, equity.net_income, equity.total]);
    assert_eq!(figures, ["130.00", "87.00", "1217.00"]);
    assert_eq!(
        texts([sheet.liabilities_and_equity, sheet.unplaced]),
        ["1262.00", "0.00"]
    );
}

#[test]
fn net_income_moves_to_retained_earnings_with_the_year() {
    let company = support::books();
    let last = company.balance_sheet(day("2023-12-31")).unwrap().equity;
    assert_eq!(
        texts([last.earlier_profit.amount(), last.net_income, last.total]),
        ["0.00", "130.00", "1130.00"]
    );
    let first = company.balance_sheet(day("2024-01-01")).unwrap().equity;
    assert_eq!(
        texts([first.earlier_profit.amount(), first.net_income, first.total]),
        ["130.00", "0.00", "1130.00"]
    );
}

#[test]
fn a_general_ledger_opens_and_closes_at_zero() {
    let company = support::books();
    let year = company.fiscal_year(day("2024-06-30")).unwrap();
    let ledger = company.general_ledger(year).unwrap();
    let opening = |account: i32| {
        let mut accounts = ledger.accounts.iter();
        let found = accounts
            .find(|found| found.account == AccountId::new(account))
            .unwrap();
        (
            found.opening.to_string(),
            found.closing.to_string(),
            found.row_count(),
        )
    };
    assert_eq!(opening(1), ("800.00".to_owned(), "837.00".to_owned(), 4));
    assert_eq!(opening(11), ("-130.00".to_owned(), "-130.00".to_owned(), 0));
    assert_eq!(opening(12), ("0.00".to_owned(), "-110.00".to_owned(), 1));
    let zero = |amounts: Vec<Money>| {
        let sum = amounts
            .into_iter()
            .reduce(|sum, amount| sum.checked_add(amount).unwrap());
        sum.unwrap().is_zero()
    };
    assert!(zero(
        ledger
            .accounts
            .iter()
            .map(|account| account.opening)
            .collect()
    ));
    assert!(zero(
        ledger
            .accounts
            .iter()
            .map(|account| account.closing)
            .collect()
    ));

    let checking = &ledger.accounts[0];
    let balances: Vec<String> = checking.rows().map(|row| row.balance.to_string()).collect();
    assert_eq!(balances, ["900.00", "845.00", "845.00", "837.00"]);
    assert_eq!(
        checking.rows().next().map(|row| row.posting.line.number()),
        Some(601)
    );
}

#[test]
fn a_general_ledger_opens_income_with_the_year_so_far() {
    let company = support::books();
    let ledger = company
        .general_ledger(Period::starting(day("2024-03-01")))
        .unwrap();
    let mut accounts = ledger.accounts.iter();
    let sales = accounts
        .find(|found| found.account == AccountId::new(12))
        .unwrap();
    assert_eq!(
        texts([sales.opening, sales.closing]),
        ["-110.00", "-110.00"]
    );
    // Every account of the books, with nothing before them.
    let all = company.general_ledger(Period::all()).unwrap();
    assert!(all.accounts.iter().all(|account| account.opening.is_zero()));
    let rows: usize = all.accounts.iter().map(|account| account.row_count()).sum();
    assert_eq!(rows, 51);
}

#[test]
fn a_journal_has_what_each_transaction_posts() {
    let company = support::books();
    let journal = company
        .journal(day("2023-01-01")..=day("2023-12-31"))
        .unwrap();
    let entries: Vec<_> = journal.entries().collect();
    let kinds: Vec<TransactionKind> = entries.iter().map(|entry| entry.kind).collect();
    let expected = [
        TransactionKind::JournalEntry,
        TransactionKind::Bill,
        TransactionKind::BillPaymentCheck,
        TransactionKind::Invoice,
    ];
    assert_eq!(kinds, expected);
    let invoice = &entries[3];
    assert_eq!(
        (invoice.date, invoice.transaction.number()),
        (day("2023-11-15"), 400)
    );
    let lines: Vec<i32> = invoice
        .postings
        .iter()
        .map(|posting| posting.line.number())
        .collect();
    assert_eq!(lines, [401, 402, 402, 402, 403, 405]);
    assert_eq!(
        texts([invoice.debits, invoice.credits]),
        ["262.00", "262.00"]
    );

    let all = company.journal(Period::all()).unwrap();
    assert_eq!(all.entries().count(), 20);
    assert!(all.entries().all(|entry| entry.debits == entry.credits));
}

#[test]
fn a_page_of_the_journal_is_a_part_of_it() {
    let company = support::books();
    let journal = company.journal(day("2024-01-01")..=day("2024-12-31"));
    let journal = journal.unwrap();
    assert_eq!(journal.row_count(), 16);
    let all: Vec<_> = journal.entries().collect();
    let page = |start, size| -> Vec<_> { journal.page(Page::new(start, size)).collect() };
    assert_eq!(page(0, 5), all[..5]);
    assert_eq!(page(5, 3), all[5..8]);
    assert_eq!(page(14, 5), all[14..]);
    assert_eq!(page(16, 5), []);
}

#[test]
fn sections_say_what_they_are() {
    let company = support::books();
    let sheet = company.balance_sheet(day("2024-06-30")).unwrap();
    let types = sheet.sections().map(|section| section.account_type);
    let expected = [
        AccountType::Bank,
        AccountType::AccountsReceivable,
        AccountType::OtherCurrentAsset,
        AccountType::FixedAsset,
        AccountType::OtherAsset,
        AccountType::AccountsPayable,
        AccountType::CreditCard,
        AccountType::OtherCurrentLiability,
        AccountType::LongTermLiability,
        AccountType::Equity,
    ];
    assert_eq!(types, expected);
    let accounts = company.accounts().unwrap();
    for section in sheet.sections() {
        for (_, account) in section.walk() {
            let account = accounts.get(account.account).unwrap();
            assert_eq!(account.account_type, section.account_type);
        }
    }
    let statement = company.profit_and_loss(Period::all()).unwrap();
    let types = statement.sections().map(|section| section.account_type);
    let expected = [
        AccountType::Income,
        AccountType::CostOfGoodsSold,
        AccountType::Expense,
        AccountType::OtherIncome,
        AccountType::OtherExpense,
    ];
    assert_eq!(types, expected);
}

#[test]
fn profit_and_loss_says_what_it_has_no_section_for() {
    let net = |books: Changed| {
        let company = books.company();
        let statement = company.profit_and_loss(Period::all()).unwrap();
        texts([statement.net_income, statement.unplaced])
    };
    assert_eq!(net(Changed::books()), ["217.00", "0.00"]);
    // An account of a type that no file had, which 150.00 were sold on.
    let sales = ("abmc_account_user", "account_id", 12);
    let unknown = Changed::books().set(sales, "type", whole(16));
    assert_eq!(net(unknown), ["67.00", "-150.00"]);
}

#[test]
fn an_account_listing_has_the_balances_of_the_balance_sheet() {
    let company = support::books();
    let listing = company.account_listing(AsOf::Latest).unwrap();
    assert_eq!(listing.rows.len(), 22);
    let balance = |account: i32| {
        let mut rows = listing.rows.iter();
        let row = rows
            .find(|row| row.account == AccountId::new(account))
            .unwrap();
        row.balance.map(|balance| balance.amount.to_string())
    };
    assert_eq!(balance(1).as_deref(), Some("837.00"));
    assert_eq!(balance(2).as_deref(), Some("150.00"));
    assert_eq!(balance(8).as_deref(), Some("-25.00"));
    // Retained Earnings has what is posted to it, which is nothing.
    assert_eq!(balance(11).as_deref(), Some("0.00"));
    for not_on_the_balance_sheet in [12, 13, 16, 17, 19] {
        assert_eq!(balance(not_on_the_balance_sheet), None);
    }
    let on_a_day = company.account_listing(day("2023-12-31")).unwrap();
    assert_eq!(
        on_a_day.rows[0].total.map(|total| total.to_string()),
        Some("800.00".to_owned())
    );
}

#[test]
fn statements_say_what_is_left_out() {
    let company: Company = support::damaged_books();
    let date = day("2024-06-30");
    let balance = company.trial_balance(date).unwrap();
    assert_eq!(balance.left_out.rows, 1);
    // The line that cannot be read was a credit of 40 in 2023.
    assert_eq!(
        texts([balance.debits, balance.credits]),
        ["1340.00", "1300.00"]
    );
    let sheet = company.balance_sheet(date).unwrap();
    assert_eq!(
        (
            sheet.left_out.rows,
            sheet.equity.earlier_profit.amount().to_string()
        ),
        (1, "90.00".to_owned())
    );
    assert_eq!(
        company
            .profit_and_loss(Period::all())
            .unwrap()
            .left_out
            .rows,
        1
    );
    assert_eq!(
        company.general_ledger(Period::all()).unwrap().left_out.rows,
        1
    );
    assert_eq!(company.journal(Period::all()).unwrap().left_out.rows, 1);
    assert_eq!(
        company.account_listing(AsOf::Latest).unwrap().left_out.rows,
        1
    );
}

#[test]
fn a_section_has_its_accounts_as_they_are_kept() {
    let company = support::books();
    let (accounts, today) = (company.accounts().unwrap(), day("2024-06-30"));
    let sheet = company.balance_sheet(today).unwrap();
    let listing = company.account_listing(today).unwrap();
    let mut compared = 0;
    for section in sheet.sections() {
        for (_, shown) in section.walk() {
            let row = listing.rows.iter().find(|row| row.account == shown.account);
            let Some(balance) = row.and_then(|row| row.balance) else {
                continue;
            };
            let account = accounts.get(shown.account).unwrap();
            // The listing has Retained Earnings as it is posted.
            if account.special != Some(SpecialAccount::RetainedEarnings) {
                let kept = account.account_type.as_kept(balance);
                assert_eq!(kept.home_amount, shown.amount, "{}", account.name);
                compared += 1;
            }
        }
    }
    assert_eq!(compared, 7);
}

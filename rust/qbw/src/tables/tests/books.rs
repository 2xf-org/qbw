//! The synthetic company's books, whose values are typed by hand, are in order.

use std::collections::{BTreeMap, HashMap};

use chrono::NaiveDate;
use sa17::{Database, Source};

use crate::amount::{Amount, Total};
use crate::codes::AccountType;
use crate::ids::{AccountId, CurrencyId, LineId, TransactionId};
use crate::money::Money;
use crate::tables::accounts::{ACCOUNT_TOTALS, ACCOUNTS};
use crate::tables::postings::{InventoryLineRow, PostingLineRow};
use crate::tables::{Form, tests};

/// The lines of one transaction, with what they post besides.
#[derive(Default)]
struct Lines {
    lines: Vec<PostingLineRow>,
    inventory: Vec<InventoryLineRow>,
}

impl Lines {
    fn own(&self) -> &PostingLineRow {
        let mut own = self.lines.iter().filter(|line| line.is_own);
        match (own.next(), own.next()) {
            (Some(line), None) => line,
            _ => panic!("a transaction has one own line"),
        }
    }

    /// The lines from the own one on, as each leads to the next.
    fn chain(&self) -> Vec<&PostingLineRow> {
        let by_id: HashMap<LineId, &PostingLineRow> =
            self.lines.iter().map(|line| (line.id, line)).collect();
        let mut chain = vec![self.own()];
        while let Some(next) = chain.last().and_then(|line| line.next)
            && chain.len() <= self.lines.len()
        {
            chain.push(by_id[&next]);
        }
        chain
    }

    /// Whether the transaction counts in the books.
    fn posts(&self) -> bool {
        let own = self.own();
        !own.is_non_posting && !own.is_memorized
    }

    /// The home amounts by account.
    fn home(&self) -> impl Iterator<Item = (AccountId, Amount)> {
        let lines = self
            .lines
            .iter()
            .map(|line| (line.account, line.home_amount));
        let inventory = self
            .inventory
            .iter()
            .map(|line| (line.account, line.home_amount));
        let posted = lines.chain(inventory);
        posted.filter_map(|(account, amount)| Some((account?, amount.unwrap_or_default())))
    }
}

fn transactions(source: &dyn Source) -> BTreeMap<TransactionId, Lines> {
    let mut transactions: BTreeMap<TransactionId, Lines> = BTreeMap::new();
    let mut transaction_of: HashMap<LineId, TransactionId> = HashMap::new();
    let mut damage = Vec::new();
    for form in Form::all() {
        let Some(lines) = form.lines::<PostingLineRow>().find(source).unwrap() else {
            continue;
        };
        for (_, line) in lines.scan(&mut damage) {
            let other = transaction_of.insert(line.id, line.transaction);
            assert_eq!(other, None, "a line has an id of its own");
            transactions
                .entry(line.transaction)
                .or_default()
                .lines
                .push(line);
        }
        let Some(inventory) = form.stock_lines() else {
            continue;
        };
        for (_, line) in inventory.find(source).unwrap().unwrap().scan(&mut damage) {
            let transaction = transaction_of[&line.line];
            transactions
                .entry(transaction)
                .or_default()
                .inventory
                .push(line);
        }
    }
    assert_eq!(damage, Vec::new());
    transactions
}

/// A sum as it is typed into the figures below.
fn typed(total: Total) -> String {
    total.money(CurrencyId::COMPANY).to_string()
}

#[test]
fn every_transaction_has_its_lines_in_a_chain() {
    let transactions = transactions(&tests::books());
    assert_eq!(transactions.len(), 25);
    for (id, lines) in &transactions {
        let chain = lines.chain();
        assert_eq!(chain.len(), lines.lines.len(), "transaction {id}");
        let own = lines.own();
        // The own line has the account of the first of the others, and the
        // others have the own one's.
        assert_eq!(own.other_account, chain[1].account, "transaction {id}");
        for line in &chain[1..] {
            assert_eq!(line.other_account, own.account, "line {}", line.id);
            assert_eq!(
                (line.kind, line.date),
                (own.kind, own.date),
                "line {}",
                line.id
            );
            assert_eq!(line.is_split, own.is_split, "line {}", line.id);
        }
    }
}

#[test]
fn every_transaction_adds_up_to_zero() {
    for (id, lines) in &transactions(&tests::books()) {
        let home: Total = lines.home().map(|(_, amount)| amount).sum();
        assert!(home.is_zero(), "transaction {id}");
        let foreign = lines.lines.iter().filter(|line| line.account.is_some());
        let foreign: Total = foreign.filter_map(|line| line.foreign_amount).sum();
        assert!(foreign.is_zero(), "transaction {id}");
    }
}

/// The balances by account that are not zero, up to a date.
fn balances(date: &str) -> Vec<(i32, String)> {
    let date: NaiveDate = date.parse().unwrap();
    let mut balances: BTreeMap<i32, Total> = BTreeMap::new();
    for lines in transactions(&tests::books()).values() {
        if !lines.posts() || lines.own().date > date {
            continue;
        }
        for (account, amount) in lines.home() {
            *balances.entry(account.number()).or_default() += amount.into();
        }
    }
    let balances = balances
        .into_iter()
        .filter(|(_, balance)| !balance.is_zero());
    balances
        .map(|(account, balance)| (account, typed(balance)))
        .collect()
}

/// The figures that the books were written for.
fn figures(figures: &[(i32, &str)]) -> Vec<(i32, String)> {
    let figures = figures.iter();
    figures
        .map(|&(account, amount)| (account, amount.to_owned()))
        .collect()
}

#[test]
fn the_balances_are_the_ones_that_the_books_were_written_for() {
    let first_year = figures(&[
        (1, "800.00"),
        (3, "202.00"),
        (6, "140.00"),
        (9, "-12.00"),
        (10, "-1000.00"),
        (12, "-40.00"),
        (13, "-150.00"),
        (14, "60.00"),
    ]);
    assert_eq!(balances("2023-12-31"), first_year);
    let in_all = figures(&[
        (1, "837.00"),
        (2, "180.00"),
        (3, "45.00"),
        (6, "200.00"),
        (7, "-20.00"),
        (8, "-25.00"),
        (10, "-1000.00"),
        (12, "-150.00"),
        (13, "-100.00"),
        (14, "40.00"),
        (15, "-20.00"),
        (16, "25.00"),
        (17, "-15.00"),
        (18, "3.00"),
    ]);
    assert_eq!(balances("2024-06-30"), in_all);
}

#[test]
fn the_euro_bank_has_150_euros() {
    let transactions = transactions(&tests::books());
    let lines = transactions.values().filter(|lines| lines.posts());
    let lines = lines.flat_map(|lines| &lines.lines);
    let lines = lines.filter(|line| line.account == Some(AccountId::new(2)));
    let euros = lines.filter_map(|line| line.foreign_amount);
    assert_eq!(typed(euros.sum()), "150.00");
}

#[test]
fn the_stored_balances_are_the_postings_but_for_one_that_is_out_of_date() {
    let source = tests::books();
    let mut damage = Vec::new();
    let totals = ACCOUNT_TOTALS.open(&source).unwrap();
    let stored: BTreeMap<i32, String> = totals
        .scan(&mut damage)
        .filter_map(|(_, totals)| Some((totals.id.number(), totals.home_balance?)))
        .map(|(account, balance)| {
            (
                account,
                Money::new(balance, CurrencyId::COMPANY).to_string(),
            )
        })
        .collect();
    let balances = balances("2024-06-30").into_iter();
    let differing: Vec<(i32, String)> = balances
        .filter(|(account, balance)| stored.get(account) != Some(balance))
        .collect();
    assert_eq!(differing, figures(&[(8, "-25.00")]));
}

#[test]
fn nothing_that_posts_is_in_an_account_outside_of_the_books() {
    let source = tests::books();
    let mut damage = Vec::new();
    let accounts = ACCOUNTS.open(&source).unwrap();
    let outside: Vec<i32> = accounts
        .scan(&mut damage)
        .filter(|(_, account)| {
            account.is_deleted || account.account_type == AccountType::NonPosting
        })
        .map(|(_, account)| account.id.number())
        .collect();
    assert_eq!(outside, [19, 20, 21, 22]);
    let balances = balances("2024-06-30");
    assert!(
        balances
            .iter()
            .all(|(account, _)| !outside.contains(account))
    );
}

#[test]
fn the_damaged_books_lack_two_rows_and_a_next_line() {
    let books = qbw_synthetic::company::damaged_books().build();
    let source = Database::from_bytes(books).unwrap();
    let mut damage = Vec::new();
    let mut lines = Vec::new();
    for form in Form::all() {
        if let Some(table) = form.lines::<PostingLineRow>().find(&source).unwrap() {
            lines.extend(table.scan(&mut damage).map(|(_, line)| line));
        }
    }
    let [damage] = damage.as_slice() else {
        panic!("one line is damaged: {damage:?}");
    };
    assert_eq!(damage.table, "abmc_invoice_lineitem");
    assert!(lines.iter().all(|line| line.id != LineId::new(403)));
    let own = lines.iter().find(|line| line.id == LineId::new(201));
    assert_eq!(own.unwrap().next, Some(LineId::new(999)));

    let headers = source.table("abmc_credit_cardcharge_header").unwrap();
    let damaged = headers.rows().filter(Result::is_err).count();
    assert_eq!((headers.rows().count(), damaged), (2, 1));
}

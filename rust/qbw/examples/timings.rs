//! Prints how long a company takes to answer, the first time and once more.
//!
//! ```console
//! cargo run --release -p qbw --example timings -- books.qbw
//! ```

use std::env;
use std::error::Error;
use std::process::ExitCode;
use std::time::Instant;

use qbw::aging::AgingOptions;
use qbw::query::{PaidState, TransactionQuery};
use qbw::transactions::TransactionKind;
use qbw::{AsOf, Company, Page, Period, Side};

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let [path] = &arguments[..] else {
        eprintln!("usage: timings <company file>");
        return ExitCode::FAILURE;
    };
    match print(path) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn print(path: &str) -> Result<(), Box<dyn Error>> {
    let page = Page::new(0, 50);
    let invoices = TransactionQuery::new().kinds([TransactionKind::Invoice]);
    // What the questions are about is asked of a company of its own, so that
    // the one that is timed was asked nothing before.
    let subjects = Company::open(path)?;
    let latest = subjects.transactions(&TransactionQuery::new(), Page::new(0, 1))?;
    let latest = latest.rows.first().ok_or("no transactions")?;
    let invoice = subjects.transactions(&invoices, Page::new(0, 1))?;
    let invoice = invoice.rows.first().ok_or("no invoices")?;
    let customer = invoice.name.ok_or("an invoice for nobody")?;
    let accounts = subjects.accounts()?.iter().map(|account| account.id);
    let ledger = subjects.ledger()?;
    let account = accounts.max_by_key(|&account| ledger.of_account(account).len());
    let account = account.ok_or("no accounts")?;
    let stock = subjects.stock()?;
    let item = stock.rows.first().ok_or("nothing in stock")?.item;

    let company = &Company::open(path)?;
    time("the names", || company.names().map(drop))?;
    time("the ledger", || company.ledger().map(drop))?;
    time("a balance sheet", || {
        company.balance_sheet(latest.date).map(drop)
    })?;
    time("a page of invoices", || {
        company.transactions(&invoices, page).map(drop)
    })?;
    time("what a customer has not paid", || {
        let unpaid = invoices.clone().name(customer).paid(PaidState::Unpaid);
        company.transactions(&unpaid, page).map(drop)
    })?;
    time("a text in every transaction", || {
        let text = TransactionQuery::new().text("valve");
        company.transactions(&text, page).map(drop)
    })?;
    time("an invoice with its lines", || {
        company.transaction(invoice.id).map(drop)
    })?;
    time("a page of the longest register", || {
        company.register(account, Period::all(), page).map(drop)
    })?;
    time("the links of an invoice", || {
        company.links(invoice.id).map(drop)
    })?;
    time("everything that is open", || {
        company.open_items(Side::Receivable, AsOf::Latest).map(drop)
    })?;
    time("what every customer owes", || {
        company.names_owed(Side::Receivable, latest.date).map(drop)
    })?;
    time("an aging", || {
        let options = AgingOptions::new(latest.date);
        company.aging(Side::Receivable, &options).map(drop)
    })?;
    time("the stock", || company.stock().map(drop))?;
    time("the movements of an item", || {
        company.movements(item).map(drop)
    })?;
    time("the company at a glance", || {
        company.overview(latest.date, 20).map(drop)
    })?;
    time("a customer at a glance", || {
        company.name_overview(customer, latest.date, 20).map(drop)
    })?;
    time("an item at a glance", || {
        company.item_overview(item, 20).map(drop)
    })?;
    time("an account at a glance", || {
        company.account_overview(account, latest.date, 20).map(drop)
    })
}

/// Prints how long an answer takes, and how long when it is asked for again.
fn time(question: &str, ask: impl Fn() -> qbw::Result<()>) -> Result<(), Box<dyn Error>> {
    let timed = || -> qbw::Result<_> {
        let started = Instant::now();
        ask()?;
        Ok(started.elapsed())
    };
    let (first, again) = (timed()?, timed()?);
    println!("{question:<32} {first:>10.2?} {again:>10.2?}");
    Ok(())
}

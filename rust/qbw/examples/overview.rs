//! Prints a company at a glance, and one of its customers or vendors.
//!
//! ```console
//! cargo run -p qbw --example overview -- books.qbw 2024-06-30 "Ada Stores"
//! ```

use std::env;
use std::error::Error;
use std::process::ExitCode;

use qbw::Company;
use qbw::chrono::NaiveDate;
use qbw::query::TransactionPage;

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let [path, day, name] = &arguments[..] else {
        eprintln!("usage: overview <company file> <day> <customer or vendor>");
        return ExitCode::FAILURE;
    };
    match print(path, day, name) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn print(path: &str, day: &str, name: &str) -> Result<(), Box<dyn Error>> {
    let company = Company::open(path)?;
    let day: NaiveDate = day.parse()?;
    let overview = company.overview(day, 5)?;
    for row in &overview.bank {
        let account = company.accounts()?.full_name(row.account);
        let balance = row.balance.map(|balance| balance.amount.to_string());
        let (account, balance) = (account.unwrap_or_default(), balance.unwrap_or_default());
        println!("{account:<28} {balance:>12}");
    }
    let (receivable, payable) = (&overview.receivable, &overview.payable);
    println!("{:<28} {:>12}", "Customers owe", receivable.home_total);
    println!("{:<28} {:>12}", "  overdue", receivable.home_overdue);
    println!("{:<28} {:>12}", "Vendors are owed", payable.home_total);
    println!("{:<28} {:>12}", "  overdue", payable.home_overdue);

    let names = company.names()?;
    let id = names.named(name).ok_or("no such customer or vendor")?.id();
    let overview = company.name_overview(id, day, 5)?.ok_or("no such name")?;
    println!();
    println!("{}", names.full_name(id).unwrap_or_default());
    if let Some(total) = overview.total {
        println!("{:<28} {total:>12}", "Balance");
    }
    print_transactions(&company, &overview.transactions)
}

/// Prints what the centers show of a transaction: its type, its number, its date,
/// its account and its amount.
fn print_transactions(company: &Company, page: &TransactionPage) -> Result<(), Box<dyn Error>> {
    let accounts = company.accounts()?;
    for row in &page.rows {
        let kind = row.kind.to_string();
        let number = row.number.as_deref().unwrap_or_default();
        let account = row.account.and_then(|account| accounts.full_name(account));
        let account = account.unwrap_or_default();
        let amount = row.amount.map(|amount| amount.to_string());
        let amount = amount.unwrap_or_default();
        println!(
            "{kind:<12} {number:>6} {} {account:<20} {amount:>9}",
            row.date
        );
    }
    println!("{} of {}", page.rows.len(), page.row_count);
    Ok(())
}

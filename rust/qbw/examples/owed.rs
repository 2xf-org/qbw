//! Prints what a customer owes on a day, and what its invoices were paid by.
//!
//! ```console
//! cargo run -p qbw --example owed -- books.qbw "Ada Stores" 2024-06-30
//! ```

use std::env;
use std::error::Error;
use std::process::ExitCode;

use qbw::Company;
use qbw::chrono::NaiveDate;

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let [path, customer, day] = &arguments[..] else {
        eprintln!("usage: owed <company file> <customer> <day>");
        return ExitCode::FAILURE;
    };
    match print(path, customer, day) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn print(path: &str, customer: &str, day: &str) -> Result<(), Box<dyn Error>> {
    let company = Company::open(path)?;
    let customers = company.names()?.customers();
    let customer = customers.named(customer).ok_or("no such customer")?;

    let day: NaiveDate = day.parse()?;
    let owed = company.owed_by(customer.id, day)?;
    let owed = owed.ok_or("no such customer")?;
    println!(
        "owes {}, of which {} is overdue",
        owed.home_total, owed.home_overdue
    );
    for item in &owed.items {
        let number = item.number.as_deref().unwrap_or_default();
        println!(
            "{} {} {number}: {} of {}",
            item.date, item.kind, item.open, item.total
        );
        let Some(paid_by) = company.paid_by(item.transaction)? else {
            continue;
        };
        for link in &paid_by.links {
            let amount = link.amount.map(|amount| amount.to_string());
            let amount = amount.unwrap_or_default();
            println!("    {} {} {amount}", link.other.date, link.relation);
        }
    }
    Ok(())
}

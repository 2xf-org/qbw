//! Prints the first hundred rows of an account's register.
//!
//! ```console
//! cargo run -p qbw --example register -- books.qbw Checking
//! ```

use std::env;
use std::error::Error;
use std::process::ExitCode;

use qbw::{Company, Page, Period};

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let [path, account] = &arguments[..] else {
        eprintln!("usage: register <company file> <account>");
        return ExitCode::FAILURE;
    };
    match print(path, account) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn print(path: &str, account: &str) -> Result<(), Box<dyn Error>> {
    let company = Company::open(path)?;
    let account = company
        .accounts()?
        .named(account)
        .ok_or("no such account")?;
    let register = company.register(account.id, Period::all(), Page::new(0, 100))?;
    let register = register.ok_or("no such account")?;
    for row in &register.rows {
        let number = row.number.as_deref().unwrap_or_default();
        println!(
            "{} {number:>8} {:>12} {:>12}",
            row.date, row.amount, row.balance
        );
    }
    println!("{} rows, {}", register.row_count, register.closing.amount);
    Ok(())
}

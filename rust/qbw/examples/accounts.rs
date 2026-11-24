//! Prints the chart of accounts, with the balances that QuickBooks keeps.
//!
//! ```console
//! cargo run -p qbw --example accounts -- books.qbw
//! ```

use std::env;
use std::process::ExitCode;

use qbw::{Company, Error};

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let [path] = &arguments[..] else {
        eprintln!("usage: accounts <company file>");
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

fn print(path: &str) -> Result<(), Error> {
    let company = Company::open(path)?;
    let accounts = company.accounts()?;
    for account in accounts.active() {
        let name = accounts.full_name(account.id).unwrap_or(&account.name);
        let balance = account.stored.map(|stored| stored.amount.to_string());
        println!("{name:<28} {:>12}", balance.unwrap_or_default());
    }
    let left_out = accounts.left_out();
    if !left_out.is_nothing() {
        println!(
            "{} rows and {} pages cannot be read",
            left_out.rows, left_out.pages
        );
    }
    Ok(())
}

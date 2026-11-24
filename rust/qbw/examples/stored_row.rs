//! Prints the whole stored row of an account, one value per line.
//!
//! ```console
//! cargo run -p qbw --example stored_row -- books.qbw Sales:Parts
//! ```

use std::env;
use std::error::Error;
use std::process::ExitCode;

use qbw::Company;
use qbw::sa17::Source;

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let [path, account] = &arguments[..] else {
        eprintln!("usage: stored_row <company file> <account>");
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
    let origin = account.origin;
    println!("{origin}");

    let table = company.source().definition(origin.table)?;
    let row = company.stored_row(origin)?.ok_or("no such row")?;
    for (column, value) in table.columns().iter().zip(row.values()) {
        println!("{:<28} {value}", column.name());
    }
    Ok(())
}

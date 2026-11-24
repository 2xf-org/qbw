//! Prints the active customers and jobs, with what they owe and how to reach
//! them.
//!
//! ```console
//! cargo run -p qbw --example customers -- books.qbw
//! ```

use std::env;
use std::process::ExitCode;

use qbw::{Company, Error};

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let [path] = &arguments[..] else {
        eprintln!("usage: customers <company file>");
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
    let names = company.names()?;
    for customer in names.customers().active() {
        let name = names.full_name(customer.id).unwrap_or(&customer.name);
        let balance = customer.stored.map(|stored| stored.amount.to_string());
        println!("{name:<28} {:>12}", balance.unwrap_or_default());
        if let Some(contact) = &customer.contact {
            let reached = [&contact.phone, &contact.email].into_iter().flatten();
            for line in reached.chain(&contact.address.printed) {
                println!("    {line}");
            }
        }
    }
    let left_out = names.left_out();
    if !left_out.is_nothing() {
        println!(
            "{} rows and {} pages cannot be read",
            left_out.rows, left_out.pages
        );
    }
    Ok(())
}

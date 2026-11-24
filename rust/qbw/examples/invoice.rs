//! Prints an invoice with its lines.
//!
//! ```console
//! cargo run -p qbw --example invoice -- books.qbw 1041
//! ```

use std::env;
use std::error::Error;
use std::process::ExitCode;

use qbw::Company;
use qbw::transactions::TransactionKind;

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let [path, number] = &arguments[..] else {
        eprintln!("usage: invoice <company file> <number>");
        return ExitCode::FAILURE;
    };
    match print(path, number) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn print(path: &str, number: &str) -> Result<(), Box<dyn Error>> {
    let company = Company::open(path)?;
    let (names, items) = (company.names()?, company.items()?);
    let id = company.find(TransactionKind::Invoice, number)?;
    let id = id.ok_or("there is no such invoice")?;
    let invoice = company.transaction(id)?.ok_or("the invoice is gone")?;

    let customer = invoice.customer().and_then(|id| names.full_name(id));
    println!("{} {}", invoice.date, customer.unwrap_or_default());
    for line in invoice.form_lines() {
        let item = line.item.and_then(|id| items.full_name(id));
        let says = item.or(line.description.as_deref());
        print!("{:<24}", says.unwrap_or_default());
        if let Some(amount) = line.amount {
            print!(" {amount:>10}");
        }
        println!();
    }
    if let (Some(total), Some(open)) = (invoice.total, invoice.open()) {
        println!("{:<24} {total:>10}", "Total");
        println!("{:<24} {open:>10}", "Still to pay");
    }
    Ok(())
}

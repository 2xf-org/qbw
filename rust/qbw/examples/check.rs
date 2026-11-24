//! Checks what is worked out from a company's books against what QuickBooks
//! keeps, and prints what differs, with what explains it or may be behind it. It
//! exits with 2 when a difference is not explained, which somebody has to look
//! at: a figure that QuickBooks has not brought up to date differs, and so does
//! one that a rule of this crate gets wrong.
//!
//! ```console
//! cargo run --release -p qbw --example check -- books.qbw
//! ```

use std::env;
use std::process::ExitCode;

use qbw::check::{Check, Finding, Subject};
use qbw::{Company, Error};

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let [path] = &arguments[..] else {
        eprintln!("usage: check <company file>");
        return ExitCode::FAILURE;
    };
    match print(path) {
        Ok(check) if check.unexplained().next().is_some() => ExitCode::from(2),
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn print(path: &str) -> Result<Check, Error> {
    let company = Company::open(path)?;
    let check = company.check()?;
    for compared in &check.compared {
        let (agreed, looked_at) = (compared.agreed, compared.looked_at);
        println!("{:<48} {agreed:>7} of {looked_at:>7}", compared.what);
    }
    for found in &check.findings {
        println!();
        println!("{}: {}", found.what, subject(&company, found)?);
        if let (Some(computed), Some(expected)) = (found.computed, found.expected) {
            println!("    {computed} instead of {expected}");
        }
        for cause in &found.causes {
            println!("    {cause}");
        }
    }
    for missing in &check.not_compared {
        println!();
        println!("{}: not compared without {}", missing.what, missing.table);
    }
    if !check.left_out.is_nothing() {
        println!();
        println!("some rows of the file cannot be read");
    }
    Ok(check)
}

/// What a finding is about, as a person knows it.
fn subject(company: &Company, found: &Finding) -> Result<String, Error> {
    let name = match &found.subject {
        Subject::Account(id) => company.accounts()?.full_name(*id),
        Subject::Name(id) => company.names()?.full_name(*id),
        Subject::Item(id) | Subject::ItemAt(id, _) => company.items()?.full_name(*id),
        Subject::Transaction(id) => {
            let Some(transaction) = company.transaction(*id)? else {
                return Ok(String::new());
            };
            let number = transaction.number.unwrap_or_default();
            return Ok(format!("{} {number}", transaction.kind));
        }
        Subject::Fact(key) => Some(key.as_str()),
        _ => None,
    };
    Ok(name.unwrap_or_default().to_owned())
}

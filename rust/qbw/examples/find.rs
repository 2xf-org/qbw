//! Prints the transactions that have a text in their number, their memo or the
//! name of who they are with.
//!
//! ```console
//! cargo run -p qbw --example find -- books.qbw ada
//! ```

use std::env;
use std::process::ExitCode;

use qbw::query::{Direction, TransactionOrder, TransactionQuery};
use qbw::{Company, Error, Page};

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let [path, text] = &arguments[..] else {
        eprintln!("usage: find <company file> <text>");
        return ExitCode::FAILURE;
    };
    match print(path, text) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn print(path: &str, text: &str) -> Result<(), Error> {
    let company = Company::open(path)?;
    let query = TransactionQuery::new()
        .text(text)
        .order(TransactionOrder::Date, Direction::Descending);
    let page = company.transactions(&query, Page::new(0, 50))?;
    for row in &page.rows {
        let kind = row.kind.to_string();
        let number = row.number.as_deref().unwrap_or_default();
        let total = row.total.map(|total| total.to_string());
        println!(
            "{} {kind:<16} {number:>8} {:>12}",
            row.date,
            total.unwrap_or_default()
        );
    }
    println!("{} of {}", page.rows.len(), page.row_count);
    if !page.left_out.is_nothing() {
        eprintln!("some rows of the file cannot be read");
    }
    Ok(())
}

//! Prints the active items that are to be ordered, with what there is of them
//! and what is on order.
//!
//! ```console
//! cargo run -p qbw --example reorder -- books.qbw
//! ```

use std::env;
use std::process::ExitCode;

use qbw::{Company, Error};

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let [path] = &arguments[..] else {
        eprintln!("usage: reorder <company file>");
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
    let items = company.items()?;
    let stock = company.stock()?;
    for row in stock.rows.iter().filter(|row| row.needs_reorder()) {
        let Some(item) = items.get(row.item).filter(|item| item.is_active) else {
            continue;
        };
        let name = items.full_name(item.id).unwrap_or(&item.name);
        let (on_hand, on_order) = (row.on_hand, row.on_order);
        println!("{name:<28} {on_hand:>8} on hand {on_order:>8} on order");
    }
    if !stock.left_out.is_nothing() {
        eprintln!("some rows of the file cannot be read");
    }
    Ok(())
}

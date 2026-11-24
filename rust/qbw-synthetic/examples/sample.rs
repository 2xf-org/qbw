//! Writes a synthetic company file, to try qbw without a real one: the
//! sample company, or the one whose books are laid out as QuickBooks does.
//!
//! ```console
//! cargo run -p qbw-synthetic --example sample -- sample.qbw
//! cargo run -p qbw-synthetic --example sample -- books.qbw books
//! ```

use std::env;
use std::fs;
use std::process::ExitCode;

use qbw_synthetic::company;

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let (path, database) = match arguments[..] {
        [path] => (path, company::sample()),
        [path, "books"] => (path, company::books()),
        _ => {
            eprintln!("usage: sample <file to write> [books]");
            return ExitCode::FAILURE;
        }
    };
    match fs::write(path, database.build()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("cannot write {path}: {error}");
            ExitCode::FAILURE
        }
    }
}

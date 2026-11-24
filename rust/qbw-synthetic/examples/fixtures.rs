//! Writes synthetic `.qbw` company fixtures to a directory for integration
//! and Python tests.
//!
//! ```console
//! cargo run -p qbw-synthetic --example fixtures -- /tmp/qbw-fixtures
//! ```

use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use qbw_synthetic::company;

fn main() -> ExitCode {
    let Some(dir) = env::args_os().nth(1) else {
        eprintln!("usage: fixtures <output directory>");
        return ExitCode::FAILURE;
    };
    let dir = Path::new(&dir);
    if let Err(error) = write_all(dir) {
        eprintln!("{error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn write_all(dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    fs::write(dir.join("books.qbw"), company::books().build())?;
    fs::write(
        dir.join("damaged_books.qbw"),
        company::damaged_books().build(),
    )?;
    fs::write(
        dir.join("one_currency.qbw"),
        company::one_currency_books().build(),
    )?;
    fs::write(dir.join("not_a_company.db"), company::sample().build())?;
    Ok(())
}

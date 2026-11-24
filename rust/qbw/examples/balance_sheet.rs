//! Prints the balance sheet of a day.
//!
//! ```console
//! cargo run -p qbw --example balance_sheet -- books.qbw 2024-06-30
//! ```

use std::env;
use std::error::Error;
use std::process::ExitCode;

use qbw::accounts::Account;
use qbw::chrono::NaiveDate;
use qbw::reports::{Section, SectionAccount};
use qbw::{Company, List, Money};

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let [path, day] = &arguments[..] else {
        eprintln!("usage: balance_sheet <company file> <day>");
        return ExitCode::FAILURE;
    };
    match print(path, day) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn print(path: &str, day: &str) -> Result<(), Box<dyn Error>> {
    let company = Company::open(path)?;
    let accounts = company.accounts()?;
    let sheet = company.balance_sheet(day.parse::<NaiveDate>()?)?;

    let assets = &sheet.assets;
    section(accounts, "Bank", &assets.bank);
    section(accounts, "Accounts receivable", &assets.accounts_receivable);
    section(
        accounts,
        "Other current assets",
        &assets.other_current_assets,
    );
    section(accounts, "Fixed assets", &assets.fixed_assets);
    section(accounts, "Other assets", &assets.other_assets);
    total("Total assets", assets.total);

    let liabilities = &sheet.liabilities;
    section(accounts, "Accounts payable", &liabilities.accounts_payable);
    section(accounts, "Credit cards", &liabilities.credit_cards);
    section(
        accounts,
        "Other current liabilities",
        &liabilities.other_current_liabilities,
    );
    section(
        accounts,
        "Long term liabilities",
        &liabilities.long_term_liabilities,
    );
    total("Total liabilities", liabilities.total);

    section(accounts, "Equity", &sheet.equity.accounts);
    total("Net income", sheet.equity.net_income);
    total("Total equity", sheet.equity.total);
    total("Total liabilities and equity", sheet.liabilities_and_equity);
    Ok(())
}

/// Prints a section, unless it has no account.
fn section(accounts: &List<Account>, title: &str, section: &Section) {
    if !section.accounts.is_empty() {
        println!("{title}");
        lines(accounts, &section.accounts, 1);
    }
}

/// Prints accounts and those below them, `depth` steps in.
fn lines(accounts: &List<Account>, shown: &[SectionAccount], depth: usize) {
    for line in shown {
        let account = accounts.get(line.account);
        let name = account.map(|account| account.name.as_str());
        let name = format!("{:indent$}{}", "", name.unwrap_or("-"), indent = depth * 2);
        println!("{name:<34} {:>12}", line.total);
        lines(accounts, &line.children, depth + 1);
    }
}

fn total(title: &str, amount: Money) {
    println!("{title:<34} {amount:>12}");
}

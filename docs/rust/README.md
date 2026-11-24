[Docs](../README.md) / Rust

# Rust API

Read QuickBooks Desktop (`.qbw`) company files in Rust without QuickBooks or a database server.

```toml
[dependencies]
qbw = "0.1"
```

## Quick start

```rust
use qbw::Company;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let company = Company::open("company.qbw")?;
    let accounts = company.accounts()?;
    let parts = accounts.named("Sales:Parts").ok_or("account not found")?;
    println!("{} ({})", parts.name, parts.account_type);

    let as_of = "2024-06-30".parse()?;
    let sheet = company.balance_sheet(as_of)?;
    println!("Assets: {}", sheet.assets.total);
    println!("Net income: {}", sheet.equity.net_income);
    Ok(())
}
```

## Reference

| Page | Covers |
| --- | --- |
| [Company & Database Access](company.md) | Opening files, reading raw rows and `sa17`, company profile, preferences, `Money`, and date ranges |
| [Accounts, Lists & Stock](lists.md) | Chart of accounts, names, items, stock on hand, valuation, open orders, and supporting lists |
| [Transactions, Ledger & Links](transactions.md) | Querying transactions, reading forms and lines, general ledger postings, registers, and links |
| [Reports, Aging & Checks](reports.md) | Financial statements, subledgers, open items, AR/AP aging, overviews, and balance checks |
| [Errors & Partial Reads](errors.md) | `qbw::Error`, `MoneyError`, `LeftOut`, `Damage`, and `Problem` |

## Examples

Runnable examples are in [`rust/qbw/examples/`](../../rust/qbw/examples):

- [`accounts.rs`](../../rust/qbw/examples/accounts.rs): list active accounts and balances.
- [`balance_sheet.rs`](../../rust/qbw/examples/balance_sheet.rs): compute and print a Balance Sheet.
- [`check.rs`](../../rust/qbw/examples/check.rs): compare computed totals against stored balances.
- [`customers.rs`](../../rust/qbw/examples/customers.rs): list customers with contacts and balances.
- [`find.rs`](../../rust/qbw/examples/find.rs): search transactions with `TransactionQuery`.
- [`invoice.rs`](../../rust/qbw/examples/invoice.rs): inspect an invoice line by line.
- [`owed.rs`](../../rust/qbw/examples/owed.rs): list open receivables or payables.
- [`overview.rs`](../../rust/qbw/examples/overview.rs): build a company overview snapshot.
- [`register.rs`](../../rust/qbw/examples/register.rs): print an account register.
- [`reorder.rs`](../../rust/qbw/examples/reorder.rs): find items at or below their reorder point.
- [`stored_row.rs`](../../rust/qbw/examples/stored_row.rs): read the underlying SQL Anywhere row for a typed record.

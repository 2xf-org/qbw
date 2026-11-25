# QuickBooks® Desktop

Read QuickBooks Desktop (`.qbw`) company files in Rust, without QuickBooks or a database server.

[Rust API](https://github.com/2xf-org/qbw/blob/main/docs/rust/README.md) · [Examples](https://github.com/2xf-org/qbw/tree/main/rust/qbw/examples)

Not affiliated with or endorsed by Intuit. QuickBooks is a trademark of Intuit Inc.

## Installing

```toml
[dependencies]
qbw = "=202611.24.0"
```

## Using

```rust
use qbw::Company;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let company = Company::open("company.qbw")?;

    for account in company.accounts()?.active() {
        println!("{:<36}  {}", account.name, account.account_type);
    }

    let as_of: qbw::chrono::NaiveDate = "2024-06-30".parse()?;
    let sheet = company.balance_sheet(as_of)?;
    println!("Assets: {}", sheet.assets.total);
    println!("Net income: {}", sheet.equity.net_income);
    Ok(())
}
```

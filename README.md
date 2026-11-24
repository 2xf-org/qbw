# QuickBooks® Desktop

Read QuickBooks Desktop (`.qbw`) company files in Rust and Python, without QuickBooks or a database server.

[Documentation](docs/README.md) · [Rust](docs/rust/README.md) · [Python](docs/python/README.md)

## Installing

Rust:

```toml
[dependencies]
qbw = "0.1"
```

Python (3.11+):

```bash
pip install qbw
```

## Using

Rust:

```rust
use qbw::Company;

let company = Company::open("company.qbw")?;
let as_of = "2024-06-30".parse()?;
let sheet = company.balance_sheet(as_of)?;

println!("Assets: {}", sheet.assets.total);
println!("Net income: {}", sheet.equity.net_income);
```

Python:

```python
from datetime import date
import qbw

with qbw.Company.open("company.qbw") as company:
    print(company.info().name)

    sheet = company.balance_sheet(date(2024, 6, 30))
    print("Assets:", sheet.assets.total.amount)
    print("Net income:", sheet.equity.net_income.amount)
```

Python with Apache Arrow, Polars, and DuckDB:

```python
import duckdb
import polars as pl
import qbw

with qbw.Company.open("company.qbw") as company:
    accounts = pl.from_arrow(company.accounts_arrow())

    ledger = company.ledger_arrow(batch_size=8192)
    rows = duckdb.sql(
        """
        SELECT account_name, SUM(home_amount) AS balance
        FROM ledger
        GROUP BY account_name
        """
    ).fetchall()
```

## What it reads

| Area | Included |
| --- | --- |
| Lists | Chart of accounts, customers and jobs, vendors, employees, other names, items, classes, terms, currencies, sales reps, tax codes, payment and shipping methods |
| Transactions | Headers and lines, filtered queries, account registers, general ledger postings, payment and order links |
| Reports | Balance Sheet, Profit and Loss, Trial Balance, open receivables and payables, AR and AP aging |
| Inventory | Stock on hand, historical valuation, open sales orders and purchase orders |

## Documentation

- [Documentation](docs/README.md)
- [Rust API](docs/rust/README.md) and [Rust examples](rust/qbw/examples)
- [Python API](docs/python/README.md) and [Python examples](python/examples)

## Layout

| Path | Contents |
| --- | --- |
| [`rust/qbw`](rust/qbw) | Rust library (`qbw` on crates.io) |
| [`rust/qbw-py`](rust/qbw-py) | [Bimetal](https://github.com/2xf-org/bimetal) Python bindings |
| [`rust/qbw-synthetic`](rust/qbw-synthetic) | Test fixture generator |
| [`python/`](python) | Python package (`qbw` on PyPI) |

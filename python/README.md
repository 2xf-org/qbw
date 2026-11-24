# QuickBooks® Desktop

Read QuickBooks Desktop (`.qbw`) company files in Python, without QuickBooks or a database server.

[Python API](https://github.com/2xf-org/qbw/blob/main/docs/python/README.md) · [Rust API](https://github.com/2xf-org/qbw/blob/main/docs/rust/README.md) · [Examples](https://github.com/2xf-org/qbw/tree/main/python/examples)

## Installing

```bash
pip install qbw
```

Requires Python 3.11 or newer.

## Using

```python
from datetime import date
import qbw

with qbw.Company.open("company.qbw") as company:
    print(company.info().name)

    for account in company.accounts(active_only=True):
        print(account.full_name, account.account_type)

    sheet = company.balance_sheet(date(2024, 6, 30))
    print("Assets:", sheet.assets.total.amount)
    print("Net income:", sheet.equity.net_income.amount)
```

## Arrow, Polars, and DuckDB

Export accounts, names, items, transactions, open items, stock, and general ledger postings to Apache Arrow:

```python
import duckdb
import polars as pl
import qbw

with qbw.Company.open("company.qbw") as company:
    accounts = pl.from_arrow(company.accounts_arrow())

    ledger = company.ledger_arrow(batch_size=8192)
    totals = duckdb.sql(
        """
        SELECT account_name, SUM(home_amount) AS balance
        FROM ledger
        GROUP BY account_name
        ORDER BY balance DESC
        """
    ).fetchall()
```

[Docs](../README.md) / Python

# Python API

Read QuickBooks Desktop (`.qbw`) company files in Python with typed Pydantic models and Apache Arrow export.

```bash
pip install qbw
```

## Quick start

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

## Reference

| Page | Covers |
| --- | --- |
| [Company & Options](company.md) | Opening files, file metadata, company profile, preferences, users, common value types, and raw rows |
| [Accounts, Lists & Stock](lists.md) | Chart of accounts, customers, vendors, employees, items, stock on hand, valuation, open orders, and supporting lists |
| [Transactions, Registers & Links](transactions.md) | Searching transactions, reading lines, account registers, and payment or order links |
| [Reports, Aging & Checks](reports.md) | Balance Sheet, Profit and Loss, Trial Balance, open items, AR/AP aging, and balance checks |
| [Apache Arrow](arrow.md) | Arrow table exports, streaming general ledger batches, Polars, DuckDB, and column schemas |
| [Errors & Partial Reads](errors.md) | `QbwError` codes and handling damaged rows with `LeftOut` and `DamageInfo` |

## Examples

Runnable scripts are in [`python/examples/`](../../python/examples):

- [`accounts.py`](../../python/examples/accounts.py): list active accounts with types and stored balances.
- [`reports.py`](../../python/examples/reports.py): print Balance Sheet, Profit and Loss, and Trial Balance.
- [`transactions.py`](../../python/examples/transactions.py): filter unpaid invoices, inspect lines, and follow payment links.
- [`arrow_analytics.py`](../../python/examples/arrow_analytics.py): export Arrow tables and query streaming ledger batches in Polars and DuckDB.
- [`check.py`](../../python/examples/check.py): compare computed totals against stored balances and print findings.

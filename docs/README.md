# Documentation

Read QuickBooks Desktop (`.qbw`) company files in Rust and Python, without QuickBooks or a database server.

## API reference

| Language | Covers |
| --- | --- |
| [Python](python/README.md) | Opening files, accounts, lists, transactions, financial statements, Apache Arrow (PyArrow, Polars, DuckDB), and errors |
| [Rust](rust/README.md) | Opening files, raw database access via `sa17`, accounts, lists, transactions, ledger, reports, and errors |

## Runnable examples

### Python

Run from `python/` with `uv run python examples/<name>.py <company.qbw>`:

| Example | Demonstrates |
| --- | --- |
| [`python/examples/accounts.py`](../python/examples/accounts.py) | Listing active accounts with hierarchy, types, and stored balances |
| [`python/examples/reports.py`](../python/examples/reports.py) | Computing Balance Sheet, Profit and Loss, and Trial Balance as of a date |
| [`python/examples/transactions.py`](../python/examples/transactions.py) | Filtering unpaid invoices, inspecting lines, and following payment links |
| [`python/examples/arrow_analytics.py`](../python/examples/arrow_analytics.py) | Exporting Arrow tables and querying streaming ledger batches in Polars and DuckDB |
| [`python/examples/check.py`](../python/examples/check.py) | Comparing computed balances against summary totals stored by QuickBooks |

### Rust

Run from the repository root with `cargo run -p qbw --example <name> -- <company.qbw>`:

| Example | Demonstrates |
| --- | --- |
| [`rust/qbw/examples/accounts.rs`](../rust/qbw/examples/accounts.rs) | Listing active accounts and balances |
| [`rust/qbw/examples/balance_sheet.rs`](../rust/qbw/examples/balance_sheet.rs) | Printing a Balance Sheet as of a date |
| [`rust/qbw/examples/check.rs`](../rust/qbw/examples/check.rs) | Running the bookkeeping check against stored totals |
| [`rust/qbw/examples/customers.rs`](../rust/qbw/examples/customers.rs) | Listing customers with contacts and balances |
| [`rust/qbw/examples/find.rs`](../rust/qbw/examples/find.rs) | Filtering transactions with `TransactionQuery` |
| [`rust/qbw/examples/invoice.rs`](../rust/qbw/examples/invoice.rs) | Reading an invoice line by line |
| [`rust/qbw/examples/owed.rs`](../rust/qbw/examples/owed.rs) | Listing open receivables or payables |
| [`rust/qbw/examples/overview.rs`](../rust/qbw/examples/overview.rs) | Building a company overview snapshot |
| [`rust/qbw/examples/register.rs`](../rust/qbw/examples/register.rs) | Printing an account register with running balances |
| [`rust/qbw/examples/reorder.rs`](../rust/qbw/examples/reorder.rs) | Finding inventory items at or below their reorder point |
| [`rust/qbw/examples/stored_row.rs`](../rust/qbw/examples/stored_row.rs) | Reading the underlying SQL Anywhere row for a typed record |

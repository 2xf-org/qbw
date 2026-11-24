[Docs](../README.md) / [Python](README.md) / Apache Arrow

# Apache Arrow

Export accounts, names, items, transactions, open items, inventory stock, and general ledger postings directly to Apache Arrow. Monetary amounts and quantities use `decimal128(28, 5)`, and dates use `date32`.

## Quick start (PyArrow, Polars, and DuckDB)

```python
import duckdb
import polars as pl
import qbw

with qbw.Company.open("company.qbw") as company:
    # PyArrow Table -> Polars DataFrame
    accounts = pl.from_arrow(company.accounts_arrow())

    # Streaming RecordBatchReader -> DuckDB SQL
    ledger = company.ledger_arrow(batch_size=8192)
    rows = duckdb.sql(
        """
        SELECT account_name, account_type, SUM(home_amount) AS balance
        FROM ledger
        GROUP BY account_name, account_type
        ORDER BY balance DESC
        """
    ).fetchall()
```

If you use an explicit DuckDB connection instead of `duckdb.sql()`, register the stream first with `conn.register("ledger", ledger)`.

## Export Methods on `Company`

| Method | Returns | Description |
| --- | --- | --- |
| `accounts_arrow(active_only=False)` | `pyarrow.Table` | Chart of accounts with hierarchy depth, types, and stored balances. |
| `names_arrow(kind=None, active_only=False)` | `pyarrow.Table` | Customers, vendors, employees, and other names with contact info and stored balances. |
| `items_arrow(active_only=False)` | `pyarrow.Table` | Items with pricing, linked accounts, and stored inventory figures. |
| `transactions_arrow(start=None, end=None)` | `pyarrow.Table` | Transaction summaries ordered by date ascending. |
| `ledger_arrow(start=None, end=None, account_id=None, batch_size=65536)` | `pyarrow.RecordBatchReader` | Streaming general ledger postings in batches of `batch_size` rows. |
| `open_items_arrow(side, as_of=None)` | `pyarrow.Table` | Open receivables (`Side.Receivable`) or payables (`Side.Payable`). |
| `stock_arrow()` | `pyarrow.Table` | Current inventory quantities, values, average costs, orders, and reorder flags. |

---

## Schemas

### `ledger_arrow()`

| Column | Arrow Type | Nullable | Description |
| --- | --- | --- | --- |
| `date` | `date32` | No | Transaction posting date. |
| `transaction_id` | `int32` | No | Transaction ID. |
| `line_id` | `int32` | No | Transaction line ID. |
| `kind` | `utf8` | No | Transaction kind label. |
| `source` | `utf8` | No | Posting source (`own_line`, `line`, `stock_value`, `cost_of_goods_sold`, `other_stock`). |
| `account_id` | `int32` | No | Account ID. |
| `account_name` | `utf8` | Yes | Full hierarchical account name. |
| `account_type` | `utf8` | Yes | Account type label. |
| `name_id` | `int32` | Yes | Customer, vendor, employee, or other name ID. |
| `name` | `utf8` | Yes | Full name. |
| `item_id` | `int32` | Yes | Item ID. |
| `item_name` | `utf8` | Yes | Full item name. |
| `currency_id` | `int32` | No | Account currency ID. |
| `currency_code` | `utf8` | Yes | ISO 4217 currency code. |
| `amount` | `decimal128(28, 5)` | No | Posting amount in the account currency (debit positive). |
| `home_amount` | `decimal128(28, 5)` | No | Posting amount in the home currency (debit positive). |
| `debit` | `decimal128(28, 5)` | Yes | Home-currency debit amount when positive or zero. |
| `credit` | `decimal128(28, 5)` | Yes | Home-currency credit amount (positive) when credit. |
| `cleared` | `utf8` | No | Reconciliation state (`not_cleared`, `cleared`, `reconciled`, `matched`). |

### `accounts_arrow()`

| Column | Arrow Type | Nullable |
| --- | --- | --- |
| `id` | `int32` | No |
| `name` | `utf8` | No |
| `full_name` | `utf8` | No |
| `parent_id` | `int32` | Yes |
| `depth` | `uint32` | No |
| `account_type` | `utf8` | No |
| `normal_side` | `utf8` | Yes |
| `statement` | `utf8` | Yes |
| `number` | `utf8` | Yes |
| `description` | `utf8` | Yes |
| `currency_id` | `int32` | No |
| `currency_code` | `utf8` | Yes |
| `is_active` | `bool` | No |
| `stored_balance` | `decimal128(28, 5)` | Yes |
| `stored_home_balance` | `decimal128(28, 5)` | Yes |

### `transactions_arrow()`

| Column | Arrow Type | Nullable |
| --- | --- | --- |
| `id` | `int32` | No |
| `kind` | `utf8` | No |
| `date` | `date32` | No |
| `number` | `utf8` | Yes |
| `name_id` | `int32` | Yes |
| `name` | `utf8` | Yes |
| `account_id` | `int32` | Yes |
| `account_name` | `utf8` | Yes |
| `memo` | `utf8` | Yes |
| `currency_id` | `int32` | No |
| `currency_code` | `utf8` | Yes |
| `amount` | `decimal128(28, 5)` | Yes |
| `total` | `decimal128(28, 5)` | Yes |
| `home_total` | `decimal128(28, 5)` | Yes |
| `due_date` | `date32` | Yes |
| `open_amount` | `decimal128(28, 5)` | Yes |
| `is_paid` | `bool` | Yes |
| `is_voided` | `bool` | No |
| `posting` | `utf8` | No |

### `names_arrow()`

| Column | Arrow Type | Nullable |
| --- | --- | --- |
| `id` | `int32` | No |
| `kind` | `utf8` | No |
| `name` | `utf8` | No |
| `full_name` | `utf8` | No |
| `parent_id` | `int32` | Yes |
| `is_active` | `bool` | No |
| `currency_id` | `int32` | Yes |
| `currency_code` | `utf8` | Yes |
| `company` | `utf8` | Yes |
| `email` | `utf8` | Yes |
| `phone` | `utf8` | Yes |
| `stored_balance` | `decimal128(28, 5)` | Yes |
| `stored_home_balance` | `decimal128(28, 5)` | Yes |
| `opened` | `date32` | Yes |

### `items_arrow()`

| Column | Arrow Type | Nullable |
| --- | --- | --- |
| `id` | `int32` | No |
| `name` | `utf8` | No |
| `full_name` | `utf8` | No |
| `parent_id` | `int32` | Yes |
| `kind` | `utf8` | No |
| `is_stocked` | `bool` | No |
| `is_active` | `bool` | No |
| `description` | `utf8` | Yes |
| `price` | `decimal128(28, 5)` | Yes |
| `cost` | `decimal128(28, 5)` | Yes |
| `income_account_id` | `int32` | Yes |
| `income_account_name` | `utf8` | Yes |
| `expense_account_id` | `int32` | Yes |
| `expense_account_name` | `utf8` | Yes |
| `asset_account_id` | `int32` | Yes |
| `asset_account_name` | `utf8` | Yes |
| `part_number` | `utf8` | Yes |
| `on_hand` | `decimal128(28, 5)` | Yes |
| `stored_value` | `decimal128(28, 5)` | Yes |
| `stored_average_cost` | `decimal128(28, 5)` | Yes |

### `open_items_arrow()`

| Column | Arrow Type | Nullable |
| --- | --- | --- |
| `transaction_id` | `int32` | No |
| `kind` | `utf8` | No |
| `date` | `date32` | No |
| `number` | `utf8` | Yes |
| `name_id` | `int32` | Yes |
| `name` | `utf8` | Yes |
| `account_id` | `int32` | No |
| `account_name` | `utf8` | Yes |
| `due_date` | `date32` | Yes |
| `currency_id` | `int32` | No |
| `currency_code` | `utf8` | Yes |
| `total` | `decimal128(28, 5)` | No |
| `open` | `decimal128(28, 5)` | No |
| `home_open` | `decimal128(28, 5)` | No |

### `stock_arrow()`

| Column | Arrow Type | Nullable |
| --- | --- | --- |
| `item_id` | `int32` | No |
| `item_name` | `utf8` | Yes |
| `on_hand` | `decimal128(28, 5)` | No |
| `value` | `decimal128(28, 5)` | No |
| `average_cost` | `decimal128(28, 5)` | Yes |
| `counted` | `decimal128(28, 5)` | No |
| `on_order` | `decimal128(28, 5)` | No |
| `on_sales_orders` | `decimal128(28, 5)` | No |
| `to_be_built` | `decimal128(28, 5)` | No |
| `required_for_builds` | `decimal128(28, 5)` | No |
| `available` | `decimal128(28, 5)` | No |
| `reorder_point` | `decimal128(28, 5)` | Yes |
| `needs_reorder` | `bool` | No |

[Docs](../README.md) / [Rust](README.md) / Errors & Partial Reads

# Errors & Partial Reads

Handle `qbw::Error`, currency mismatches (`MoneyError`), and damaged records reported via `LeftOut` and `Damage`.

## `qbw::Error`

`Error` is returned when an operation cannot run or when a specifically requested record (such as a transaction header) cannot be read.

| Variant | Fields | When returned |
| --- | --- | --- |
| `Error::Database(sa17::Error)` | Wrapped `sa17::Error` | Opening or reading the underlying SQL Anywhere 17 file failed. |
| `Error::NotACompany` | `table: &'static str` | Database lacks core QuickBooks tables (`abmc_account_user` or `abmc_preference_user`). |
| `Error::NoTable` | `table: &'static str` | Company file lacks a table required for the requested query. |
| `Error::Schema` | `table: &'static str`, `column: &'static str` | A required column is missing from a QuickBooks table. |
| `Error::Damaged(Damage)` | `Damage` | The specifically requested record is damaged and cannot be read. |

---

## `MoneyError`

Returned by `Money::checked_add` and `Money::checked_sub`.

| Variant | Fields | Description |
| --- | --- | --- |
| `MoneyError::Currencies` | `left: CurrencyId`, `right: CurrencyId` | Tried to add or subtract amounts in different currencies. |
| `MoneyError::TooLarge` | None | Result exceeded `Decimal` precision bounds. |

---

## Partial Reads (`LeftOut`, `Damage`, `Problem`)

A damaged row in a list or ledger does not abort reading. Every list, balance, statement, and overview carries a `LeftOut` summarizing how many unreadable rows or pages were skipped:

```rust
use qbw::{AsOf, Company, Side};

let company = Company::open("company.qbw")?;
let ar = company.open_items(Side::Receivable, AsOf::Latest)?;
let ap = company.open_items(Side::Payable, AsOf::Latest)?;

if !ar.left_out.is_nothing() {
    println!("AR skipped {} rows, {} pages", ar.left_out.rows, ar.left_out.pages);
}

// Deduplicate overlapping skipped counts across results that read the same tables:
let combined = ar.left_out.union(ap.left_out);
println!("Combined skipped rows: {}", combined.rows);
```

### `LeftOut`

| Member | Description |
| --- | --- |
| `rows: u32` | Individual unreadable rows skipped. |
| `pages: u32` | Whole unreadable pages skipped. |
| `unread_tables: u32` | Tables with rows that `qbw` does not read. |
| `is_nothing()` | Returns `true` when `rows == 0 && pages == 0 && unread_tables == 0`. |
| `with(other)` | Sum skipped counts from independent tables. |
| `union(other)` | Deduplicate skipped counts from results built on shared tables. |

### `Damage`

| Field | Type | Description |
| --- | --- | --- |
| `table` | `&'static str` | Underlying table name. |
| `row` | `Option<sa17::RowId>` | Row ID of the damaged record, when known. |
| `problem` | `Problem` | Specific failure encountered on this record. |

### `Problem`

| Variant | Description |
| --- | --- |
| `Problem::Page` | Unreadable database page and underlying reason. |
| `Problem::Row` | Unreadable table row. |
| `Problem::Column` | Column value failed type or range validation. |
| `Problem::Repeated` | Duplicate primary key row encountered; kept once. |
| `Problem::NoLine` | Inventory posting whose parent transaction line could not be read. |
| `Problem::Header` | Transaction header could not be read. |
| `Problem::LineChain` | Linked chain of transaction lines is broken. |
| `Problem::OwnLine` | Transaction has no header line. |
| `Problem::Currencies` | Transaction lines mix more than one currency. |

[Docs](../README.md) / [Python](README.md) / Errors & Partial Reads

# Errors & Partial Reads

Catch `QbwError` exceptions and check `LeftOut` and `DamageInfo` when a company file contains damaged rows or pages.

## `QbwError`

Every fallible operation raises `qbw.QbwError` (a subclass of `RuntimeError`) with a `code: QbwErrorCode` string and optional structured `details`.

```python
import qbw

try:
    with qbw.Company.open("not_a_company.db") as company:
        print(company.info().name)
except qbw.QbwError as err:
    print(err.code, err.details, str(err))
```

### Error Codes (`QbwErrorCode`)

| `code` | `details` fields | When raised |
| --- | --- | --- |
| `"io"` | `message: str` | Operating system failed to open or read the file. |
| `"format"` | `message: str` | File is not a valid SQL Anywhere 17 (`3.201.12`) database. |
| `"no_key"` | `None` | No SQL Anywhere scrambling key matches the file's pages. |
| `"ambiguous_key"` | `keys: list[StoreKey]` | Multiple scrambling keys fit equally well; pass one in `OpenOptions`. |
| `"wrong_key"` | `None` | The key passed in `OpenOptions` does not match the file. |
| `"unsupported_encoding"` | `label: str` | File declares a text encoding other than `windows-1252` or `UTF-8`. |
| `"database"` | `message: str` | Another error inside the underlying `sa17` database reader. |
| `"not_a_company"` | `table: str` | Valid SQL Anywhere 17 file, but missing core QuickBooks tables (`abmc_account_user` or `abmc_preference_user`). |
| `"no_table"` | `table: str` | Company file lacks a table required by the requested operation. |
| `"schema"` | `table: str`, `column: str` | A QuickBooks table lacks a required column. |
| `"damaged"` | `table: str`, `row: str \| None`, `message: str` | The specifically requested record (such as a transaction header) is damaged. |
| `"invalid_argument"` | `message: str` | Invalid argument (such as an unknown transaction kind, `days=0`, or `batch_size=0`). |
| `"arrow"` | `message: str` | Exporting data to Apache Arrow failed. |

---

## Partial Reads (`LeftOut` and `DamageInfo`)

A damaged row in a table does not stop list reads, ledger builds, or financial reports. `qbw` skips the unreadable record and reports how many rows or pages were skipped in `left_out`:

```python
import qbw

with qbw.Company.open("company.qbw") as company:
    sheet = company.balance_sheet()
    if sheet.left_out.rows or sheet.left_out.pages:
        print(
            f"Incomplete balance sheet: {sheet.left_out.rows} rows, "
            f"{sheet.left_out.pages} pages skipped"
        )
```

### `DamageInfo`

Returned in `Transaction.damage` when individual lines or supporting references on a transaction could not be read.

| Field | Type | Description |
| --- | --- | --- |
| `table` | `str` | Table containing the damaged record. |
| `row` | `str \| None` | Row address string, when known. |
| `problem` | `str` | Explanation of what failed. |

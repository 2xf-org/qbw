[Docs](../README.md) / [Python](README.md) / Company & Options

# Company & Options

Open a `.qbw` company file from disk or memory, configure open options, read company metadata and preferences, or inspect the underlying SQL Anywhere rows.

## `Company`

`Company` holds an open QuickBooks Desktop (`.qbw`) file. Lists, the general ledger, and transaction links are loaded on demand and cached for the lifetime of the handle. Use a `with` block so native memory is released when you are done.

```python
from datetime import date
import qbw

with qbw.Company.open("company.qbw") as company:
    info = company.info()
    prefs = company.preferences()
    print(info.name, prefs.home_currency_code, prefs.fiscal_year_start)
    print(company.fiscal_year(date(2024, 6, 30)))
```

### Constructors

| Call | Returns | Description |
| --- | --- | --- |
| `Company(path, options=None)` | `Company` | Open a `.qbw` file from disk (`path: str`). Same as `Company.open()`. |
| `Company.open(path, options=None)` | `Company` | Open a `.qbw` file from disk read-only. |
| `Company.from_bytes(data, options=None)` | `Company` | Open a `.qbw` file from `bytes` already in memory. |

Raises `QbwError` on I/O failure (`io`), invalid SQL Anywhere format (`format`), scrambling key failures (`no_key`, `ambiguous_key`, `wrong_key`), unsupported text encoding (`unsupported_encoding`), or a database missing core QuickBooks tables (`not_a_company`).

### Properties

| Property | Type | Description |
| --- | --- | --- |
| `file_info` | `FileInfo` | SQL Anywhere 17 file metadata (format version, page size, page count, encoding, table count, LSN, digest). |
| `key` | `StoreKey` | Page-scrambling key recovered for (or passed to) the file. |
| `closed` | `bool` | Whether `close()` has already been called. |

### Metadata and raw row methods

| Method | Returns | Description |
| --- | --- | --- |
| `info()` | `CompanyInfo` | Company name, legal address, contact details, start date, and QuickBooks product version. |
| `preferences()` | `Preferences` | Home currency, fiscal year start month, multicurrency, inventory, tax, and closing date settings. |
| `users()` | `list[User]` | Audit-trail users recorded on saved and deleted transactions. |
| `fiscal_year(date)` | `Period` | Inclusive start and end dates of the fiscal year containing `date`. |
| `transaction_days()` | `Period \| None` | Earliest and latest transaction dates in the file, or `None` if there are no transactions. |
| `table_names()` | `list[str]` | Names of all SQL Anywhere tables in the file. |
| `stored_row(origin)` | `dict[str, Any] \| None` | Raw SQL Anywhere row at `origin`, including columns not mapped to typed models. |
| `close()` | `None` | Release native resources. Called automatically on exit from a `with` block. |

Every major record (`Account`, `NameEntry`, `Item`, `Transaction`, `TransactionLine`, `RegisterRow`, `OpenItem`) includes an `origin` field pointing to its database row. Pass it to `stored_row()` to read unmapped QuickBooks columns:

```python
import qbw

with qbw.Company.open("company.qbw") as company:
    account = company.account_named("Sales:Parts")
    if account is not None:
        raw = company.stored_row(account.origin)
        print(raw)
```

---

## `OpenOptions` and `StoreKey`

Pass `OpenOptions` to `Company.open()` or `Company.from_bytes()` when you want to supply an explicit scrambling key or raise the decompression limit for large blobs.

```python
import qbw

options = qbw.OpenOptions(
    key=qbw.StoreKey(mask=0x12345, addend=0x987654, sign=qbw.KeySign.Positive),
    longest_value=32 * 1024 * 1024,
)
with qbw.Company.open("company.qbw", options=options) as company:
    print(company.info().name)
```

### `OpenOptions`

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `key` | `StoreKey \| None` | `None` | Explicit page-scrambling key to skip automatic key search. |
| `longest_value` | `int \| None` | `None` | Maximum decompressed byte length for a single compressed value (defaults to 16 MiB). |

### `StoreKey`

| Field | Type | Description |
| --- | --- | --- |
| `mask` | `int` | 25-bit XOR mask applied to the page number. |
| `addend` | `int` | 25-bit addend applied after the mask. |
| `sign` | `KeySign` | `KeySign.Positive` or `KeySign.Negative`. |

---

## `FileInfo`

| Field | Type | Description |
| --- | --- | --- |
| `format_version` | `str` | SQL Anywhere format version string (for example `"3.201.12"`). |
| `page_size` | `int` | Page size in bytes (`4096`). |
| `page_count` | `int` | Total number of pages in the file. |
| `encoding` | `TextEncoding` | `TextEncoding.Windows1252` or `TextEncoding.Utf8`. |
| `table_count` | `int` | Number of catalog entries in the database. |
| `lsn` | `int` | Log sequence number from page 0. |
| `digest` | `str` | Content fingerprint for the file and its current write state. |

---

## `CompanyInfo` and `ProductVersion`

### `CompanyInfo`

| Field | Type | Description |
| --- | --- | --- |
| `name` | `str` | Company display name. |
| `legal_name` | `str \| None` | Legal business name. |
| `legal_address` | `Address` | Structured legal address. |
| `address_lines` | `list[str]` | Address lines formatted for printing on forms. |
| `phone` | `str \| None` | Primary phone number. |
| `fax` | `str \| None` | Fax number. |
| `email` | `str \| None` | Company email address. |
| `website` | `str \| None` | Company website URL. |
| `started` | `datetime.date \| None` | Start date of the company file. |
| `created` | `datetime.datetime \| None` | UTC creation timestamp of the file. |
| `version` | `ProductVersion` | QuickBooks Desktop product version details. |
| `left_out` | `LeftOut` | Skipped rows and pages if the file has damage. |

### `ProductVersion`

| Field | Type | Description |
| --- | --- | --- |
| `year` | `int \| None` | Release year (for example `2024`). |
| `update` | `int \| None` | Release update number. |
| `edition` | `str \| None` | Raw edition code recorded in the file. |
| `newest_used` | `str \| None` | Newest version string that has opened the file. |

---

## `Preferences` and `User`

### `Preferences`

| Field | Type | Description |
| --- | --- | --- |
| `home_currency_id` | `int` | Home currency ID (`0` in single-currency files). |
| `home_currency_code` | `str \| None` | ISO 4217 code of the home currency (for example `"USD"`). |
| `uses_multicurrency` | `bool` | Whether multicurrency is enabled. |
| `fiscal_year_start` | `int` | Starting month of the fiscal year (`1` = January to `12` = December). |
| `tax_year_start` | `int \| None` | Starting month of the income tax year, if set. |
| `aging_basis` | `AgingBasis` | `AgingBasis.DueDate` or `AgingBasis.TransactionDate`. |
| `uses_account_numbers` | `bool` | Whether account numbers are shown. |
| `uses_classes` | `bool` | Whether class tracking is enabled. |
| `uses_inventory` | `bool` | Whether inventory tracking is enabled. |
| `uses_sales_tax` | `bool` | Whether sales tax tracking is enabled. |
| `uses_sales_orders` | `bool` | Whether sales orders are enabled. |
| `uses_sites` | `bool` | Whether multi-site inventory is enabled. |
| `uses_bins` | `bool` | Whether bin tracking within sites is enabled. |
| `uses_serial_numbers` | `bool` | Whether serial number tracking is enabled. |
| `available_less_sales_orders` | `bool` | Whether open sales orders reduce available stock. |
| `available_less_builds` | `bool` | Whether pending builds reduce available stock. |
| `default_tax_item_id` | `int \| None` | Default sales tax item ID for new customers. |
| `closing_date` | `str \| None` | Closing date text stored in the file. |
| `left_out` | `LeftOut` | Skipped rows and pages if the file has damage. |

### `User`

| Field | Type | Description |
| --- | --- | --- |
| `id` | `int` | Audit-trail user ID. |
| `names` | `list[str]` | Sign-in names seen for this user, newest first. |

---

## Common Value Models

### `Money`

Exact fixed-point amount paired with its currency. Pass `money.amount` to `decimal.Decimal` when doing arithmetic in Python.

| Field | Type | Description |
| --- | --- | --- |
| `amount` | `str` | Exact decimal string (for example `"-100.00"`). |
| `currency_id` | `int` | Currency ID (`0` in single-currency files). |
| `currency_code` | `str \| None` | ISO 4217 currency code, if set. |

### `Price`

| Field | Type | Description |
| --- | --- | --- |
| `kind` | `PriceKind` | `PriceKind.Each` (unit price) or `PriceKind.Percent` (percentage rate). |
| `amount` | `str` | Exact decimal string. |
| `currency_id` | `int \| None` | Currency ID when `kind == PriceKind.Each`. |
| `currency_code` | `str \| None` | ISO 4217 code when `kind == PriceKind.Each` and set. |

### `Address`

| Field | Type | Description |
| --- | --- | --- |
| `printed` | `list[str]` | Address lines as formatted for printing on forms. |
| `street` | `list[str]` | Street lines. |
| `city` | `str \| None` | City name. |
| `state` | `str \| None` | State or province. |
| `postal_code` | `str \| None` | Postal or ZIP code. |
| `country` | `str \| None` | Country name. |

### `Period`

| Field | Type | Description |
| --- | --- | --- |
| `start` | `datetime.date \| None` | Inclusive start date, or `None` if unbounded. |
| `end` | `datetime.date \| None` | Inclusive end date, or `None` if unbounded. |

### `Origin`

| Field | Type | Description |
| --- | --- | --- |
| `table` | `str` | Underlying SQL Anywhere table name. |
| `page` | `int` | Zero-based page number. |
| `slot` | `int` | Zero-based slot index within the page. |

### `LeftOut`

Summary of unreadable data skipped during a read. See [Errors & Partial Reads](errors.md) for details.

| Field | Type | Description |
| --- | --- | --- |
| `rows` | `int` | Individual unreadable rows skipped. |
| `pages` | `int` | Whole unreadable pages skipped. |
| `unread_tables` | `int` | Tables with rows that `qbw` does not read. |

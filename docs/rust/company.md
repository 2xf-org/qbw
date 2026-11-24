[Docs](../README.md) / [Rust](README.md) / Company & Database Access

# Company & Database Access

Open `.qbw` company files, read raw SQL Anywhere rows or tables through `sa17`, and work with exact currency amounts and date spans.

## Opening a `Company`

`Company<S = sa17::Database>` loads lists, the general ledger, and links on demand, caches them, and is `Send + Sync`.

```rust
use qbw::sa17::OpenOptions;
use qbw::Company;

let company = Company::open("company.qbw")?;

let options = OpenOptions::new().longest_value(32 * 1024 * 1024);
let configured = Company::open_with("company.qbw", &options)?;
```

### Constructors

| Method | Returns | Description |
| --- | --- | --- |
| `Company::open(path)` | `Result<Company>` | Open a `.qbw` file from disk read-only. |
| `Company::open_with(path, &options)` | `Result<Company>` | Open a `.qbw` file with explicit `sa17::OpenOptions` (key override or decompression bound). |
| `Company::from_bytes(bytes)` | `Result<Company>` | Open a `.qbw` file from bytes already in memory. |
| `Company::from_bytes_with(bytes, &options)` | `Result<Company>` | Open from bytes with explicit `sa17::OpenOptions`. |
| `Company::try_new(source)` | `Result<Company<S>>` | Wrap any `sa17::Source` implementation. |

---

## Raw Database Access

Every typed record carries an `Origin` pointing to its database row so you can inspect unmapped columns or query the underlying `sa17::Database` directly:

```rust
use qbw::sa17::Source;
use qbw::Company;

let company = Company::open("company.qbw")?;
let accounts = company.accounts()?;
let parts = accounts.named("Sales:Parts").ok_or("no such account")?;

// Read the full underlying SQL Anywhere row for this account
let table = company.source().definition(parts.origin.table)?;
let row = company.stored_row(parts.origin)?.ok_or("no such row")?;
for (column, value) in table.columns().iter().zip(row.values()) {
    println!("{:<28} {value}", column.name());
}
```

### Database and Metadata Methods on `Company`

| Method | Returns | Description |
| --- | --- | --- |
| `source()` | `&S` | Borrow the underlying `sa17::Database` (or custom `Source`). |
| `into_source()` | `S` | Consume `Company` and return the underlying source. |
| `stored_row(origin)` | `Result<Option<sa17::Row>>` | Fetch the full raw database row for a record's `Origin`. |
| `info()` | `Result<&CompanyInfo>` | Company profile metadata, legal address, and QuickBooks `ProductVersion`. |
| `preferences()` | `Result<&Preferences>` | Home currency, fiscal year start month, multicurrency, inventory, and feature flags. |
| `facts()` | `Result<&Facts>` | Key-value summary figures stored by QuickBooks. |
| `users()` | `Result<&Users>` | Audit-trail users reconstructed from transaction history. |
| `fiscal_year(date)` | `Result<Period>` | Fiscal year `Period` containing `date`. |
| `transaction_days()` | `Result<Option<Period>>` | Inclusive span from earliest to latest transaction date in the file. |

---

## Amounts and Dates

### `Money` and `Totals`

`Money` pairs an exact `sa17::Decimal` with a `CurrencyId`. Adding or subtracting mismatched currencies returns `MoneyError::Currencies`. `Totals` accumulates amounts per currency without mixing them.

| Method | Returns | Description |
| --- | --- | --- |
| `Money::new(amount, currency)` | `Money` | Construct an amount in one currency. |
| `money.amount()` | `sa17::Decimal` | Underlying exact decimal value. |
| `money.currency()` | `CurrencyId` | Currency ID (`CurrencyId::COMPANY` in single-currency files). |
| `money.checked_add(other)` | `Result<Money, MoneyError>` | Exact sum when currencies match. |
| `money.checked_sub(other)` | `Result<Money, MoneyError>` | Exact difference when currencies match. |
| `totals.get(currency)` | `Money` | Sum for one currency (zero if absent). |
| `totals.single()` | `Option<Money>` | The single total when only one currency is present. |

### `Period` and `AsOf`

`qbw` never reads the system clock. Point-in-time queries take `impl Into<AsOf>` (`AsOf::Latest` or a `chrono::NaiveDate`), and range queries take `impl Into<Period>`.

| Type / Constructor | Description |
| --- | --- |
| `AsOf::Latest` | Everything in the file, including future-dated entries. |
| `AsOf::Date(date)` | End of `date` (entries dated `<= date`). `NaiveDate` converts via `From`. |
| `Period::all()` | Unbounded on both ends. |
| `Period::between(first, last)` | Inclusive range `first..=last`. |
| `Period::starting(first)` | All dates `>= first`. |
| `Period::until(last)` | All dates `<= last`. |
| `Period::month_of(date)` | Calendar month containing `date`. |
| `Period::year_of(date, month)` | Fiscal year starting in `month` containing `date`. |

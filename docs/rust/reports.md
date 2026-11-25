[Docs](../README.md) / [Rust](README.md) / Reports, Aging & Checks

# Reports, Aging & Checks

Compute financial statements, subledgers, open receivables and payables, aging summaries, overviews, and balance cross-checks.

## Financial Statements (`qbw::reports`)

```rust
use qbw::Company;

let company = Company::open("company.qbw")?;
let day = "2024-06-30".parse()?;

let sheet = company.balance_sheet(day)?;
assert_eq!(sheet.liabilities_and_equity, sheet.assets.total);

let pnl = company.profit_and_loss(company.fiscal_year(day)?)?;
assert_eq!(pnl.net_income, sheet.equity.net_income);
```

### Report Methods on `Company`

| Method | Returns | Description |
| --- | --- | --- |
| `balance_sheet(as_of)` | `Result<BalanceSheet>` | Assets, liabilities, equity, prior-year profit, and YTD net income at the end of `as_of`. |
| `profit_and_loss(period)` | `Result<ProfitAndLoss>` | Income, COGS, gross profit, expenses, other income/expense, and net income over `period`. |
| `trial_balance(as_of)` | `Result<TrialBalance>` | Non-zero accounts with signed balance, debit/credit totals, and prior-year profit closure. |
| `general_ledger(period)` | `Result<GeneralLedger<'_>>` | Account-by-account ledger activity with opening and closing balances over `period`. |
| `journal(period)` | `Result<Journal<'_>>` | Chronological debit/credit journal entries over `period`. |
| `account_listing(as_of)` | `Result<AccountListing>` | Chart of accounts paired with computed balances at `as_of`. |
| `name_balances(side, as_of)` | `Result<NameBalances>` | Balances owed by customers (`Side::Receivable`) or to vendors (`Side::Payable`). |
| `name_ledger(name_id, side, period)` | `Result<Option<NameLedger>>` | Running subledger activity for one customer or vendor over `period`. |

---

## Open Items and Aging (`qbw::open_items`, `qbw::aging`)

```rust
use qbw::aging::AgingOptions;
use qbw::{Company, Side};

let company = Company::open("company.qbw")?;
let day: qbw::chrono::NaiveDate = "2024-02-15".parse()?;
let aging = company.aging(Side::Receivable, &AgingOptions::new(day))?;
println!("Total open AR: {}", aging.totals.total);
```

| Method | Returns | Description |
| --- | --- | --- |
| `open_items(side, as_of)` | `Result<OpenItems>` | Unsettled transactions on `side` at `AsOf::Latest` (stored) or a historical date (computed from links). |
| `owed_by(customer_id, as_of)` | `Result<Option<Owed>>` | Open items and overdue balance for one customer (including jobs). |
| `owed_to(vendor_id, as_of)` | `Result<Option<Owed>>` | Open items and overdue balance for one vendor. |
| `names_owed(side, as_of)` | `Result<NamesOwed>` | Open and overdue totals for every customer or vendor in one pass. |
| `aging(side, &options)` | `Result<Aging>` | Open items grouped into configurable age buckets by name or by item (`AgingBy`). |

---

## Overviews (`qbw::overviews`)

| Method | Returns | Description |
| --- | --- | --- |
| `overview(as_of, rows)` | `Result<CompanyOverview>` | Cash balances, receivables, payables, and recent transactions summary. |
| `account_overview(id, as_of, rows)` | `Result<Option<AccountOverview<'_>>>` | Account metadata, current balance, and latest register rows. |
| `name_overview(id, as_of, rows)` | `Result<Option<NameOverview<'_>>>` | Customer, vendor, employee, or other name metadata, jobs, open balance, and recent transactions. |
| `item_overview(id, rows)` | `Result<Option<ItemOverview<'_>>>` | Item metadata, current stock, and recent transaction lines. |

---

## Balance Cross-Checks (`qbw::check`)

`company.check()` verifies double-entry invariants and compares computed balances, open items, and stock quantities against the summary totals stored by QuickBooks.

```rust
use qbw::check::CheckKind;
use qbw::Company;

let company = Company::open("company.qbw")?;
let check = company.check()?;
if let Some(cmp) = check.get(CheckKind::AccountBalance) {
    println!("Account balances agreed: {}/{}", cmp.agreed, cmp.looked_at);
}
for finding in check.unexplained() {
    println!("{:?} on {:?}: {:?}", finding.what, finding.subject, finding.causes);
}
```

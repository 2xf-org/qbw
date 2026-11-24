[Docs](../README.md) / [Python](README.md) / Reports, Aging & Checks

# Reports, Aging & Checks

Compute Balance Sheets, Profit and Loss statements, Trial Balances, open receivables and payables, aging summaries, and balance cross-checks.

## Financial Statements

```python
from datetime import date
import qbw

with qbw.Company.open("company.qbw") as company:
    day = date(2024, 6, 30)
    sheet = company.balance_sheet(day)
    fy = company.fiscal_year(day)
    pnl = company.profit_and_loss(fy.start, day)
    tb = company.trial_balance(day)

    print("Assets:", sheet.assets.total.amount)
    print("Liabilities + Equity:", sheet.liabilities_and_equity.amount)
    print("Net income:", pnl.net_income.amount)
    print("Trial balance:", tb.debits.amount, tb.credits.amount)
```

### Methods on `Company`

| Method | Returns | Description |
| --- | --- | --- |
| `balance_sheet(as_of=None)` | `BalanceSheet` | Assets, liabilities, and equity at the end of `as_of` (or latest transaction date if `None`). |
| `profit_and_loss(start=None, end=None)` | `ProfitAndLoss` | Income, COGS, expenses, and net income for `start..=end` (unbounded ends if `None`). |
| `trial_balance(as_of=None)` | `TrialBalance` | Every non-zero account balance with debit and credit totals at `as_of`. |
| `open_items(side, as_of=None)` | `OpenItems` | Unsettled invoices and credits (`Side.Receivable`) or bills and credits (`Side.Payable`). |
| `aging(side, as_of=None, basis=None, buckets=3, days=30)` | `Aging` | AR or AP aging summary by customer or vendor into `buckets` intervals of `days` days. |
| `check()` | `CheckReport` | Compare computed ledger, open-item, and inventory totals against summary figures stored by QuickBooks. |

---

## Balance Sheet

Amounts in Balance Sheet sections are positive on each section's normal side: assets, liabilities, and equity all read positive when in their normal direction.

### `BalanceSheet`

| Field | Type | Description |
| --- | --- | --- |
| `as_of` | `datetime.date` | Effective date of the statement. |
| `assets` | `Assets` | Asset sections and totals. |
| `liabilities` | `Liabilities` | Liability sections and totals. |
| `equity` | `Equity` | Equity accounts, prior-year profit, fiscal YTD net income, and total. |
| `liabilities_and_equity` | `Money` | Combined total of liabilities and equity (equals `assets.total` when `unplaced` and `left_out` are zero). |
| `unplaced` | `Money` | Balance posted to accounts outside standard statement sections (debit positive). |
| `left_out` | `LeftOut` | Skipped rows and pages if the file has damage. |

### `Assets`

| Field | Type | Description |
| --- | --- | --- |
| `bank` | `ReportSection` | Bank accounts. |
| `accounts_receivable` | `ReportSection` | Accounts receivable accounts. |
| `other_current_assets` | `ReportSection` | Other current asset accounts. |
| `current_assets` | `Money` | Sum of bank, accounts receivable, and other current assets. |
| `fixed_assets` | `ReportSection` | Fixed asset accounts. |
| `other_assets` | `ReportSection` | Other asset accounts. |
| `total` | `Money` | Total assets. |

### `Liabilities`

| Field | Type | Description |
| --- | --- | --- |
| `accounts_payable` | `ReportSection` | Accounts payable accounts. |
| `credit_cards` | `ReportSection` | Credit card accounts. |
| `other_current_liabilities` | `ReportSection` | Other current liability accounts. |
| `current_liabilities` | `Money` | Sum of accounts payable, credit cards, and other current liabilities. |
| `long_term_liabilities` | `ReportSection` | Long-term liability accounts. |
| `total` | `Money` | Total liabilities. |

### `Equity`

| Field | Type | Description |
| --- | --- | --- |
| `accounts` | `ReportSection` | Equity accounts (Retained Earnings includes prior-year profit). |
| `earlier_profit` | `Money` | Cumulative profit of all fiscal years before the balance sheet's year. |
| `retained_earnings_account_id` | `int \| None` | Retained Earnings account ID when `earlier_profit` is rolled into it. |
| `net_income` | `Money` | Fiscal year-to-date net income. |
| `total` | `Money` | Total equity. |

### `ReportSection`

| Field | Type | Description |
| --- | --- | --- |
| `account_type` | `str` | Account type label for this section. |
| `accounts` | `list[SectionAccount]` | Top-level accounts in list order. |
| `total` | `Money` | Section total in the home currency. |

### `SectionAccount`

| Field | Type | Description |
| --- | --- | --- |
| `account_id` | `int` | Account ID. |
| `account_name` | `str \| None` | Short account name. |
| `account_full_name` | `str \| None` | Full hierarchical account name. |
| `account_number` | `str \| None` | Account number, if assigned. |
| `amount` | `Money` | Amount posted directly to this account. |
| `total` | `Money` | Rollup total of this account and all sub-accounts. |
| `children` | `list[SectionAccount]` | Nested child sub-accounts. |

---

## `ProfitAndLoss`

| Field | Type | Description |
| --- | --- | --- |
| `start` | `datetime.date \| None` | Inclusive start date, or `None` if unbounded. |
| `end` | `datetime.date \| None` | Inclusive end date, or `None` if unbounded. |
| `income` | `ReportSection` | Operating income accounts and total. |
| `cost_of_goods_sold` | `ReportSection` | Cost of goods sold accounts and total. |
| `gross_profit` | `Money` | `income.total - cost_of_goods_sold.total`. |
| `expense` | `ReportSection` | Operating expense accounts and total. |
| `net_ordinary_income` | `Money` | `gross_profit - expense.total`. |
| `other_income` | `ReportSection` | Non-operating income accounts and total. |
| `other_expense` | `ReportSection` | Non-operating expense accounts and total. |
| `net_other_income` | `Money` | `other_income.total - other_expense.total`. |
| `net_income` | `Money` | `net_ordinary_income + net_other_income`. |
| `unplaced` | `Money` | Amount posted to accounts outside standard sections. |
| `left_out` | `LeftOut` | Skipped rows and pages if the file has damage. |

---

## `TrialBalance` and `TrialBalanceRow`

### `TrialBalance`

| Field | Type | Description |
| --- | --- | --- |
| `as_of` | `datetime.date` | Effective date of the trial balance. |
| `rows` | `list[TrialBalanceRow]` | Non-zero accounts in chart-of-accounts order. |
| `debits` | `Money` | Sum of all debit balances. |
| `credits` | `Money` | Sum of all credit balances (equals `debits` in balanced books). |
| `left_out` | `LeftOut` | Skipped rows and pages if the file has damage. |

### `TrialBalanceRow`

| Field | Type | Description |
| --- | --- | --- |
| `account_id` | `int \| None` | Account ID (`None` for standalone prior-year profit if no Retained Earnings account exists). |
| `account_name` | `str \| None` | Full account name. |
| `account_type` | `str \| None` | Account type label. |
| `balance` | `Money` | Signed balance in home currency (debit positive). |
| `debit` | `Money \| None` | Debit column amount, if debit balance. |
| `credit` | `Money \| None` | Credit column amount (positive), if credit balance. |

---

## Open Items and Aging

```python
from datetime import date
import qbw

with qbw.Company.open("company.qbw") as company:
    ar = company.open_items(qbw.Side.Receivable)
    print("Open AR:", ar.home_total.amount, "items:", len(ar.items))

    aging = company.aging(qbw.Side.Receivable, date(2024, 2, 15))
    for bucket, total in zip(aging.buckets, aging.totals, strict=True):
        print(f"{bucket.label:<10} {total.amount}")
```

### `OpenItems`

| Field | Type | Description |
| --- | --- | --- |
| `side` | `Side` | `Side.Receivable` or `Side.Payable`. |
| `as_of` | `datetime.date \| None` | Evaluation date (`None` for latest stored open state). |
| `items` | `list[OpenItem]` | Unsettled transactions in date order. |
| `home_total` | `Money` | Home-currency sum of all open items. |
| `adjustments` | `Money` | Exchange-rate adjustment not attached to individual items. |
| `left_out` | `LeftOut` | Skipped rows and pages if the file has damage. |

### `OpenItem`

| Field | Type | Description |
| --- | --- | --- |
| `transaction_id` | `int` | Transaction ID. |
| `kind` | `str` | Transaction kind label. |
| `date` | `datetime.date` | Transaction date. |
| `number` | `str \| None` | Document number. |
| `name_id` | `int \| None` | Customer or vendor ID. |
| `name` | `str \| None` | Customer or vendor full name. |
| `account_id` | `int` | AR or AP account ID. |
| `account_name` | `str \| None` | AR or AP full account name. |
| `due_date` | `datetime.date \| None` | Due date, if applicable. |
| `total` | `Money` | Original total in the account's currency. |
| `open` | `Money` | Remaining open amount in the account's currency. |
| `home_open` | `Money` | Remaining open amount in the home currency. |
| `origin` | `Origin` | Underlying row location. |

### `Aging`

| Field | Type | Description |
| --- | --- | --- |
| `side` | `Side` | `Side.Receivable` or `Side.Payable`. |
| `day` | `datetime.date` | Reference date ages are measured to. |
| `basis` | `AgingBasis` | `AgingBasis.DueDate` or `AgingBasis.TransactionDate`. |
| `buckets` | `list[AgingBucket]` | Ordered age buckets. |
| `rows` | `list[AgingRow]` | One row per customer or vendor. |
| `totals` | `list[Money]` | Home-currency column totals matching `buckets`. |
| `total` | `Money` | Grand total across all buckets in home currency. |
| `adjustments` | `Money` | Home-currency exchange-rate adjustments. |
| `left_out` | `LeftOut` | Skipped rows and pages if the file has damage. |

### `AgingBucket`

| Field | Type | Description |
| --- | --- | --- |
| `label` | `str` | Display label (for example `"Current"`, `"1-30"`, `">90"`). |
| `first_day` | `int \| None` | Inclusive lower bound in days (`None` for the Current bucket). |
| `last_day` | `int \| None` | Inclusive upper bound in days (`None` for the oldest bucket). |

### `AgingRow`

| Field | Type | Description |
| --- | --- | --- |
| `name_id` | `int \| None` | Customer or vendor ID. |
| `name` | `str \| None` | Customer or vendor full name. |
| `amounts` | `list[Money]` | Bucket amounts in the customer or vendor's trading currency. |
| `total` | `Money` | Total open amount in the trading currency. |
| `home_amounts` | `list[Money]` | Bucket amounts in the home currency. |
| `home_total` | `Money` | Total open amount in the home currency. |

---

## Balance Cross-Checks

`company.check()` reads the file and compares computed account balances, open items, and inventory stock quantities against the summary totals stored by QuickBooks.

```python
import qbw

with qbw.Company.open("company.qbw") as company:
    report = company.check()
    for cmp in report.compared:
        print(f"{cmp.kind}: {cmp.agreed}/{cmp.looked_at} agreed")
    for finding in report.findings:
        if not finding.is_explained:
            print(finding.kind, finding.subject, finding.computed, finding.expected)
```

### `CheckReport`

| Field | Type | Description |
| --- | --- | --- |
| `compared` | `list[CheckComparison]` | Per-category agreement counts. |
| `findings` | `list[CheckFinding]` | Every discrepancy found. |
| `unexplained_count` | `int` | Count of findings whose known causes do not explain the full difference. |
| `not_compared` | `list[CheckNotCompared]` | Categories skipped because an optional table was absent. |
| `left_out` | `LeftOut` | Skipped rows and pages if the file has damage. |

### `CheckComparison`

| Field | Type | Description |
| --- | --- | --- |
| `kind` | `str` | Category label. |
| `looked_at` | `int` | Number of records or figures compared. |
| `agreed` | `int` | Number that matched. |

### `CheckFinding`

| Field | Type | Description |
| --- | --- | --- |
| `kind` | `str` | Category label. |
| `subject` | `str` | Record or scope where the figures differed. |
| `computed` | `str \| None` | Computed figure, formatted as text. |
| `expected` | `str \| None` | Stored or expected figure, formatted as text. |
| `is_explained` | `bool` | Whether known causes account for the full difference. |
| `causes` | `list[str]` | Known or possible explanations for the discrepancy. |

### `CheckNotCompared`

| Field | Type | Description |
| --- | --- | --- |
| `kind` | `str` | Category label. |
| `table` | `str` | Missing optional table name. |

[Docs](../README.md) / [Python](README.md) / Transactions, Registers & Links

# Transactions, Registers & Links

Search transactions, inspect headers and line items, walk account registers with running balances, and follow payment or order links.

## Searching and Reading Transactions

```python
import qbw

with qbw.Company.open("company.qbw") as company:
    query = qbw.TransactionFilter(
        kinds=["Invoice"],
        paid=qbw.PaidState.Unpaid,
        order=qbw.TransactionOrder.DueDate,
        direction=qbw.SortDirection.Ascending,
    )
    page = company.transactions(query, offset=0, limit=50)
    for summary in page.rows:
        tx = company.transaction(summary.id)
        if tx is not None:
            print(tx.number, tx.total.amount if tx.total else None, len(tx.lines))
```

### Methods on `Company`

| Method | Returns | Description |
| --- | --- | --- |
| `transaction(transaction_id)` | `Transaction \| None` | Read a full transaction header and all its lines. |
| `transaction_summary(transaction_id)` | `TransactionSummary \| None` | Read header metadata without loading lines. |
| `find(kind, number)` | `int \| None` | Find the latest transaction ID matching `kind` (for example `"Invoice"`) and document `number`. |
| `find_all(kind, number)` | `list[int]` | Find every transaction ID matching `kind` and `number`. |
| `transactions(filter=None, offset=0, limit=100)` | `TransactionPage` | Paginated, filtered, and sorted transaction summaries with per-kind counts. |
| `register(account_id, start=None, end=None, offset=0, limit=100)` | `Register \| None` | Paginated account register with opening, running, and closing balances (`None` if no such account). |
| `links(transaction_id)` | `TransactionLinks \| None` | All links (payments, credits, deposits, order fulfillments) attached to `transaction_id`. |
| `paid_by(transaction_id)` | `TransactionLinks \| None` | Payments, credits, and discounts that settled `transaction_id`, with total settled. |
| `applied_to(transaction_id)` | `TransactionLinks \| None` | Invoices or bills that `transaction_id` paid or credited, with total applied. |

---

## `TransactionFilter`

All fields are optional.

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `kinds` | `list[str] \| None` | `None` | Restrict to transaction kind names (for example `["Invoice", "Bill"]`). |
| `start` | `datetime.date \| None` | `None` | Inclusive earliest transaction date. |
| `end` | `datetime.date \| None` | `None` | Inclusive latest transaction date. |
| `name_id` | `int \| None` | `None` | Restrict to transactions with this customer, vendor, employee, or other name ID. |
| `with_jobs` | `bool` | `False` | When `name_id` is a customer, also include transactions for its jobs. |
| `account_id` | `int \| None` | `None` | Restrict to transactions whose header account (such as bank, AR, or AP) is this account. |
| `posts_to_account_id` | `int \| None` | `None` | Restrict to transactions that post any line to this account. |
| `number` | `str \| None` | `None` | Case-insensitive match on document number. |
| `paid` | `PaidState \| None` | `None` | `PaidState.Paid` or `PaidState.Unpaid`. |
| `text` | `str \| None` | `None` | Case-insensitive search across number, name, memo, and line descriptions. |
| `order` | `TransactionOrder \| None` | `None` | `Date`, `Entered`, `Number`, `Name`, `Total`, or `DueDate` (defaults to `Date`). |
| `direction` | `SortDirection \| None` | `None` | `SortDirection.Ascending` or `SortDirection.Descending` (defaults to `Descending`). |

---

## `Transaction` and `TransactionLine`

On forms with a header account (such as the bank account on a check, AR on an invoice, or AP on a bill), `Transaction.account_id` and `Transaction.account_name` hold that account and `Transaction.lines` hold the detail lines. On journal entries, the first line is marked with `is_own = True`.

### `Transaction`

| Field | Type | Description |
| --- | --- | --- |
| `id` | `int` | Transaction ID. |
| `kind` | `str` | Transaction kind label (for example `"Invoice"`, `"Check"`, `"Bill"`). |
| `kind_code` | `int` | Raw numeric transaction kind code. |
| `date` | `datetime.date` | Transaction date. |
| `number` | `str \| None` | Document number shown on the form. |
| `name_id` | `int \| None` | Customer, vendor, employee, or other name ID. |
| `name` | `str \| None` | Full name. |
| `account_id` | `int \| None` | Header account ID. |
| `account_name` | `str \| None` | Header account full name. |
| `memo` | `str \| None` | Transaction memo. |
| `currency_id` | `int` | Currency ID. |
| `currency_code` | `str \| None` | ISO 4217 currency code. |
| `exchange_rate` | `float` | Stored exchange rate to home currency. |
| `total` | `Money \| None` | Form total in the transaction currency. |
| `home_total` | `Money \| None` | Form total in the home currency. |
| `subtotal` | `Money \| None` | Subtotal before sales tax on sales forms. |
| `tax_total` | `Money \| None` | Sales tax total on sales forms. |
| `due_date` | `datetime.date \| None` | Due date, if applicable. |
| `open` | `Money \| None` | Remaining open amount, if trackable. |
| `is_paid` | `bool \| None` | Whether fully paid or applied (`None` when not trackable). |
| `is_voided` | `bool` | Whether the transaction was voided. |
| `posting` | `PostingState` | `PostingState.Posted` or `PostingState.NonPosting`. |
| `terms_name` | `str \| None` | Payment terms label recorded on the form. |
| `sales_rep_initials` | `str \| None` | Sales rep initials recorded on the form. |
| `lines` | `list[TransactionLine]` | Lines in form order. |
| `damage` | `list[DamageInfo]` | Damaged lines or references encountered while reading this form. |
| `origin` | `Origin` | Underlying header row location. |

### `TransactionLine`

| Field | Type | Description |
| --- | --- | --- |
| `id` | `int` | Line ID. |
| `is_own` | `bool` | Whether this line is also the transaction's header side (such as line 1 of a journal entry). |
| `role` | `str` | Line role (for example `"Item"`, `"Account"`, `"Sales Tax"`). |
| `item_id` | `int \| None` | Item ID. |
| `item_name` | `str \| None` | Full item name. |
| `account_id` | `int \| None` | Account ID on the line. |
| `account_name` | `str \| None` | Full account name. |
| `name_id` | `int \| None` | Line-level name ID. |
| `name` | `str \| None` | Line-level full name. |
| `class_id` | `int \| None` | Class ID. |
| `class_name` | `str \| None` | Full class name. |
| `description` | `str \| None` | Line description or memo. |
| `quantity` | `str \| None` | Exact quantity string. |
| `rate` | `Price \| None` | Unit price or percentage rate. |
| `amount` | `Money \| None` | Amount in the transaction currency. |
| `home_amount` | `Money \| None` | Amount in the home currency. |
| `debit` | `Money \| None` | Journal entry debit amount, when applicable. |
| `credit` | `Money \| None` | Journal entry credit amount, when applicable. |
| `service_date` | `datetime.date \| None` | Service date. |
| `cleared` | `Cleared` | `NotCleared`, `Cleared`, `Reconciled`, or `Matched`. |
| `reconciled_on` | `datetime.date \| None` | Reconciliation statement date. |
| `origin` | `Origin` | Underlying line row location. |

---

## `TransactionPage`, `KindCount`, and `TransactionSummary`

### `TransactionPage`

| Field | Type | Description |
| --- | --- | --- |
| `rows` | `list[TransactionSummary]` | Summaries on the requested page. |
| `total_count` | `int` | Total matching transactions across all pages. |
| `kinds` | `list[KindCount]` | Matching count breakdown by transaction kind. |
| `offset` | `int` | Zero-based starting offset. |
| `limit` | `int` | Requested page size. |
| `left_out` | `LeftOut` | Skipped rows and pages if the file has damage. |

### `KindCount`

| Field | Type | Description |
| --- | --- | --- |
| `kind` | `str` | Transaction kind label. |
| `kind_code` | `int` | Raw numeric transaction kind code. |
| `count` | `int` | Number of matching transactions of this kind. |

### `TransactionSummary`

| Field | Type | Description |
| --- | --- | --- |
| `id` | `int` | Transaction ID. |
| `kind` | `str` | Transaction kind label. |
| `kind_code` | `int` | Raw numeric transaction kind code. |
| `date` | `datetime.date` | Transaction date. |
| `number` | `str \| None` | Document number. |
| `name_id` | `int \| None` | Customer, vendor, employee, or other name ID. |
| `name` | `str \| None` | Full name. |
| `account_id` | `int \| None` | Header account ID. |
| `account_name` | `str \| None` | Header account full name. |
| `other_account_id` | `int \| None` | Destination account ID on transfers. |
| `other_account_name` | `str \| None` | Destination account full name on transfers. |
| `memo` | `str \| None` | Transaction memo. |
| `currency_id` | `int` | Currency ID. |
| `currency_code` | `str \| None` | ISO 4217 currency code. |
| `amount` | `Money \| None` | Header account posting amount (debit positive). |
| `total` | `Money \| None` | Form total in the transaction currency. |
| `home_total` | `Money \| None` | Form total in the home currency. |
| `due_date` | `datetime.date \| None` | Due date. |
| `open` | `Money \| None` | Remaining open amount. |
| `is_paid` | `bool \| None` | Whether fully paid or applied. |
| `is_voided` | `bool` | Whether voided. |
| `posting` | `PostingState` | `PostingState.Posted` or `PostingState.NonPosting`. |
| `origin` | `Origin` | Header row location. |
| `left_out` | `LeftOut` | Skipped rows and pages if the file has damage. |

---

## Account Registers

```python
from datetime import date
import qbw

with qbw.Company.open("company.qbw") as company:
    checking = company.account_named("Checking")
    if checking is not None:
        reg = company.register(checking.id, start=date(2024, 1, 1), end=date(2024, 6, 30))
        if reg is not None:
            print("Opening:", reg.opening_balance.amount, "Closing:", reg.closing_balance.amount)
            for row in reg.rows:
                print(row.date, row.kind, row.amount.amount, row.balance.amount)
```

### `Register`

| Field | Type | Description |
| --- | --- | --- |
| `account_id` | `int` | Account ID. |
| `account_name` | `str \| None` | Full account name. |
| `opening_balance` | `Money` | Balance before the period in the account's currency (debit positive). |
| `closing_balance` | `Money` | Balance after the period in the account's currency (debit positive). |
| `rows` | `list[RegisterRow]` | Register rows on the requested page. |
| `total_count` | `int` | Total rows in the period across all pages. |
| `left_out` | `LeftOut` | Skipped rows and pages if the file has damage. |

### `RegisterRow`

| Field | Type | Description |
| --- | --- | --- |
| `date` | `datetime.date` | Transaction date. |
| `kind` | `str` | Transaction kind label. |
| `number` | `str \| None` | Document number. |
| `name_id` | `int \| None` | Name ID. |
| `name` | `str \| None` | Full name. |
| `other_side` | `str \| None` | Counterparty account name, `"Split"`, or `None`. |
| `memo` | `str \| None` | Line or header memo. |
| `cleared` | `Cleared` | Reconciliation status. |
| `reconciled_on` | `datetime.date \| None` | Reconciliation statement date. |
| `amount` | `Money` | Posting amount in the account's currency (debit positive). |
| `balance` | `Money` | Running account balance after this row (debit positive). |
| `transaction_id` | `int` | Transaction ID. |
| `line_id` | `int` | Line ID. |
| `origin` | `Origin` | Underlying row location. |

---

## Settlement and Order Links

```python
import qbw

with qbw.Company.open("company.qbw") as company:
    tx_id = company.find("Invoice", "1001")
    if tx_id is not None:
        settled = company.paid_by(tx_id)
        if settled is not None:
            for link in settled.links:
                print(link.date, link.relation, link.kind, link.amount.amount if link.amount else None)
```

### `TransactionLinks`

| Field | Type | Description |
| --- | --- | --- |
| `transaction_id` | `int` | Source transaction ID. |
| `currency_id` | `int` | Currency ID of link amounts. |
| `currency_code` | `str \| None` | ISO 4217 currency code. |
| `links` | `list[TransactionLink]` | Linked transactions in order. |
| `total` | `Money \| None` | Sum of settlement amounts (populated by `paid_by()` and `applied_to()`). |
| `left_out` | `LeftOut` | Skipped rows and pages if the file has damage. |

### `TransactionLink`

| Field | Type | Description |
| --- | --- | --- |
| `transaction_id` | `int` | Linked transaction ID. |
| `kind` | `str` | Linked transaction kind label. |
| `date` | `datetime.date` | Linked transaction date. |
| `number` | `str \| None` | Linked document number. |
| `name_id` | `int \| None` | Linked name ID. |
| `name` | `str \| None` | Linked full name. |
| `relation` | `str` | Relationship label (for example `"paid by"`, `"applied to"`, `"deposited in"`). |
| `amount` | `Money \| None` | Linked or settled amount. |
| `quantity` | `str \| None` | Linked quantity on order fulfillment links. |

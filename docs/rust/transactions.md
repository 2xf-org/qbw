[Docs](../README.md) / [Rust](README.md) / Transactions, Ledger & Links

# Transactions, Ledger & Links

Query transactions, read full forms and lines, scan the general ledger, build account registers, and follow payment or order links.

## Querying and Reading Transactions

```rust
use qbw::query::{Direction, PaidState, TransactionOrder, TransactionQuery};
use qbw::transactions::TransactionKind;
use qbw::{Company, Page};

let company = Company::open("company.qbw")?;
let query = TransactionQuery::new()
    .kinds([TransactionKind::Invoice])
    .paid(PaidState::Unpaid)
    .order(TransactionOrder::DueDate, Direction::Ascending);

let page = company.transactions(&query, Page::new(0, 50))?;
for summary in &page.rows {
    if let Some(tx) = company.transaction(summary.id)? {
        println!("{:?} lines={}", tx.number, tx.form_lines().count());
    }
}
```

### Transaction Methods on `Company`

| Method | Returns | Description |
| --- | --- | --- |
| `transaction(id)` | `Result<Option<Transaction>>` | Read the full header, form-specific `Details`, and ordered `Vec<Line>`. |
| `transaction_summary(id)` | `Result<Option<TransactionSummary>>` | Read header summary metadata without loading all lines. |
| `transactions(&query, page)` | `Result<TransactionPage>` | Filtered, sorted, paginated summaries with per-kind counts. |
| `find(kind, number)` | `Result<Option<TransactionId>>` | Find the latest transaction matching `kind` and document `number`. |
| `find_all(kind, number)` | `Result<Vec<TransactionId>>` | Find every transaction matching `kind` and `number`. |

### `TransactionQuery` Builders

| Method | Description |
| --- | --- |
| `TransactionQuery::new()` | All listed transactions, newest first. |
| `.kinds(iter)` | Restrict to specific `TransactionKind` values. |
| `.dated(period)` | Restrict to dates within `Period`. |
| `.name(name_id)` | Restrict to transactions with `name_id`. |
| `.with_jobs()` | Include jobs of `.name(name_id)`. |
| `.account(account_id)` | Restrict by header account (such as bank, AR, or AP). |
| `.posts_to(account_id)` | Restrict to transactions with any line posting to `account_id`. |
| `.amount(range)` | Restrict by unsigned amount range (`RangeBounds<Decimal>`). |
| `.number(text)` | Case-insensitive match on document number. |
| `.paid(state)` | Filter by `PaidState::Paid` or `PaidState::Unpaid`. |
| `.text(query)` | Case-insensitive substring match across number, memo, and name. |
| `.posting(filter)` | Select `PostingFilter::Listed`, `Posted`, `NonPosting`, or `All`. |
| `.order(order, direction)` | Set `TransactionOrder` and `Direction`. |

---

## General Ledger and Account Registers

`company.ledger()` builds and caches every double-entry posting in the company file (debits positive, credits negative).

```rust
use qbw::{Company, Page, Period};

let company = Company::open("company.qbw")?;
let checking = company.accounts()?.named("Checking").ok_or("no such account")?;
let register = company
    .register(checking.id, Period::all(), Page::new(0, 100))?
    .ok_or("no such account")?;

for row in &register.rows {
    println!("{}  {:>12}  {:>12}", row.date, row.amount, row.balance);
}
```

### Methods on `Ledger`

| Method | Returns | Description |
| --- | --- | --- |
| `postings()` | `Postings<'_>` | Iterator over all postings in account, date, and line order. |
| `of_account(account_id)` | `Postings<'_>` | All postings to one account (excluding sub-accounts). |
| `during(account_id, period)` | `Postings<'_>` | Postings to one account within `Period`. |
| `of_transaction(tx_id)` | `impl Iterator<Item = Posting>` | Postings produced by one transaction in form line order. |
| `balance(account_id, as_of)` | `Option<Balance>` | Account balance in native and home currencies through `AsOf`. |
| `left_out()` | `LeftOut` | Count of unreadable rows skipped while building the ledger. |

### Register Methods on `Company`

| Method | Returns | Description |
| --- | --- | --- |
| `register(account_id, period, page)` | `Result<Option<Register>>` | Opening balance, paginated `RegisterRow` entries with running balances, and closing balance. |
| `posting_texts(postings)` | `Result<PostingTexts>` | Look up header numbers and line memos for a slice of `Posting` values. |

---

## Settlement and Order Links (`qbw::links`)

| Method | Returns | Description |
| --- | --- | --- |
| `links(tx_id)` | `Result<Option<Links>>` | Every linked transaction with its `Relation` (payment, credit, discount, deposit, refund, or order). |
| `paid_by(tx_id)` | `Result<Option<Settled>>` | Payments, credits, and discounts that settled `tx_id`, plus `total`. |
| `applied_to(tx_id)` | `Result<Option<Settled>>` | Invoices or bills settled by `tx_id`, plus `total`. |
| `line_links(tx_id)` | `Result<Option<LineLinks>>` | Line-to-line links (such as purchase order lines linked to bill or receipt lines). |

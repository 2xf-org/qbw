[Docs](../README.md) / [Rust](README.md) / Accounts, Lists & Stock

# Accounts, Lists & Stock

Read hierarchical lists (`List<T>`), the chart of accounts, names (customers, vendors, employees, other names), items, inventory stock, historical valuation, open orders, and supporting lists.

## `List<T>`

Accounts, items, currencies, terms, classes, and supporting lists are returned as `&List<T>`.

| Method | Returns | Description |
| --- | --- | --- |
| `get(id)` | `Option<&T>` | Look up an entry by typed ID. |
| `named(name)` | `Option<&T>` | Case-insensitive lookup by full (`Parent:Child`) or unique short name. |
| `all_named(name)` | `Vec<&T>` | Every entry matching `name` (full or short name, case-insensitive). |
| `full_name(id)` | `Option<&str>` | Full hierarchical `Parent:Child` name. |
| `own_name(id)` | `Option<&str>` | Short name without parent prefix. |
| `iter()` | `slice::Iter<'_, T>` | All entries in QuickBooks list order, including inactive entries. |
| `active()` | `impl Iterator<Item = &T>` | Active entries in list order. |
| `walk()` | `impl Iterator<Item = (usize, &T)>` | Depth-first tree traversal yielding `(depth, entry)`. |
| `left_out()` | `LeftOut` | Count of damaged rows skipped while reading this list. |
| `damage()` | `&[Damage]` | Detailed damage records encountered while reading. |

---

## Chart of Accounts (`qbw::accounts`)

```rust
use qbw::Company;

let company = Company::open("company.qbw")?;
let accounts = company.accounts()?;
for (depth, account) in accounts.walk() {
    if account.is_active {
        let indent = "  ".repeat(depth);
        println!("{indent}{} ({})", account.name, account.account_type);
    }
}
```

| Item | Description |
| --- | --- |
| `company.accounts()` | Returns `Result<&List<Account>>`. |
| `Account` | Account ID, parent link, `account_type`, number, currency, active flag, `stored` balance, `reconciling` statement info, and `origin`. |
| `AccountType::normal_side()` | Returns `Option<NormalSide>` (`Debit` or `Credit`). |
| `AccountType::statement()` | Returns `Option<Statement>` (`BalanceSheet` or `ProfitAndLoss`). |
| `AccountType::as_kept(money)` | Flips the sign on credit-normal accounts so normal balances read positive. |

---

## Names (`qbw::names`)

`company.names()` returns `&Names`, covering customers, vendors, employees, and other names under one shared `NameId` space.

| Method | Returns | Description |
| --- | --- | --- |
| `get(id)` | `Option<Name<'_>>` | Look up a name across all four lists by `NameId`. |
| `named(name)` | `Option<Name<'_>>` | Case-insensitive lookup by full (`Customer:Job`) or unique short name. |
| `full_name(id)` | `Option<&str>` | Full hierarchical name for `id`. |
| `iter()` | `impl Iterator<Item = Name<'_>>` | Every name across customers, vendors, employees, and other names. |
| `customers()` | `&List<Customer>` | Customers and customer jobs. |
| `vendors()` | `&List<Vendor>` | Vendors. |
| `employees()` | `&List<Employee>` | Employees. |
| `other_names()` | `&List<OtherName>` | Other names. |

---

## Items, Stock, Valuation & Open Orders

```rust
use qbw::Company;

let company = Company::open("company.qbw")?;
let items = company.items()?;
let stock = company.stock()?;

for row in stock.rows.iter().filter(|row| row.needs_reorder()) {
    if let Some(item) = items.get(row.item).filter(|item| item.is_active) {
        let name = items.full_name(item.id).unwrap_or(&item.name);
        println!("{name}: {} on hand, {} on order", row.on_hand, row.on_order);
    }
}
```

| Method | Returns | Description |
| --- | --- | --- |
| `items()` | `Result<&List<Item>>` | Products, services, assemblies, groups, discounts, and sales tax items. |
| `stock()` | `Result<Stock>` | Current on-hand quantity, value, average cost, commitments, and `needs_reorder()` flags. |
| `valuation(as_of)` | `Result<Valuation>` | Historical on-hand quantity, value, and average cost at the end of `as_of`. |
| `movements(item_id)` | `Result<Option<Movements>>` | Chronological stock history entries for one stocked item. |
| `open_orders(side)` | `Result<OpenOrders>` | Open lines on sales orders (`Side::Receivable`) or purchase orders (`Side::Payable`). |
| `has_open_orders(side)` | `Result<bool>` | Check whether any open order line exists on `side`. |

---

## Supporting Lists (`qbw::lists` and `qbw::custom_fields`)

| Method | Returns |
| --- | --- |
| `currencies()` | `Result<&List<Currency>>` |
| `terms()` | `Result<&List<Terms>>` |
| `classes()` | `Result<&List<Class>>` |
| `categories()` | `Result<&List<Category>>` |
| `customer_messages()` | `Result<&List<CustomerMessage>>` |
| `customer_types()` | `Result<&List<CustomerType>>` |
| `job_types()` | `Result<&List<JobType>>` |
| `vendor_types()` | `Result<&List<VendorType>>` |
| `payment_methods()` | `Result<&List<PaymentMethod>>` |
| `price_levels()` | `Result<&List<PriceLevel>>` |
| `sales_reps()` | `Result<&List<SalesRep>>` |
| `sales_tax_codes()` | `Result<&List<SalesTaxCode>>` |
| `ship_methods()` | `Result<&List<ShipMethod>>` |
| `sites()` | `Result<&List<Site>>` |
| `templates()` | `Result<&List<Template>>` |
| `memorized()` | `Result<&List<Memorized>>` |
| `custom_fields()` | `Result<&CustomFields>` |

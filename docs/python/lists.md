[Docs](../README.md) / [Python](README.md) / Accounts, Lists & Stock

# Accounts, Lists & Stock

Read the chart of accounts, customers, vendors, employees, other names, items, inventory stock, historical valuation, open orders, and supporting lists.

## Chart of Accounts

```python
from datetime import date
import qbw

with qbw.Company.open("company.qbw") as company:
    parts = company.account_named("Sales:Parts")
    if parts is not None:
        bal = company.account_balance(parts.id, date(2024, 6, 30))
        print(parts.full_name, parts.account_type, bal.amount.amount if bal else None)
```

### Methods on `Company`

| Method | Returns | Description |
| --- | --- | --- |
| `accounts(active_only=False)` | `list[Account]` | All accounts in tree order. Pass `active_only=True` to skip inactive accounts. |
| `account(account_id)` | `Account \| None` | Look up an account by numeric ID. |
| `account_named(name)` | `Account \| None` | Look up an account by full (`Parent:Child`) or unique short name (case-insensitive). |
| `account_balance(account_id, as_of=None)` | `AccountBalance \| None` | Computed ledger balance through `as_of` (or latest state if `None`). Debits are positive. |

### `Account`

| Field | Type | Description |
| --- | --- | --- |
| `id` | `int` | Account ID. |
| `name` | `str` | Short name without parent prefix. |
| `full_name` | `str` | Full hierarchical name separated by `:`. |
| `parent_id` | `int \| None` | Parent account ID, if this is a sub-account. |
| `depth` | `int` | Nesting depth (`0` for top-level accounts). |
| `account_type` | `str` | Account type label (for example `"Bank"`, `"Income"`). |
| `account_type_code` | `int` | Raw numeric account type code stored in the file. |
| `normal_side` | `NormalSide \| None` | `NormalSide.Debit` or `NormalSide.Credit` (`None` for non-posting accounts). |
| `statement` | `Statement \| None` | `Statement.BalanceSheet` or `Statement.ProfitAndLoss`. |
| `special` | `str \| None` | Built-in QuickBooks account role, if any. |
| `number` | `str \| None` | Account number, if assigned. |
| `description` | `str \| None` | Account description. |
| `currency_id` | `int` | Currency ID. |
| `currency_code` | `str \| None` | ISO 4217 currency code, if set. |
| `tax_line` | `int \| None` | Income tax form line number, if mapped. |
| `is_active` | `bool` | Whether the account is active. |
| `stored` | `StoredBalance \| None` | Balance stored on the account row by QuickBooks. |
| `origin` | `Origin` | Underlying database row location. |

### `AccountBalance`

| Field | Type | Description |
| --- | --- | --- |
| `account_id` | `int` | Account ID. |
| `account_name` | `str \| None` | Full account name, if listed. |
| `amount` | `Money` | Computed balance in the account's currency (debit positive). |
| `home_amount` | `Money` | Computed balance in the home currency (debit positive). |
| `left_out` | `LeftOut` | Skipped rows and pages if the file has damage. |

### `StoredBalance`

| Field | Type | Description |
| --- | --- | --- |
| `amount` | `Money` | Stored balance in the record's currency. |
| `home_amount` | `Money \| None` | Stored balance in the home currency, when recorded separately. |

---

## Names (Customers, Vendors, Employees, Other Names)

Customers, vendors, employees, and other names share a single numeric `id` space.

```python
import qbw

with qbw.Company.open("company.qbw") as company:
    for customer in company.customers(active_only=True):
        email = customer.contact.email if customer.contact else None
        print(customer.full_name, email)
```

### Methods on `Company`

| Method | Returns | Description |
| --- | --- | --- |
| `names(kind=None, active_only=False)` | `list[NameEntry]` | List all names, optionally filtered to one `NameKind` (`Customer`, `Vendor`, `Employee`, `Other`). |
| `customers(active_only=False)` | `list[NameEntry]` | Customers and customer jobs. |
| `vendors(active_only=False)` | `list[NameEntry]` | Vendors. |
| `employees(active_only=False)` | `list[NameEntry]` | Employees. |
| `other_names(active_only=False)` | `list[NameEntry]` | Other names. |
| `name(name_id)` | `NameEntry \| None` | Look up a name by shared numeric ID. |
| `name_named(name)` | `NameEntry \| None` | Look up a name by full or unique short name (case-insensitive). |

### `NameEntry`

| Field | Type | Description |
| --- | --- | --- |
| `id` | `int` | Shared ID across all four name lists. |
| `kind` | `NameKind` | `NameKind.Customer`, `NameKind.Vendor`, `NameKind.Employee`, or `NameKind.Other`. |
| `name` | `str` | Short name without parent customer prefix. |
| `full_name` | `str` | Full hierarchical name (`Customer:Job` for jobs). |
| `parent_id` | `int \| None` | Parent customer or job ID. |
| `is_active` | `bool` | Whether the entry is active. |
| `currency_id` | `int \| None` | Trading currency ID (customers and vendors). |
| `currency_code` | `str \| None` | ISO 4217 currency code, if set. |
| `terms_id` | `int \| None` | Payment terms ID. |
| `terms_name` | `str \| None` | Payment terms display name. |
| `credit_limit` | `Money \| None` | Credit limit (customers and vendors). |
| `is_1099` | `bool` | Whether vendor payments are tracked for 1099 reporting. |
| `name_on_check` | `str \| None` | Name printed on checks (vendors and employees). |
| `resale_number` | `str \| None` | Resale certificate number (customers). |
| `hired` | `datetime.date \| None` | Hire date (employees). |
| `opened` | `datetime.date \| None` | Opening date. |
| `stored` | `StoredBalance \| None` | Balance stored on the record by QuickBooks. |
| `contact` | `Contact \| None` | Primary contact details and postal address. |
| `list_id` | `str \| None` | QuickBooks ListID string. |
| `origin` | `Origin` | Underlying database row location. |

### `Contact`

| Field | Type | Description |
| --- | --- | --- |
| `id` | `str` | Contact record ID. |
| `company` | `str \| None` | Company name. |
| `salutation` | `str \| None` | Salutation prefix. |
| `first_name` | `str \| None` | First name. |
| `middle_name` | `str \| None` | Middle name or initial. |
| `last_name` | `str \| None` | Last name. |
| `job_title` | `str \| None` | Job title. |
| `contact` | `str \| None` | Primary contact name. |
| `alt_contact` | `str \| None` | Alternate contact name. |
| `phone` | `str \| None` | Primary phone number. |
| `alt_phone` | `str \| None` | Alternate phone number. |
| `mobile` | `str \| None` | Mobile phone number. |
| `fax` | `str \| None` | Fax number. |
| `email` | `str \| None` | Primary email address. |
| `cc` | `str \| None` | CC email address. |
| `account_number` | `str \| None` | Account number with this customer or vendor. |
| `address` | `Address` | Primary billing or mailing address. |

---

## Items, Stock, Valuation & Open Orders

```python
from datetime import date
import qbw

with qbw.Company.open("company.qbw") as company:
    stock = company.stock()
    for row in stock.rows:
        if row.needs_reorder:
            print(row.item_name, "on hand:", row.on_hand, "on order:", row.on_order)

    val = company.valuation(date(2024, 6, 30))
    print("Inventory value:", val.value.amount)

    open_pos = company.open_orders(qbw.Side.Payable)
    print("Open purchase order lines:", len(open_pos.lines))
```

### Methods on `Company`

| Method | Returns | Description |
| --- | --- | --- |
| `items(active_only=False)` | `list[Item]` | All products, services, assemblies, discounts, subtotals, and tax items in list order. |
| `item(item_id)` | `Item \| None` | Look up an item by numeric ID. |
| `item_named(name)` | `Item \| None` | Look up an item by full or unique short name (case-insensitive). |
| `stock()` | `Stock` | Current stock on hand, value, average cost, orders, builds, and reorder flags. |
| `valuation(as_of=None)` | `Valuation` | Inventory valuation at the end of `as_of` (or latest state if `None`). |
| `open_orders(side)` | `OpenOrders` | Open sales orders (`Side.Receivable`) or purchase orders (`Side.Payable`). |

### `Item`

| Field | Type | Description |
| --- | --- | --- |
| `id` | `int` | Item ID. |
| `name` | `str` | Short item name without parent prefix. |
| `full_name` | `str` | Full hierarchical item name separated by `:`. |
| `parent_id` | `int \| None` | Parent item ID. |
| `kind` | `str` | Item kind label (for example `"Inventory Part"`, `"Service"`). |
| `kind_code` | `int` | Raw numeric item kind code. |
| `is_stocked` | `bool` | Whether tracked in inventory stock. |
| `is_active` | `bool` | Whether the item is active. |
| `description` | `str \| None` | Sales description. |
| `price` | `Price \| None` | Sales unit price or percentage rate. |
| `income_account_id` | `int \| None` | Income account ID. |
| `income_account_name` | `str \| None` | Full income account name. |
| `purchase_description` | `str \| None` | Purchase description. |
| `cost` | `Money \| None` | Unit purchase cost. |
| `expense_account_id` | `int \| None` | Expense or COGS account ID. |
| `expense_account_name` | `str \| None` | Full expense or COGS account name. |
| `preferred_vendor_id` | `int \| None` | Preferred vendor ID. |
| `preferred_vendor_name` | `str \| None` | Preferred vendor full name. |
| `asset_account_id` | `int \| None` | Inventory asset account ID. |
| `asset_account_name` | `str \| None` | Full inventory asset account name. |
| `reorder_point` | `str \| None` | Reorder point quantity. |
| `build_point` | `str \| None` | Assembly build point quantity. |
| `part_number` | `str \| None` | Manufacturer part number. |
| `on_hand` | `str \| None` | Stored quantity on hand. |
| `stored_value` | `Money \| None` | Stored inventory value. |
| `stored_average_cost` | `Money \| None` | Stored average unit cost. |
| `list_id` | `str \| None` | QuickBooks ListID string. |
| `origin` | `Origin` | Underlying database row location. |

### `Stock`

| Field | Type | Description |
| --- | --- | --- |
| `rows` | `list[StockRow]` | One row per stocked item in list order. |
| `value` | `Money` | Total inventory value in home currency. |
| `left_out` | `LeftOut` | Skipped rows and pages if the file has damage. |

### `StockRow`

| Field | Type | Description |
| --- | --- | --- |
| `item_id` | `int` | Item ID. |
| `item_name` | `str \| None` | Full item name. |
| `on_hand` | `str` | Quantity on hand from inventory history. |
| `value` | `Money` | Total value in home currency. |
| `average_cost` | `Money \| None` | Average unit cost in home currency. |
| `counted` | `str` | Quantity summed directly from transaction lines. |
| `on_order` | `str` | Quantity remaining on open purchase orders. |
| `on_sales_orders` | `str` | Quantity committed on open sales orders. |
| `to_be_built` | `str` | Quantity added by pending assembly builds. |
| `required_for_builds` | `str` | Quantity consumed by pending assembly builds. |
| `available` | `str` | Available quantity according to company preferences. |
| `reorder_point` | `str \| None` | Reorder threshold, if set. |
| `needs_reorder` | `bool` | Whether `on_hand + on_order <= reorder_point`. |

### `Valuation`

| Field | Type | Description |
| --- | --- | --- |
| `as_of` | `datetime.date \| None` | Valuation date (`None` for latest state). |
| `rows` | `list[ValuationRow]` | Valuation rows in item list order. |
| `value` | `Money` | Total stock value in home currency. |
| `left_out` | `LeftOut` | Skipped rows and pages if the file has damage. |

### `ValuationRow`

| Field | Type | Description |
| --- | --- | --- |
| `item_id` | `int` | Item ID. |
| `item_name` | `str \| None` | Full item name. |
| `on_hand` | `str` | Quantity on hand at the end of the day. |
| `value` | `Money` | Stock value in home currency. |
| `average_cost` | `Money \| None` | Average unit cost in home currency. |

### `OpenOrders`

| Field | Type | Description |
| --- | --- | --- |
| `side` | `Side` | `Side.Receivable` (sales orders) or `Side.Payable` (purchase orders). |
| `lines` | `list[OpenOrderLine]` | Open order lines ordered by date. |
| `left_out` | `LeftOut` | Skipped rows and pages if the file has damage. |

### `OpenOrderLine`

| Field | Type | Description |
| --- | --- | --- |
| `transaction_id` | `int` | Order transaction ID. |
| `date` | `datetime.date` | Order date. |
| `number` | `str \| None` | Order document number. |
| `name_id` | `int \| None` | Customer or vendor ID. |
| `name` | `str \| None` | Customer or vendor full name. |
| `is_order_done` | `bool` | Whether the order header is marked done or manually closed. |
| `item_id` | `int \| None` | Item ID on the line. |
| `item_name` | `str \| None` | Full item name. |
| `description` | `str \| None` | Line description. |
| `ordered` | `str` | Ordered quantity. |
| `filled` | `str` | Quantity received or invoiced so far. |
| `open` | `str` | Remaining open quantity. |
| `open_amount` | `Money \| None` | Remaining open amount in the order currency. |

---

## Supporting Lists

### Methods on `Company`

| Method | Returns | Description |
| --- | --- | --- |
| `currencies(active_only=False)` | `list[Currency]` | Currencies with ISO 4217 codes, symbols, decimal places, and historical exchange rates. |
| `terms(active_only=False)` | `list[Terms]` | Standard and date-driven payment terms. |
| `classes(active_only=False)` | `list[ClassEntry]` | Hierarchical classes. |
| `payment_methods(active_only=False)` | `list[PaymentMethod]` | Accepted payment methods. |
| `ship_methods(active_only=False)` | `list[ShipMethod]` | Shipping methods. |
| `sales_reps(active_only=False)` | `list[SalesRep]` | Sales representatives linked to names. |
| `sales_tax_codes(active_only=False)` | `list[SalesTaxCode]` | Taxable and non-taxable sales tax codes. |

### `Currency`

| Field | Type | Description |
| --- | --- | --- |
| `id` | `int` | Currency ID. |
| `name` | `str` | Currency display name. |
| `code` | `str` | ISO 4217 code (for example `"USD"` or `"EUR"`). |
| `symbol` | `str \| None` | Currency symbol (for example `"$"`). |
| `decimal_places` | `int \| None` | Decimal places used when formatting amounts. |
| `is_active` | `bool` | Whether the currency is active. |
| `rates` | `list[ExchangeRate]` | Historical exchange rates ordered by date. |

### `ExchangeRate`

| Field | Type | Description |
| --- | --- | --- |
| `as_of` | `datetime.date` | Effective date of the rate. |
| `rate` | `float` | Value of one unit in the home currency. |

### `Terms`

| Field | Type | Description |
| --- | --- | --- |
| `id` | `int` | Terms ID. |
| `name` | `str` | Display name (for example `"Net 30"`). |
| `kind` | `str` | `"Standard"` or `"Date Driven"`. |
| `due_days` | `int` | Days until due (standard terms). |
| `discount_days` | `int` | Days eligible for early-payment discount (standard terms). |
| `discount_percent` | `str \| None` | Early-payment discount percentage. |
| `due_day` | `int` | Day of month due (date-driven terms). |
| `next_month_days` | `int` | Cutoff days before `due_day` to roll into the following month. |
| `discount_day` | `int` | Day of month eligible for discount (date-driven terms). |
| `is_active` | `bool` | Whether the terms entry is active. |

### `ClassEntry`

| Field | Type | Description |
| --- | --- | --- |
| `id` | `int` | Class ID. |
| `name` | `str` | Short class name. |
| `full_name` | `str` | Full hierarchical class name separated by `:`. |
| `parent_id` | `int \| None` | Parent class ID, if nested. |
| `is_active` | `bool` | Whether the class is active. |

### `PaymentMethod`

| Field | Type | Description |
| --- | --- | --- |
| `id` | `int` | Payment method ID. |
| `name` | `str` | Display name (for example `"Check"`, `"Visa"`). |
| `kind` | `str` | Payment method kind label. |
| `is_active` | `bool` | Whether the method is active. |

### `ShipMethod`

| Field | Type | Description |
| --- | --- | --- |
| `id` | `int` | Shipping method ID. |
| `name` | `str` | Display name. |
| `is_active` | `bool` | Whether the method is active. |

### `SalesRep`

| Field | Type | Description |
| --- | --- | --- |
| `id` | `int` | Sales rep ID. |
| `initials` | `str` | Initials shown on sales forms. |
| `name_id` | `int \| None` | Linked name ID, if set. |
| `name` | `str \| None` | Linked full name, if resolved. |
| `is_active` | `bool` | Whether the sales rep is active. |

### `SalesTaxCode`

| Field | Type | Description |
| --- | --- | --- |
| `id` | `int` | Tax code ID. |
| `code` | `str` | Short code (for example `"Tax"` or `"Non"`). |
| `description` | `str \| None` | Description. |
| `is_taxable` | `bool` | Whether lines with this code are taxable. |
| `is_active` | `bool` | Whether the tax code is active. |

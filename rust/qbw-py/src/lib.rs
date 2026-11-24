//! Python bindings for `qbw`, generated with Bimetal.

mod arrow_bridge;
mod convert;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use bimetal::Model;
use bimetal::arrow::{RecordBatchStream, Table as ArrowTable};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Sign of a database file's page key formula.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Model)]
#[serde(rename_all = "snake_case")]
pub enum KeySign {
    /// Positive page keys.
    Positive,
    /// Negative page keys.
    Negative,
}

/// Constants that generate the scrambling key for every page in a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct StoreKey {
    /// 25-bit XOR mask applied to the page number.
    pub mask: u32,
    /// 25-bit addend applied after the mask.
    pub addend: u32,
    /// Sign that selects the sector walk direction.
    pub sign: KeySign,
}

/// Options controlling how a QuickBooks Desktop company file is opened.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct OpenOptions {
    /// Explicit page key to use instead of searching for one.
    #[serde(default)]
    pub key: Option<StoreKey>,
    /// Maximum decompressed byte length allowed for a single compressed value
    /// (defaults to 16 MiB).
    #[serde(default)]
    pub longest_value: Option<u32>,
}

/// Character encoding declared by the underlying database file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Model)]
pub enum TextEncoding {
    /// Windows-1252 single-byte encoding.
    #[serde(rename = "windows-1252")]
    Windows1252,
    /// UTF-8 encoding.
    #[serde(rename = "utf-8")]
    Utf8,
}

/// Low-level file metadata from the underlying SQL Anywhere 17 database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct FileInfo {
    /// SQL Anywhere format version string (for example `"3.201.12"`).
    pub format_version: String,
    /// Page size in bytes.
    pub page_size: u32,
    /// Total number of pages in the file.
    pub page_count: u32,
    /// Character encoding declared by the superblock.
    pub encoding: TextEncoding,
    /// Number of catalog entries in the database.
    pub table_count: u32,
    /// Log sequence number from page 0.
    pub lsn: u32,
    /// Content identity digest distinguishing the file and its write state.
    pub digest: String,
}

/// Stored row location in the underlying SQL Anywhere database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct Origin {
    /// Name of the underlying table.
    pub table: String,
    /// Zero-based page number.
    pub page: u32,
    /// Zero-based slot index within the page.
    pub slot: u16,
}

/// Counts of unreadable rows, pages, and unread tables omitted from a result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct LeftOut {
    /// Individual rows that could not be read.
    pub rows: u32,
    /// Whole pages that could not be read.
    pub pages: u32,
    /// Tables with rows that this version does not read.
    pub unread_tables: u32,
}

/// A damaged part of the company file encountered while reading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct DamageInfo {
    /// Table containing the damaged record.
    pub table: String,
    /// Row location when known.
    pub row: Option<String>,
    /// Human-readable description of the problem.
    pub problem: String,
}

/// Exact monetary amount in a single currency.
///
/// `amount` is an exact decimal string (for example `"-100.00"`), suitable for
/// `decimal.Decimal(money.amount)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct Money {
    /// Exact decimal amount string.
    pub amount: String,
    /// Currency identifier (`0` in a single-currency company file).
    pub currency_id: i32,
    /// ISO 4217 currency code when declared (for example `"USD"` or `"EUR"`).
    pub currency_code: Option<String>,
}

/// Whether a price or rate is a unit amount or a percentage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Model)]
#[serde(rename_all = "snake_case")]
pub enum PriceKind {
    /// Fixed price per unit.
    Each,
    /// Percentage rate.
    Percent,
}

/// A unit price or percentage rate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct Price {
    /// Whether this price is a unit amount or a percentage.
    pub kind: PriceKind,
    /// Exact decimal string.
    pub amount: String,
    /// Currency identifier when `kind == PriceKind.Each`.
    pub currency_id: Option<i32>,
    /// ISO 4217 currency code when `kind == PriceKind.Each` and declared.
    pub currency_code: Option<String>,
}

/// A postal address with printed lines and structured fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct Address {
    /// Lines as formatted for printing on forms.
    pub printed: Vec<String>,
    /// Structured street lines.
    pub street: Vec<String>,
    /// City name.
    pub city: Option<String>,
    /// State or province.
    pub state: Option<String>,
    /// Postal or ZIP code.
    pub postal_code: Option<String>,
    /// Country name.
    pub country: Option<String>,
}

/// Inclusive date span. Either bound may be `None` for an open-ended span.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct Period {
    /// Inclusive first date, or `None` for unbounded start.
    pub start: Option<NaiveDate>,
    /// Inclusive last date, or `None` for unbounded end.
    pub end: Option<NaiveDate>,
}

/// QuickBooks Desktop product version recorded in the company file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct ProductVersion {
    /// Product release year (for example `2024`).
    pub year: Option<i32>,
    /// Release update number.
    pub update: Option<i32>,
    /// Raw edition code stored in the file.
    pub edition: Option<String>,
    /// Newest version string that has opened the file.
    pub newest_used: Option<String>,
}

/// Company profile metadata and QuickBooks version details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct CompanyInfo {
    /// Company display name.
    pub name: String,
    /// Legal business name.
    pub legal_name: Option<String>,
    /// Structured legal address.
    pub legal_address: Address,
    /// Printed address lines stored for forms.
    pub address_lines: Vec<String>,
    /// Primary phone number.
    pub phone: Option<String>,
    /// Fax number.
    pub fax: Option<String>,
    /// Company e-mail address.
    pub email: Option<String>,
    /// Company website URL.
    pub website: Option<String>,
    /// Start date of the company's books.
    pub started: Option<NaiveDate>,
    /// UTC timestamp when the company file was created.
    pub created: Option<DateTime<Utc>>,
    /// QuickBooks Desktop product version metadata.
    pub version: ProductVersion,
    /// Counts of unreadable rows omitted from company metadata.
    pub left_out: LeftOut,
}

/// Date from which open invoices and bills are aged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Model)]
#[serde(rename_all = "snake_case")]
pub enum AgingBasis {
    /// Age from the due date.
    DueDate,
    /// Age from the transaction date.
    TransactionDate,
}

/// Company-wide accounting and feature preferences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct Preferences {
    /// Currency identifier of the company's home currency.
    pub home_currency_id: i32,
    /// ISO 4217 code of the home currency, if declared.
    pub home_currency_code: Option<String>,
    /// Whether multicurrency is enabled.
    pub uses_multicurrency: bool,
    /// Starting month number (`1` = January .. `12` = December) of the fiscal year.
    pub fiscal_year_start: u32,
    /// Starting month number (`1`..`12`) of the income tax year, if set.
    pub tax_year_start: Option<u32>,
    /// Default date basis used for aging reports.
    pub aging_basis: AgingBasis,
    /// Whether account numbers are displayed.
    pub uses_account_numbers: bool,
    /// Whether class tracking is enabled.
    pub uses_classes: bool,
    /// Whether inventory tracking is enabled.
    pub uses_inventory: bool,
    /// Whether sales tax tracking is enabled.
    pub uses_sales_tax: bool,
    /// Whether sales orders are enabled.
    pub uses_sales_orders: bool,
    /// Whether multi-site inventory is enabled.
    pub uses_sites: bool,
    /// Whether bin tracking within sites is enabled.
    pub uses_bins: bool,
    /// Whether serial number tracking is enabled.
    pub uses_serial_numbers: bool,
    /// Whether quantities on sales orders reduce available stock.
    pub available_less_sales_orders: bool,
    /// Whether quantities required by pending builds reduce available stock.
    pub available_less_builds: bool,
    /// Default sales tax item identifier assigned to new customers.
    pub default_tax_item_id: Option<i32>,
    /// Closing date text as stored by QuickBooks.
    pub closing_date: Option<String>,
    /// Counts of unreadable rows omitted from preferences.
    pub left_out: LeftOut,
}

/// Audit-trail user record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct User {
    /// User identifier.
    pub id: i32,
    /// Names seen on transactions saved or deleted by this user, newest first.
    pub names: Vec<String>,
}

/// Normal side on which an account type grows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Model)]
#[serde(rename_all = "snake_case")]
pub enum NormalSide {
    /// Assets, expenses, and cost of goods sold grow by debits.
    Debit,
    /// Liabilities, equity, and income grow by credits.
    Credit,
}

/// Primary financial statement on which an account type appears.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Model)]
#[serde(rename_all = "snake_case")]
pub enum Statement {
    /// Balance sheet account.
    BalanceSheet,
    /// Profit and loss account.
    ProfitAndLoss,
}

/// Balance stored directly by QuickBooks on an account or party record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct StoredBalance {
    /// Balance in the record's own currency.
    pub amount: Money,
    /// Balance in the company's home currency, when kept separately.
    pub home_amount: Option<Money>,
}

/// An account in the chart of accounts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct Account {
    /// Account identifier.
    pub id: i32,
    /// Own name without ancestor prefixes.
    pub name: String,
    /// Full hierarchical name separated by `:` (for example `"Sales:Parts"`).
    pub full_name: String,
    /// Parent account identifier, if this is a sub-account.
    pub parent_id: Option<i32>,
    /// Nesting depth (`0` for top-level accounts).
    pub depth: u32,
    /// Account type display label (for example `"Bank"`, `"Income"`).
    pub account_type: String,
    /// Raw numeric account type code.
    pub account_type_code: i64,
    /// Normal balance side, or `None` for non-posting/unknown account types.
    pub normal_side: Option<NormalSide>,
    /// Financial statement placement, or `None` for non-posting/unknown types.
    pub statement: Option<Statement>,
    /// Built-in QuickBooks role label when this is a special account.
    pub special: Option<String>,
    /// Account number, if assigned.
    pub number: Option<String>,
    /// Account description.
    pub description: Option<String>,
    /// Currency identifier.
    pub currency_id: i32,
    /// ISO 4217 currency code, if declared.
    pub currency_code: Option<String>,
    /// Tax form line code, if mapped.
    pub tax_line: Option<i32>,
    /// Whether the account is active.
    pub is_active: bool,
    /// Stored balance recorded by QuickBooks.
    pub stored: Option<StoredBalance>,
    /// Stored row location in the underlying database.
    pub origin: Origin,
}

/// Ledger balance computed for an account as of a given date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct AccountBalance {
    /// Account identifier.
    pub account_id: i32,
    /// Full account name, if listed.
    pub account_name: Option<String>,
    /// Computed balance in the account's own currency (debit positive).
    pub amount: Money,
    /// Computed balance in the company's home currency (debit positive).
    pub home_amount: Money,
    /// Counts of unreadable rows omitted from the ledger.
    pub left_out: LeftOut,
}

/// Which name list a party belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Model)]
#[serde(rename_all = "snake_case")]
pub enum NameKind {
    /// Customer or customer job.
    Customer,
    /// Vendor.
    Vendor,
    /// Employee.
    Employee,
    /// Other name.
    Other,
}

/// Contact details and primary postal address for a party.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct Contact {
    /// Contact record identifier.
    pub id: String,
    /// Company name.
    pub company: Option<String>,
    /// Salutation prefix.
    pub salutation: Option<String>,
    /// First name.
    pub first_name: Option<String>,
    /// Middle name or initial.
    pub middle_name: Option<String>,
    /// Last name.
    pub last_name: Option<String>,
    /// Job title.
    pub job_title: Option<String>,
    /// Primary contact name.
    pub contact: Option<String>,
    /// Alternate contact name.
    pub alt_contact: Option<String>,
    /// Primary phone number.
    pub phone: Option<String>,
    /// Alternate phone number.
    pub alt_phone: Option<String>,
    /// Mobile phone number.
    pub mobile: Option<String>,
    /// Fax number.
    pub fax: Option<String>,
    /// Primary e-mail address.
    pub email: Option<String>,
    /// CC e-mail address.
    pub cc: Option<String>,
    /// Account number with the customer or vendor.
    pub account_number: Option<String>,
    /// Primary billing or mailing address.
    pub address: Address,
}

/// A customer, job, vendor, employee, or other name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct NameEntry {
    /// Shared name identifier across all four lists.
    pub id: i32,
    /// Which name list this entry belongs to.
    pub kind: NameKind,
    /// Own name without parent customer prefix.
    pub name: String,
    /// Full hierarchical name (`Customer:Job` for jobs).
    pub full_name: String,
    /// Parent customer/job identifier for jobs.
    pub parent_id: Option<i32>,
    /// Whether the entry is active.
    pub is_active: bool,
    /// Trading currency identifier (customers and vendors).
    pub currency_id: Option<i32>,
    /// ISO 4217 currency code, if declared.
    pub currency_code: Option<String>,
    /// Payment terms identifier (customers and vendors).
    pub terms_id: Option<i32>,
    /// Payment terms name, if resolved.
    pub terms_name: Option<String>,
    /// Credit limit (customers and vendors).
    pub credit_limit: Option<Money>,
    /// Whether vendor payments are tracked for 1099 reporting.
    pub is_1099: bool,
    /// Name printed on checks (vendors and employees).
    pub name_on_check: Option<String>,
    /// Resale certificate number (customers).
    pub resale_number: Option<String>,
    /// Hire date (employees).
    pub hired: Option<NaiveDate>,
    /// Opening date (customers, vendors, and other names).
    pub opened: Option<NaiveDate>,
    /// Stored balance recorded by QuickBooks (customers and vendors).
    pub stored: Option<StoredBalance>,
    /// Active contact record.
    pub contact: Option<Contact>,
    /// External QuickBooks ListID string.
    pub list_id: Option<String>,
    /// Stored row location in the underlying database.
    pub origin: Origin,
}

/// A product, service, discount, subtotal, or sales tax item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct Item {
    /// Item identifier.
    pub id: i32,
    /// Own item name.
    pub name: String,
    /// Full hierarchical item name separated by `:`.
    pub full_name: String,
    /// Parent item identifier, if this is a sub-item.
    pub parent_id: Option<i32>,
    /// Item kind display label (for example `"Inventory Part"`, `"Service"`).
    pub kind: String,
    /// Raw numeric item kind code.
    pub kind_code: i64,
    /// Whether this item is tracked in inventory stock.
    pub is_stocked: bool,
    /// Whether the item is active.
    pub is_active: bool,
    /// Sales description.
    pub description: Option<String>,
    /// Sales price or percentage rate.
    pub price: Option<Price>,
    /// Income/sales account identifier.
    pub income_account_id: Option<i32>,
    /// Income/sales account full name.
    pub income_account_name: Option<String>,
    /// Purchase description.
    pub purchase_description: Option<String>,
    /// Purchase cost.
    pub cost: Option<Money>,
    /// Expense/COGS account identifier.
    pub expense_account_id: Option<i32>,
    /// Expense/COGS account full name.
    pub expense_account_name: Option<String>,
    /// Preferred vendor identifier.
    pub preferred_vendor_id: Option<i32>,
    /// Preferred vendor full name.
    pub preferred_vendor_name: Option<String>,
    /// Inventory asset account identifier.
    pub asset_account_id: Option<i32>,
    /// Inventory asset account full name.
    pub asset_account_name: Option<String>,
    /// Reorder point quantity.
    pub reorder_point: Option<String>,
    /// Assembly build point quantity.
    pub build_point: Option<String>,
    /// Manufacturer part number.
    pub part_number: Option<String>,
    /// Stored quantity on hand.
    pub on_hand: Option<String>,
    /// Stored inventory value.
    pub stored_value: Option<Money>,
    /// Stored average unit cost.
    pub stored_average_cost: Option<Money>,
    /// External QuickBooks ListID string.
    pub list_id: Option<String>,
    /// Stored row location in the underlying database.
    pub origin: Origin,
}

/// Current stock status for one inventory item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct StockRow {
    /// Item identifier.
    pub item_id: i32,
    /// Full item name, if listed.
    pub item_name: Option<String>,
    /// Quantity on hand from inventory history.
    pub on_hand: String,
    /// Total stock value in the home currency.
    pub value: Money,
    /// Average unit cost in the home currency.
    pub average_cost: Option<Money>,
    /// Quantity summed directly from transaction lines.
    pub counted: String,
    /// Quantity remaining on open purchase orders.
    pub on_order: String,
    /// Quantity committed on open sales orders.
    pub on_sales_orders: String,
    /// Quantity to be added by pending assembly builds.
    pub to_be_built: String,
    /// Quantity required by pending assembly builds.
    pub required_for_builds: String,
    /// Available quantity after commitments per company preferences.
    pub available: String,
    /// Reorder point quantity, if set.
    pub reorder_point: Option<String>,
    /// Whether `on_hand + on_order <= reorder_point`.
    pub needs_reorder: bool,
}

/// Current inventory stock summary across all stocked items.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct Stock {
    /// One row per stocked item in list order.
    pub rows: Vec<StockRow>,
    /// Total inventory value in the company's home currency.
    pub value: Money,
    /// Counts of unreadable rows omitted from stock figures.
    pub left_out: LeftOut,
}

/// Historical valuation row for one stocked item on a given date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct ValuationRow {
    /// Item identifier.
    pub item_id: i32,
    /// Full item name, if listed.
    pub item_name: Option<String>,
    /// Quantity on hand at the end of the day.
    pub on_hand: String,
    /// Stock value in the home currency.
    pub value: Money,
    /// Average unit cost in the home currency.
    pub average_cost: Option<Money>,
}

/// Inventory valuation report as of a date or latest state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct Valuation {
    /// Effective valuation date, or `None` when evaluated at latest state.
    pub as_of: Option<NaiveDate>,
    /// Valuation rows in item list order.
    pub rows: Vec<ValuationRow>,
    /// Total inventory value in the company's home currency.
    pub value: Money,
    /// Counts of unreadable rows omitted from valuation figures.
    pub left_out: LeftOut,
}

/// Receivable (customer) or payable (vendor) side of the books.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Model)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    /// Customers, invoices, and sales orders.
    Receivable,
    /// Vendors, bills, and purchase orders.
    Payable,
}

/// One open line of a sales order or purchase order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct OpenOrderLine {
    /// Order transaction identifier.
    pub transaction_id: i32,
    /// Order date.
    pub date: NaiveDate,
    /// Order number.
    pub number: Option<String>,
    /// Customer or vendor identifier.
    pub name_id: Option<i32>,
    /// Customer or vendor full name.
    pub name: Option<String>,
    /// Whether the order header is marked done or manually closed.
    pub is_order_done: bool,
    /// Item identifier on the line.
    pub item_id: Option<i32>,
    /// Item full name.
    pub item_name: Option<String>,
    /// Line description.
    pub description: Option<String>,
    /// Quantity ordered.
    pub ordered: String,
    /// Quantity already received or invoiced.
    pub filled: String,
    /// Remaining open quantity.
    pub open: String,
    /// Remaining open amount in the order's currency.
    pub open_amount: Option<Money>,
}

/// All open lines of sales orders or purchase orders.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct OpenOrders {
    /// Receivable (sales orders) or payable (purchase orders).
    pub side: Side,
    /// Open order lines ordered by date.
    pub lines: Vec<OpenOrderLine>,
    /// Counts of unreadable rows omitted from open orders.
    pub left_out: LeftOut,
}

/// Exchange rate snapshot recorded on a currency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Model)]
pub struct ExchangeRate {
    /// Effective date of the rate.
    pub as_of: NaiveDate,
    /// Value of one unit in the company's home currency.
    pub rate: f64,
}

/// A currency from the company's currency list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Model)]
pub struct Currency {
    /// Currency identifier.
    pub id: i32,
    /// Currency name.
    pub name: String,
    /// ISO 4217 code (for example `"USD"` or `"EUR"`).
    pub code: String,
    /// Currency symbol (for example `"$"`).
    pub symbol: Option<String>,
    /// Decimal places used when formatting amounts.
    pub decimal_places: Option<i32>,
    /// Whether this currency is active in the company.
    pub is_active: bool,
    /// Historical exchange rates ordered by date.
    pub rates: Vec<ExchangeRate>,
}

/// Payment terms from the terms list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct Terms {
    /// Terms identifier.
    pub id: i32,
    /// Terms display name (for example `"Net 30"`).
    pub name: String,
    /// Terms kind label (`"Standard"` or `"Date Driven"`).
    pub kind: String,
    /// Days until due (for standard terms).
    pub due_days: i32,
    /// Days eligible for early-payment discount (for standard terms).
    pub discount_days: i32,
    /// Early-payment discount percentage.
    pub discount_percent: Option<String>,
    /// Day of month due (for date-driven terms).
    pub due_day: i32,
    /// Cutoff days before due day to roll into the following month.
    pub next_month_days: i32,
    /// Day of month eligible for discount (for date-driven terms).
    pub discount_day: i32,
    /// Whether these terms are active.
    pub is_active: bool,
}

/// A class from the class list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct ClassEntry {
    /// Class identifier.
    pub id: i32,
    /// Own class name.
    pub name: String,
    /// Full hierarchical class name separated by `:`.
    pub full_name: String,
    /// Parent class identifier, if nested.
    pub parent_id: Option<i32>,
    /// Whether the class is active.
    pub is_active: bool,
}

/// A payment method from the payment methods list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct PaymentMethod {
    /// Payment method identifier.
    pub id: i32,
    /// Method name (for example `"Check"`, `"Visa"`).
    pub name: String,
    /// Payment method kind label.
    pub kind: String,
    /// Whether the method is active.
    pub is_active: bool,
}

/// A shipping method from the ship methods list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct ShipMethod {
    /// Shipping method identifier.
    pub id: i32,
    /// Method name.
    pub name: String,
    /// Whether the method is active.
    pub is_active: bool,
}

/// A sales representative entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct SalesRep {
    /// Sales rep identifier.
    pub id: i32,
    /// Initials shown on sales forms.
    pub initials: String,
    /// Backing name identifier, if linked.
    pub name_id: Option<i32>,
    /// Backing name full name, if resolved.
    pub name: Option<String>,
    /// Whether the sales rep is active.
    pub is_active: bool,
}

/// A sales tax code entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct SalesTaxCode {
    /// Tax code identifier.
    pub id: i32,
    /// Short tax code (for example `"Tax"` or `"Non"`).
    pub code: String,
    /// Description.
    pub description: Option<String>,
    /// Whether lines carrying this code are taxable.
    pub is_taxable: bool,
    /// Whether the tax code is active.
    pub is_active: bool,
}

/// Reconciliation status of a posting or transaction line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Model)]
#[serde(rename_all = "snake_case")]
pub enum Cleared {
    /// Not yet cleared or reconciled.
    NotCleared,
    /// Ticked as cleared in a reconciliation.
    Cleared,
    /// Reconciled on a finished bank statement.
    Reconciled,
    /// Matched against a bank feed.
    Matched,
}

/// Whether a transaction posts to the general ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Model)]
#[serde(rename_all = "snake_case")]
pub enum PostingState {
    /// Contributes postings to accounts and financial statements.
    Posted,
    /// Does not post to the general ledger (for example estimates and orders).
    NonPosting,
}

/// Settlement status filter for transaction queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Model)]
#[serde(rename_all = "snake_case")]
pub enum PaidState {
    /// Fully paid or applied.
    Paid,
    /// Still has an open balance.
    Unpaid,
}

/// Sort key for `Company.transactions()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Model)]
#[serde(rename_all = "snake_case")]
pub enum TransactionOrder {
    /// Sort by transaction date.
    Date,
    /// Sort in order entered.
    Entered,
    /// Sort by document number in natural order.
    Number,
    /// Sort by party name.
    Name,
    /// Sort by home-currency total.
    Total,
    /// Sort by due date.
    DueDate,
}

/// Sort direction for `Company.transactions()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Model)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    /// Ascending order.
    Ascending,
    /// Descending order.
    Descending,
}

/// Filter and ordering criteria for `Company.transactions()`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct TransactionFilter {
    /// Restrict to specific transaction kind names (for example `["Invoice", "Bill"]`).
    #[serde(default)]
    pub kinds: Option<Vec<String>>,
    /// Inclusive earliest transaction date.
    #[serde(default)]
    pub start: Option<NaiveDate>,
    /// Inclusive latest transaction date.
    #[serde(default)]
    pub end: Option<NaiveDate>,
    /// Restrict to transactions with this party identifier.
    #[serde(default)]
    pub name_id: Option<i32>,
    /// When `name_id` is a customer, also include transactions of its jobs.
    #[serde(default)]
    pub with_jobs: bool,
    /// Restrict to transactions whose own side is this account identifier.
    #[serde(default)]
    pub account_id: Option<i32>,
    /// Restrict to transactions posting any line to this account identifier.
    #[serde(default)]
    pub posts_to_account_id: Option<i32>,
    /// Restrict to transactions matching this document number (case-insensitive).
    #[serde(default)]
    pub number: Option<String>,
    /// Restrict by paid/unpaid status.
    #[serde(default)]
    pub paid: Option<PaidState>,
    /// Case-insensitive text search across number, name, memo, and line descriptions.
    #[serde(default)]
    pub text: Option<String>,
    /// Column to sort by (defaults to `TransactionOrder.Date`).
    #[serde(default)]
    pub order: Option<TransactionOrder>,
    /// Sort direction (defaults to `SortDirection.Descending`).
    #[serde(default)]
    pub direction: Option<SortDirection>,
}

/// One line of a transaction form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct TransactionLine {
    /// Line identifier.
    pub id: i32,
    /// Whether this line is also the transaction's own side (such as line 1 of a journal entry).
    pub is_own: bool,
    /// Structural role of the line (for example `"Item"`, `"Account"`, `"Sales Tax"`).
    pub role: String,
    /// Item identifier, if this is an item line.
    pub item_id: Option<i32>,
    /// Full item name, if resolved.
    pub item_name: Option<String>,
    /// Account identifier on the line.
    pub account_id: Option<i32>,
    /// Full account name, if resolved.
    pub account_name: Option<String>,
    /// Line-level name identifier, if different from the header.
    pub name_id: Option<i32>,
    /// Line-level full name, if resolved.
    pub name: Option<String>,
    /// Class identifier on the line.
    pub class_id: Option<i32>,
    /// Full class name, if resolved.
    pub class_name: Option<String>,
    /// Line description or memo.
    pub description: Option<String>,
    /// Exact quantity string.
    pub quantity: Option<String>,
    /// Unit rate or percentage.
    pub rate: Option<Price>,
    /// Line amount in the transaction's currency.
    pub amount: Option<Money>,
    /// Line amount in the company's home currency.
    pub home_amount: Option<Money>,
    /// Journal entry debit amount, when applicable.
    pub debit: Option<Money>,
    /// Journal entry credit amount, when applicable.
    pub credit: Option<Money>,
    /// Service date on the line.
    pub service_date: Option<NaiveDate>,
    /// Bank reconciliation status.
    pub cleared: Cleared,
    /// Bank statement date when reconciled.
    pub reconciled_on: Option<NaiveDate>,
    /// Stored row location in the underlying database.
    pub origin: Origin,
}

/// Complete transaction header and lines.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Model)]
pub struct Transaction {
    /// Transaction identifier.
    pub id: i32,
    /// Transaction kind display label (for example `"Invoice"`, `"Check"`).
    pub kind: String,
    /// Raw numeric transaction kind code.
    pub kind_code: i64,
    /// Transaction date.
    pub date: NaiveDate,
    /// Document number shown on the form.
    pub number: Option<String>,
    /// Party identifier.
    pub name_id: Option<i32>,
    /// Party full name, if resolved.
    pub name: Option<String>,
    /// Own-side account identifier.
    pub account_id: Option<i32>,
    /// Own-side account full name, if resolved.
    pub account_name: Option<String>,
    /// Transaction memo.
    pub memo: Option<String>,
    /// Currency identifier.
    pub currency_id: i32,
    /// ISO 4217 currency code, if declared.
    pub currency_code: Option<String>,
    /// Stored exchange rate to home currency.
    pub exchange_rate: f64,
    /// Form total in the transaction's currency.
    pub total: Option<Money>,
    /// Form total in the company's home currency.
    pub home_total: Option<Money>,
    /// Sales form subtotal before sales tax.
    pub subtotal: Option<Money>,
    /// Sales tax total on sales forms.
    pub tax_total: Option<Money>,
    /// Due date, if applicable.
    pub due_date: Option<NaiveDate>,
    /// Remaining open amount, if trackable.
    pub open: Option<Money>,
    /// Whether the transaction is fully paid/applied (`None` when not applicable).
    pub is_paid: Option<bool>,
    /// Whether the transaction was voided.
    pub is_voided: bool,
    /// Whether this transaction posts to the general ledger.
    pub posting: PostingState,
    /// Payment terms display name recorded on the form.
    pub terms_name: Option<String>,
    /// Sales rep initials recorded on the form.
    pub sales_rep_initials: Option<String>,
    /// Lines in form order.
    pub lines: Vec<TransactionLine>,
    /// Damage encountered while reading this transaction's lines or references.
    pub damage: Vec<DamageInfo>,
    /// Stored row location of the transaction header.
    pub origin: Origin,
}

/// Summary row for a transaction returned in paginated listings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct TransactionSummary {
    /// Transaction identifier.
    pub id: i32,
    /// Transaction kind display label.
    pub kind: String,
    /// Raw numeric transaction kind code.
    pub kind_code: i64,
    /// Transaction date.
    pub date: NaiveDate,
    /// Document number.
    pub number: Option<String>,
    /// Party identifier.
    pub name_id: Option<i32>,
    /// Party full name, if resolved.
    pub name: Option<String>,
    /// Own-side account identifier.
    pub account_id: Option<i32>,
    /// Own-side account full name, if resolved.
    pub account_name: Option<String>,
    /// Destination account identifier on transfers.
    pub other_account_id: Option<i32>,
    /// Destination account full name on transfers.
    pub other_account_name: Option<String>,
    /// Transaction memo.
    pub memo: Option<String>,
    /// Currency identifier.
    pub currency_id: i32,
    /// ISO 4217 currency code, if declared.
    pub currency_code: Option<String>,
    /// Own-side amount (debit positive).
    pub amount: Option<Money>,
    /// Form total in the transaction's currency.
    pub total: Option<Money>,
    /// Form total in the company's home currency.
    pub home_total: Option<Money>,
    /// Due date.
    pub due_date: Option<NaiveDate>,
    /// Remaining open amount.
    pub open: Option<Money>,
    /// Whether the transaction is fully paid/applied.
    pub is_paid: Option<bool>,
    /// Whether the transaction was voided.
    pub is_voided: bool,
    /// Whether the transaction posts to the general ledger.
    pub posting: PostingState,
    /// Stored row location of the transaction header.
    pub origin: Origin,
    /// Counts of unreadable rows omitted from this summary.
    pub left_out: LeftOut,
}

/// Count of matching transactions for one kind in a `TransactionPage`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct KindCount {
    /// Transaction kind display label.
    pub kind: String,
    /// Raw numeric transaction kind code.
    pub kind_code: i64,
    /// Number of matching transactions of this kind.
    pub count: u32,
}

/// One page of transactions returned by `Company.transactions()`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct TransactionPage {
    /// Summaries on this page.
    pub rows: Vec<TransactionSummary>,
    /// Total matching transactions across all pages.
    pub total_count: u32,
    /// Breakdown of matching transactions by kind.
    pub kinds: Vec<KindCount>,
    /// Zero-based starting row offset.
    pub offset: u32,
    /// Requested page size.
    pub limit: u32,
    /// Counts of unreadable rows omitted from the query.
    pub left_out: LeftOut,
}

/// One row of an account register with running balance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct RegisterRow {
    /// Transaction date.
    pub date: NaiveDate,
    /// Transaction kind display label.
    pub kind: String,
    /// Document number.
    pub number: Option<String>,
    /// Party identifier.
    pub name_id: Option<i32>,
    /// Party full name.
    pub name: Option<String>,
    /// Counterparty account name, `"Split"`, or `None`.
    pub other_side: Option<String>,
    /// Line or header memo.
    pub memo: Option<String>,
    /// Reconciliation status.
    pub cleared: Cleared,
    /// Reconciliation statement date.
    pub reconciled_on: Option<NaiveDate>,
    /// Posting amount in the account's currency (debit positive).
    pub amount: Money,
    /// Running account balance after this row (debit positive).
    pub balance: Money,
    /// Transaction identifier.
    pub transaction_id: i32,
    /// Line identifier.
    pub line_id: i32,
    /// Stored row location.
    pub origin: Origin,
}

/// Account register over an optional date period, paginated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct Register {
    /// Account identifier.
    pub account_id: i32,
    /// Full account name, if listed.
    pub account_name: Option<String>,
    /// Opening balance before the period in the account's currency.
    pub opening_balance: Money,
    /// Closing balance after the period in the account's currency.
    pub closing_balance: Money,
    /// Register rows on this page.
    pub rows: Vec<RegisterRow>,
    /// Total register rows in the period across all pages.
    pub total_count: u32,
    /// Counts of unreadable rows omitted from the register.
    pub left_out: LeftOut,
}

/// A link between one transaction and another (for example payment to invoice).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct TransactionLink {
    /// Linked transaction identifier.
    pub transaction_id: i32,
    /// Linked transaction kind display label.
    pub kind: String,
    /// Linked transaction date.
    pub date: NaiveDate,
    /// Linked transaction document number.
    pub number: Option<String>,
    /// Linked transaction party identifier.
    pub name_id: Option<i32>,
    /// Linked transaction party full name.
    pub name: Option<String>,
    /// Relationship description (for example `"paid by"`, `"applied to"`).
    pub relation: String,
    /// Settled or linked monetary amount.
    pub amount: Option<Money>,
    /// Linked quantity (for order fulfillment links).
    pub quantity: Option<String>,
}

/// All links or settlement links for a transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct TransactionLinks {
    /// Source transaction identifier.
    pub transaction_id: i32,
    /// Currency identifier of link amounts.
    pub currency_id: i32,
    /// ISO 4217 currency code, if declared.
    pub currency_code: Option<String>,
    /// Linked transactions in order.
    pub links: Vec<TransactionLink>,
    /// Sum of settlement amounts (set by `paid_by` and `applied_to`).
    pub total: Option<Money>,
    /// Counts of unreadable rows omitted from link resolution.
    pub left_out: LeftOut,
}

/// One account line (with nested sub-accounts) inside a financial statement section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct SectionAccount {
    /// Account identifier.
    pub account_id: i32,
    /// Own account name, if listed.
    pub account_name: Option<String>,
    /// Full hierarchical account name, if listed.
    pub account_full_name: Option<String>,
    /// Account number, if assigned.
    pub account_number: Option<String>,
    /// Amount posted directly to this account.
    pub amount: Money,
    /// Rollup total of this account and all sub-accounts.
    pub total: Money,
    /// Child sub-accounts in list order.
    pub children: Vec<SectionAccount>,
}

/// One account-type section of a Balance Sheet or Profit and Loss statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct ReportSection {
    /// Account type display label (for example `"Bank"`, `"Income"`).
    pub account_type: String,
    /// Top-level accounts in this section.
    pub accounts: Vec<SectionAccount>,
    /// Section total in the home currency.
    pub total: Money,
}

/// Assets block of a `BalanceSheet`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct Assets {
    /// Bank accounts section.
    pub bank: ReportSection,
    /// Accounts receivable section.
    pub accounts_receivable: ReportSection,
    /// Other current assets section.
    pub other_current_assets: ReportSection,
    /// Sum of bank, accounts receivable, and other current assets.
    pub current_assets: Money,
    /// Fixed assets section.
    pub fixed_assets: ReportSection,
    /// Other assets section.
    pub other_assets: ReportSection,
    /// Total assets.
    pub total: Money,
}

/// Liabilities block of a `BalanceSheet`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct Liabilities {
    /// Accounts payable section.
    pub accounts_payable: ReportSection,
    /// Credit card accounts section.
    pub credit_cards: ReportSection,
    /// Other current liabilities section.
    pub other_current_liabilities: ReportSection,
    /// Sum of accounts payable, credit cards, and other current liabilities.
    pub current_liabilities: Money,
    /// Long-term liabilities section.
    pub long_term_liabilities: ReportSection,
    /// Total liabilities.
    pub total: Money,
}

/// Equity block of a `BalanceSheet`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct Equity {
    /// Equity accounts section (Retained Earnings includes prior-year profit).
    pub accounts: ReportSection,
    /// Cumulative profit of all fiscal years before the balance sheet's year.
    pub earlier_profit: Money,
    /// Retained Earnings account identifier when earlier profit is rolled into it.
    pub retained_earnings_account_id: Option<i32>,
    /// Fiscal year-to-date net income.
    pub net_income: Money,
    /// Total equity.
    pub total: Money,
}

/// Balance Sheet statement as of a given date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct BalanceSheet {
    /// Effective date of the balance sheet.
    pub as_of: NaiveDate,
    /// Assets sections and totals.
    pub assets: Assets,
    /// Liabilities sections and totals.
    pub liabilities: Liabilities,
    /// Equity section, retained profit, and net income.
    pub equity: Equity,
    /// Combined total of liabilities and equity.
    pub liabilities_and_equity: Money,
    /// Balance posted to accounts that do not map to any statement section.
    pub unplaced: Money,
    /// Counts of unreadable rows omitted from the statement.
    pub left_out: LeftOut,
}

/// Profit and Loss (Income Statement) over a date range.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct ProfitAndLoss {
    /// Inclusive start date, or `None` if unbounded.
    pub start: Option<NaiveDate>,
    /// Inclusive end date, or `None` if unbounded.
    pub end: Option<NaiveDate>,
    /// Operating income section.
    pub income: ReportSection,
    /// Cost of goods sold section.
    pub cost_of_goods_sold: ReportSection,
    /// Income minus cost of goods sold.
    pub gross_profit: Money,
    /// Operating expenses section.
    pub expense: ReportSection,
    /// Gross profit minus operating expenses.
    pub net_ordinary_income: Money,
    /// Other income section.
    pub other_income: ReportSection,
    /// Other expense section.
    pub other_expense: ReportSection,
    /// Other income minus other expense.
    pub net_other_income: Money,
    /// Net income (`net_ordinary_income + net_other_income`).
    pub net_income: Money,
    /// Amount posted to accounts outside standard statement sections.
    pub unplaced: Money,
    /// Counts of unreadable rows omitted from the statement.
    pub left_out: LeftOut,
}

/// One row of a `TrialBalance`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct TrialBalanceRow {
    /// Account identifier (`None` for standalone earlier profit when no Retained Earnings account exists).
    pub account_id: Option<i32>,
    /// Full account name, if listed.
    pub account_name: Option<String>,
    /// Account type label, if listed.
    pub account_type: Option<String>,
    /// Signed balance in the home currency (debit positive).
    pub balance: Money,
    /// Debit column amount, if debit balance.
    pub debit: Option<Money>,
    /// Credit column amount (positive), if credit balance.
    pub credit: Option<Money>,
}

/// Trial Balance as of a given date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct TrialBalance {
    /// Effective date of the trial balance.
    pub as_of: NaiveDate,
    /// Non-zero account rows in chart-of-accounts order.
    pub rows: Vec<TrialBalanceRow>,
    /// Sum of debit balances.
    pub debits: Money,
    /// Sum of credit balances.
    pub credits: Money,
    /// Counts of unreadable rows omitted from the trial balance.
    pub left_out: LeftOut,
}

/// One unsettled invoice, bill, credit, or unapplied payment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct OpenItem {
    /// Transaction identifier.
    pub transaction_id: i32,
    /// Transaction kind display label.
    pub kind: String,
    /// Transaction date.
    pub date: NaiveDate,
    /// Document number.
    pub number: Option<String>,
    /// Customer or vendor identifier.
    pub name_id: Option<i32>,
    /// Customer or vendor full name.
    pub name: Option<String>,
    /// Receivable or payable account identifier.
    pub account_id: i32,
    /// Account full name, if listed.
    pub account_name: Option<String>,
    /// Due date, if applicable.
    pub due_date: Option<NaiveDate>,
    /// Original total in the account's currency.
    pub total: Money,
    /// Remaining open amount in the account's currency.
    pub open: Money,
    /// Remaining open amount in the company's home currency.
    pub home_open: Money,
    /// Stored row location.
    pub origin: Origin,
}

/// Open receivables or payables at a date or latest state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct OpenItems {
    /// Receivable or payable side.
    pub side: Side,
    /// Effective date, or `None` when evaluated at latest state.
    pub as_of: Option<NaiveDate>,
    /// Unsettled transactions in date order.
    pub items: Vec<OpenItem>,
    /// Home-currency sum of all open items.
    pub home_total: Money,
    /// Home-currency exchange-rate adjustment not attached to individual items.
    pub adjustments: Money,
    /// Counts of unreadable rows omitted from open items.
    pub left_out: LeftOut,
}

/// One age bucket in an `Aging` report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct AgingBucket {
    /// Human-readable label (for example `"Current"`, `"1-30"`, `">90"`).
    pub label: String,
    /// Inclusive lower bound in days (`None` for the Current bucket).
    pub first_day: Option<i64>,
    /// Inclusive upper bound in days (`None` for the oldest bucket).
    pub last_day: Option<i64>,
}

/// One customer or vendor row in an `Aging` report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct AgingRow {
    /// Customer or vendor identifier.
    pub name_id: Option<i32>,
    /// Customer or vendor full name.
    pub name: Option<String>,
    /// Bucket amounts in the party's trading currency.
    pub amounts: Vec<Money>,
    /// Total open amount in the party's trading currency.
    pub total: Money,
    /// Bucket amounts in the company's home currency.
    pub home_amounts: Vec<Money>,
    /// Total open amount in the company's home currency.
    pub home_total: Money,
}

/// Accounts Receivable or Accounts Payable aging summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct Aging {
    /// Receivable or payable side.
    pub side: Side,
    /// Date ages are measured to.
    pub day: NaiveDate,
    /// Date basis ages are counted from.
    pub basis: AgingBasis,
    /// Ordered aging buckets.
    pub buckets: Vec<AgingBucket>,
    /// One row per customer or vendor.
    pub rows: Vec<AgingRow>,
    /// Home-currency column totals matching `buckets`.
    pub totals: Vec<Money>,
    /// Grand total across all buckets in the home currency.
    pub total: Money,
    /// Exchange-rate adjustments in the home currency.
    pub adjustments: Money,
    /// Counts of unreadable rows omitted from the aging report.
    pub left_out: LeftOut,
}

/// Agreement count for one check category in `CheckReport`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct CheckComparison {
    /// Category label.
    pub kind: String,
    /// Total records or figures compared.
    pub looked_at: u32,
    /// Number of records or figures that agreed.
    pub agreed: u32,
}

/// One discrepancy found by `Company.check()`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct CheckFinding {
    /// Category label.
    pub kind: String,
    /// Description of the record or scope where figures differed.
    pub subject: String,
    /// Computed figure as formatted text, if applicable.
    pub computed: Option<String>,
    /// Stored or expected figure as formatted text, if applicable.
    pub expected: Option<String>,
    /// Whether quantified causes account for the full difference.
    pub is_explained: bool,
    /// Known or possible causes for the discrepancy.
    pub causes: Vec<String>,
}

/// Check category skipped because an optional table was absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct CheckNotCompared {
    /// Category label.
    pub kind: String,
    /// Missing table name.
    pub table: String,
}

/// Cross-check comparing computed accounting figures against QuickBooks' stored totals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Model)]
pub struct CheckReport {
    /// Agreement counts by category.
    pub compared: Vec<CheckComparison>,
    /// All discrepancies found.
    pub findings: Vec<CheckFinding>,
    /// Count of discrepancies whose quantified causes do not explain the full difference.
    pub unexplained_count: u32,
    /// Categories skipped due to missing optional tables.
    pub not_compared: Vec<CheckNotCompared>,
    /// Counts of unreadable rows omitted from the check.
    pub left_out: LeftOut,
}

/// Errors returned when opening or reading a QuickBooks Desktop company file.
#[derive(Debug, thiserror::Error, bimetal::Error)]
pub enum QbwError {
    /// The operating system could not read the file.
    #[error("{message}")]
    #[bimetal(details)]
    Io {
        /// Underlying OS error message.
        message: String,
    },
    /// The file is not a valid SQL Anywhere 17 database.
    #[error("{message}")]
    #[bimetal(details)]
    Format {
        /// Format validation failure details.
        message: String,
    },
    /// No scrambling key matches the file's pages.
    #[error("no scrambling key matches this file")]
    NoKey,
    /// Multiple scrambling keys fit equally well; supply one in `OpenOptions`.
    #[error("multiple scrambling keys fit this file")]
    #[bimetal(details)]
    AmbiguousKey {
        /// Candidate keys discovered by the search.
        keys: Vec<StoreKey>,
    },
    /// The key passed in `OpenOptions` does not fit the file's pages.
    #[error("the supplied scrambling key does not fit this file")]
    WrongKey,
    /// The database declares an unsupported character encoding.
    #[error("unsupported text encoding: {label}")]
    #[bimetal(details)]
    UnsupportedEncoding {
        /// Encoding label stored in page 0.
        label: String,
    },
    /// Another error occurred inside the underlying `sa17` database reader.
    #[error("{message}")]
    #[bimetal(details)]
    Database {
        /// Underlying database error message.
        message: String,
    },
    /// The database is valid SQL Anywhere 17, but not a QuickBooks company file.
    #[error("the database is not a QuickBooks company file: missing table {table}")]
    #[bimetal(details)]
    NotACompany {
        /// Required QuickBooks table that was missing.
        table: String,
    },
    /// The company file lacks a table required for this query.
    #[error("the company file has no table {table}")]
    #[bimetal(details)]
    NoTable {
        /// Missing table name.
        table: String,
    },
    /// A table lacks a column required by this query.
    #[error("table {table} has no column {column}")]
    #[bimetal(details)]
    Schema {
        /// Table name.
        table: String,
        /// Missing column name.
        column: String,
    },
    /// The specifically requested record is damaged and cannot be read.
    #[error("{message}")]
    #[bimetal(details)]
    Damaged {
        /// Table containing the damaged record.
        table: String,
        /// Row location when known.
        row: Option<String>,
        /// Description of the damage.
        message: String,
    },
    /// A caller-supplied argument is invalid (for example an unknown transaction kind or zero bucket size).
    #[error("{message}")]
    #[bimetal(details)]
    InvalidArgument {
        /// Description of what was invalid.
        message: String,
    },
    /// Exporting company data to Apache Arrow failed.
    #[error("{message}")]
    #[bimetal(details)]
    Arrow {
        /// Arrow conversion failure details.
        message: String,
    },
}

/// An open QuickBooks Desktop (`.qbw`) company file.
pub struct Company {
    inner: Arc<qbw::Company>,
}

/// An open QuickBooks Desktop (`.qbw`) company file.
///
/// Use `Company(path)` or `Company.open(path)` to load a `.qbw` file from disk,
/// or `Company.from_bytes(data)` for in-memory bytes. Supports `with` blocks.
#[bimetal::export]
impl Company {
    /// Open a QuickBooks Desktop `.qbw` company file from disk.
    #[bimetal(constructor)]
    pub fn new(path: PathBuf, options: Option<OpenOptions>) -> Result<Self, QbwError> {
        Self::open(path, options)
    }

    /// Open a QuickBooks Desktop `.qbw` company file from disk.
    #[bimetal(constructor)]
    pub fn open(path: PathBuf, options: Option<OpenOptions>) -> Result<Self, QbwError> {
        let options = convert::open_options(options);
        let inner = qbw::Company::open_with(path, &options).map_err(convert::error)?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    /// Open a QuickBooks Desktop `.qbw` company file from raw bytes in memory.
    #[bimetal(constructor)]
    pub fn from_bytes(data: Vec<u8>, options: Option<OpenOptions>) -> Result<Self, QbwError> {
        let options = convert::open_options(options);
        let inner = qbw::Company::from_bytes_with(data, &options).map_err(convert::error)?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    /// Low-level SQL Anywhere 17 file metadata.
    #[bimetal(getter)]
    #[must_use]
    pub fn file_info(&self) -> FileInfo {
        convert::file_info(self.inner.source())
    }

    /// Recovered SQL Anywhere 17 page-scrambling key.
    #[bimetal(getter)]
    #[must_use]
    pub fn key(&self) -> StoreKey {
        convert::store_key_from_core(self.inner.source().store().key())
    }

    /// Names of all catalog tables in the underlying database.
    #[must_use]
    pub fn table_names(&self) -> Vec<String> {
        self.inner
            .source()
            .catalog()
            .iter()
            .map(|table| table.name().to_owned())
            .collect()
    }

    /// Read the full raw SQL Anywhere row at `origin` as a column-name dict,
    /// or `None` if the table or slot does not exist.
    pub fn stored_row(
        &self,
        origin: Origin,
    ) -> Result<Option<BTreeMap<String, JsonValue>>, QbwError> {
        let db = self.inner.source();
        let table = match db.table(&origin.table) {
            Ok(table) => table,
            Err(sa17::Error::UnknownTable { .. }) => return Ok(None),
            Err(error) => return Err(convert::sa17_error(error)),
        };
        let columns = sa17::Columns::all(table.definition());
        let address = sa17::RowAddress::new(origin.page, origin.slot);
        let row = table.row(address, &columns).map_err(convert::sa17_error)?;
        Ok(row.as_ref().map(convert::row_to_dict))
    }

    /// Company profile metadata and QuickBooks Desktop product version.
    pub fn info(&self) -> Result<CompanyInfo, QbwError> {
        let info = self.inner.info().map_err(convert::error)?;
        Ok(convert::company_info(info))
    }

    /// Company-wide accounting and feature preferences.
    pub fn preferences(&self) -> Result<Preferences, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let prefs = self.inner.preferences().map_err(convert::error)?;
        Ok(convert::preferences(prefs, &lookup))
    }

    /// Audit-trail users recorded in the company file.
    pub fn users(&self) -> Result<Vec<User>, QbwError> {
        let users = self.inner.users().map_err(convert::error)?;
        Ok(users
            .iter()
            .map(|user| User {
                id: user.id.number(),
                names: user.names.clone(),
            })
            .collect())
    }

    /// Return the fiscal year containing `date` according to company preferences.
    pub fn fiscal_year(&self, date: NaiveDate) -> Result<Period, QbwError> {
        let period = self.inner.fiscal_year(date).map_err(convert::error)?;
        Ok(convert::period(period))
    }

    /// Earliest and latest transaction dates in the file, or `None` if there are none.
    pub fn transaction_days(&self) -> Result<Option<Period>, QbwError> {
        let days = self.inner.transaction_days().map_err(convert::error)?;
        Ok(days.map(convert::period))
    }

    /// Chart of accounts in QuickBooks list order.
    pub fn accounts(
        &self,
        #[bimetal(default = false)] active_only: bool,
    ) -> Result<Vec<Account>, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let accounts = self.inner.accounts().map_err(convert::error)?;
        Ok(accounts
            .walk()
            .filter(|(_, account)| !active_only || account.is_active)
            .map(|(depth, account)| convert::account(account, depth, accounts, &lookup))
            .collect())
    }

    /// Look up one account by its numeric `account_id`.
    pub fn account(&self, account_id: i32) -> Result<Option<Account>, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let accounts = self.inner.accounts().map_err(convert::error)?;
        let id = qbw::AccountId::new(account_id);
        let Some(account) = accounts.get(id) else {
            return Ok(None);
        };
        let depth = convert::account_depth(accounts, id);
        Ok(Some(convert::account(account, depth, accounts, &lookup)))
    }

    /// Look up one account by name or full `Parent:Child` name (case-insensitive).
    pub fn account_named(&self, name: &str) -> Result<Option<Account>, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let accounts = self.inner.accounts().map_err(convert::error)?;
        let Some(account) = accounts.named(name) else {
            return Ok(None);
        };
        let depth = convert::account_depth(accounts, account.id);
        Ok(Some(convert::account(account, depth, accounts, &lookup)))
    }

    /// Compute the ledger balance of `account_id` through `as_of` (or latest state if `None`).
    pub fn account_balance(
        &self,
        account_id: i32,
        as_of: Option<NaiveDate>,
    ) -> Result<Option<AccountBalance>, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let ledger = self.inner.ledger().map_err(convert::error)?;
        let id = qbw::AccountId::new(account_id);
        let moment = convert::as_of(as_of);
        let Some(balance) = ledger.balance(id, moment) else {
            return Ok(None);
        };
        Ok(Some(AccountBalance {
            account_id,
            account_name: lookup.account_name(id),
            amount: lookup.money(balance.amount),
            home_amount: lookup.money(balance.home_amount),
            left_out: convert::left_out(balance.left_out),
        }))
    }

    /// List parties across all name lists or filtered to one `kind`.
    pub fn names(
        &self,
        kind: Option<NameKind>,
        #[bimetal(default = false)] active_only: bool,
    ) -> Result<Vec<NameEntry>, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let names = self.inner.names().map_err(convert::error)?;
        let terms = self.inner.terms().map_err(convert::error)?;
        let out = names
            .iter()
            .filter(|entry| {
                let kind_ok = kind.is_none_or(|wanted| convert::name_kind(entry.kind()) == wanted);
                let active_ok = !active_only || entry.is_active();
                kind_ok && active_ok
            })
            .map(|entry| convert::name_entry(entry, names, terms, &lookup))
            .collect();
        Ok(out)
    }

    /// List customers and jobs.
    pub fn customers(
        &self,
        #[bimetal(default = false)] active_only: bool,
    ) -> Result<Vec<NameEntry>, QbwError> {
        self.names(Some(NameKind::Customer), active_only)
    }

    /// List vendors.
    pub fn vendors(
        &self,
        #[bimetal(default = false)] active_only: bool,
    ) -> Result<Vec<NameEntry>, QbwError> {
        self.names(Some(NameKind::Vendor), active_only)
    }

    /// List employees.
    pub fn employees(
        &self,
        #[bimetal(default = false)] active_only: bool,
    ) -> Result<Vec<NameEntry>, QbwError> {
        self.names(Some(NameKind::Employee), active_only)
    }

    /// List other names.
    pub fn other_names(
        &self,
        #[bimetal(default = false)] active_only: bool,
    ) -> Result<Vec<NameEntry>, QbwError> {
        self.names(Some(NameKind::Other), active_only)
    }

    /// Look up one party by its shared numeric `name_id`.
    pub fn name(&self, name_id: i32) -> Result<Option<NameEntry>, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let names = self.inner.names().map_err(convert::error)?;
        let terms = self.inner.terms().map_err(convert::error)?;
        Ok(names
            .get(qbw::NameId::new(name_id))
            .map(|entry| convert::name_entry(entry, names, terms, &lookup)))
    }

    /// Look up one party by full or own name (case-insensitive).
    pub fn name_named(&self, name: &str) -> Result<Option<NameEntry>, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let names = self.inner.names().map_err(convert::error)?;
        let terms = self.inner.terms().map_err(convert::error)?;
        Ok(names
            .named(name)
            .map(|entry| convert::name_entry(entry, names, terms, &lookup)))
    }

    /// List items (products, services, taxes, discounts) in QuickBooks order.
    pub fn items(
        &self,
        #[bimetal(default = false)] active_only: bool,
    ) -> Result<Vec<Item>, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let items = self.inner.items().map_err(convert::error)?;
        Ok(items
            .iter()
            .filter(|item| !active_only || item.is_active)
            .map(|item| convert::item(item, items, &lookup))
            .collect())
    }

    /// Look up one item by its numeric `item_id`.
    pub fn item(&self, item_id: i32) -> Result<Option<Item>, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let items = self.inner.items().map_err(convert::error)?;
        Ok(items
            .get(qbw::ItemId::new(item_id))
            .map(|item| convert::item(item, items, &lookup)))
    }

    /// Look up one item by full or own name (case-insensitive).
    pub fn item_named(&self, name: &str) -> Result<Option<Item>, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let items = self.inner.items().map_err(convert::error)?;
        Ok(items
            .named(name)
            .map(|item| convert::item(item, items, &lookup)))
    }

    /// Current stock on hand, open orders, commitments, and availability.
    pub fn stock(&self) -> Result<Stock, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let stock = self.inner.stock().map_err(convert::error)?;
        Ok(convert::stock(&stock, &lookup))
    }

    /// Inventory valuation as of `as_of` (or latest state if `None`).
    pub fn valuation(&self, as_of: Option<NaiveDate>) -> Result<Valuation, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let valuation = self
            .inner
            .valuation(convert::as_of(as_of))
            .map_err(convert::error)?;
        Ok(convert::valuation(&valuation, &lookup))
    }

    /// Open sales order (`Side.Receivable`) or purchase order (`Side.Payable`) lines.
    pub fn open_orders(&self, side: Side) -> Result<OpenOrders, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let orders = self
            .inner
            .open_orders(convert::side_to_core(side))
            .map_err(convert::error)?;
        Ok(convert::open_orders(&orders, &lookup))
    }

    /// Currencies defined in the file, with historical exchange rates.
    pub fn currencies(
        &self,
        #[bimetal(default = false)] active_only: bool,
    ) -> Result<Vec<Currency>, QbwError> {
        let currencies = self.inner.currencies().map_err(convert::error)?;
        Ok(currencies
            .iter()
            .filter(|currency| !active_only || currency.is_active)
            .map(convert::currency)
            .collect())
    }

    /// Payment terms defined in the file.
    pub fn terms(
        &self,
        #[bimetal(default = false)] active_only: bool,
    ) -> Result<Vec<Terms>, QbwError> {
        let terms = self.inner.terms().map_err(convert::error)?;
        Ok(terms
            .iter()
            .filter(|entry| !active_only || entry.is_active)
            .map(convert::terms)
            .collect())
    }

    /// Classes defined in the file.
    pub fn classes(
        &self,
        #[bimetal(default = false)] active_only: bool,
    ) -> Result<Vec<ClassEntry>, QbwError> {
        let classes = self.inner.classes().map_err(convert::error)?;
        Ok(classes
            .iter()
            .filter(|entry| !active_only || entry.is_active)
            .map(|entry| ClassEntry {
                id: entry.id.number(),
                name: entry.name.clone(),
                full_name: classes
                    .full_name(entry.id)
                    .unwrap_or(&entry.name)
                    .to_owned(),
                parent_id: entry.parent.map(qbw::ClassId::number),
                is_active: entry.is_active,
            })
            .collect())
    }

    /// Payment methods defined in the file.
    pub fn payment_methods(
        &self,
        #[bimetal(default = false)] active_only: bool,
    ) -> Result<Vec<PaymentMethod>, QbwError> {
        let methods = self.inner.payment_methods().map_err(convert::error)?;
        Ok(methods
            .iter()
            .filter(|entry| !active_only || entry.is_active)
            .map(|entry| PaymentMethod {
                id: entry.id.number(),
                name: entry.name.clone(),
                kind: convert::payment_method_kind(entry.kind),
                is_active: entry.is_active,
            })
            .collect())
    }

    /// Shipping methods defined in the file.
    pub fn ship_methods(
        &self,
        #[bimetal(default = false)] active_only: bool,
    ) -> Result<Vec<ShipMethod>, QbwError> {
        let methods = self.inner.ship_methods().map_err(convert::error)?;
        Ok(methods
            .iter()
            .filter(|entry| !active_only || entry.is_active)
            .map(|entry| ShipMethod {
                id: entry.id.number(),
                name: entry.name.clone(),
                is_active: entry.is_active,
            })
            .collect())
    }

    /// Sales reps defined in the file.
    pub fn sales_reps(
        &self,
        #[bimetal(default = false)] active_only: bool,
    ) -> Result<Vec<SalesRep>, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let reps = self.inner.sales_reps().map_err(convert::error)?;
        Ok(reps
            .iter()
            .filter(|entry| !active_only || entry.is_active)
            .map(|entry| SalesRep {
                id: entry.id.number(),
                initials: entry.initials.clone(),
                name_id: entry.person.map(qbw::NameId::number),
                name: entry.person.and_then(|id| lookup.name(id)),
                is_active: entry.is_active,
            })
            .collect())
    }

    /// Sales tax codes defined in the file.
    pub fn sales_tax_codes(
        &self,
        #[bimetal(default = false)] active_only: bool,
    ) -> Result<Vec<SalesTaxCode>, QbwError> {
        let codes = self.inner.sales_tax_codes().map_err(convert::error)?;
        Ok(codes
            .iter()
            .filter(|entry| !active_only || entry.is_active)
            .map(|entry| SalesTaxCode {
                id: entry.id.number(),
                code: entry.code.clone(),
                description: entry.description.clone(),
                is_taxable: entry.is_taxable,
                is_active: entry.is_active,
            })
            .collect())
    }

    /// Read a complete transaction (header and lines) by `transaction_id`.
    pub fn transaction(&self, transaction_id: i32) -> Result<Option<Transaction>, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let classes = self.inner.classes().map_err(convert::error)?;
        let tx = self
            .inner
            .transaction(qbw::TransactionId::new(transaction_id))
            .map_err(convert::error)?;
        Ok(tx
            .as_ref()
            .map(|tx| convert::transaction(tx, classes, &lookup)))
    }

    /// Read a transaction summary without loading its lines.
    pub fn transaction_summary(
        &self,
        transaction_id: i32,
    ) -> Result<Option<TransactionSummary>, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let summary = self
            .inner
            .transaction_summary(qbw::TransactionId::new(transaction_id))
            .map_err(convert::error)?;
        Ok(summary
            .as_ref()
            .map(|summary| convert::transaction_summary(summary, &lookup)))
    }

    /// Find the latest transaction matching `kind` (for example `"Invoice"`) and `number`.
    pub fn find(&self, kind: &str, number: &str) -> Result<Option<i32>, QbwError> {
        let kind = convert::parse_transaction_kind(kind)?;
        let id = self.inner.find(kind, number).map_err(convert::error)?;
        Ok(id.map(qbw::TransactionId::number))
    }

    /// Find all transaction identifiers matching `kind` and `number`.
    pub fn find_all(&self, kind: &str, number: &str) -> Result<Vec<i32>, QbwError> {
        let kind = convert::parse_transaction_kind(kind)?;
        let ids = self.inner.find_all(kind, number).map_err(convert::error)?;
        Ok(ids.into_iter().map(qbw::TransactionId::number).collect())
    }

    /// Query paginated transaction summaries with optional filter and sort criteria.
    pub fn transactions(
        &self,
        filter: Option<TransactionFilter>,
        #[bimetal(default = 0)] offset: u32,
        #[bimetal(default = 100)] limit: u32,
    ) -> Result<TransactionPage, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let query = convert::transaction_query(filter)?;
        let page = qbw::Page::new(offset as usize, limit as usize);
        let result = self
            .inner
            .transactions(&query, page)
            .map_err(convert::error)?;
        Ok(convert::transaction_page(&result, offset, limit, &lookup))
    }

    /// Paginated account register with opening, running, and closing balances,
    /// or `None` if no such account exists.
    pub fn register(
        &self,
        account_id: i32,
        start: Option<NaiveDate>,
        end: Option<NaiveDate>,
        #[bimetal(default = 0)] offset: u32,
        #[bimetal(default = 100)] limit: u32,
    ) -> Result<Option<Register>, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let period = convert::period_to_core(start, end);
        let page = qbw::Page::new(offset as usize, limit as usize);
        let reg = self
            .inner
            .register(qbw::AccountId::new(account_id), period, page)
            .map_err(convert::error)?;
        Ok(reg.as_ref().map(|reg| convert::register(reg, &lookup)))
    }

    /// All transaction links (payments, credits, deposits, orders) for `transaction_id`.
    pub fn links(&self, transaction_id: i32) -> Result<Option<TransactionLinks>, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let links = self
            .inner
            .links(qbw::TransactionId::new(transaction_id))
            .map_err(convert::error)?;
        Ok(links
            .as_ref()
            .map(|links| convert::transaction_links(links, &lookup)))
    }

    /// Payments, credits, and discounts that settled `transaction_id`.
    pub fn paid_by(&self, transaction_id: i32) -> Result<Option<TransactionLinks>, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let settled = self
            .inner
            .paid_by(qbw::TransactionId::new(transaction_id))
            .map_err(convert::error)?;
        Ok(settled
            .as_ref()
            .map(|settled| convert::settled_links(settled, &lookup)))
    }

    /// Invoices or bills that `transaction_id` paid or credited.
    pub fn applied_to(&self, transaction_id: i32) -> Result<Option<TransactionLinks>, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let settled = self
            .inner
            .applied_to(qbw::TransactionId::new(transaction_id))
            .map_err(convert::error)?;
        Ok(settled
            .as_ref()
            .map(|settled| convert::settled_links(settled, &lookup)))
    }

    /// Balance Sheet as of `as_of` (or latest transaction date if `None`).
    pub fn balance_sheet(&self, as_of: Option<NaiveDate>) -> Result<BalanceSheet, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let sheet = self
            .inner
            .balance_sheet(convert::as_of(as_of))
            .map_err(convert::error)?;
        Ok(convert::balance_sheet(&sheet, &lookup))
    }

    /// Profit and Loss (Income Statement) for `start..=end` (unbounded ends if `None`).
    pub fn profit_and_loss(
        &self,
        start: Option<NaiveDate>,
        end: Option<NaiveDate>,
    ) -> Result<ProfitAndLoss, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let period = convert::period_to_core(start, end);
        let pnl = self.inner.profit_and_loss(period).map_err(convert::error)?;
        Ok(convert::profit_and_loss(&pnl, &lookup))
    }

    /// Trial Balance as of `as_of` (or latest transaction date if `None`).
    pub fn trial_balance(&self, as_of: Option<NaiveDate>) -> Result<TrialBalance, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let tb = self
            .inner
            .trial_balance(convert::as_of(as_of))
            .map_err(convert::error)?;
        Ok(convert::trial_balance(&tb, &lookup))
    }

    /// Open receivables (`Side.Receivable`) or payables (`Side.Payable`).
    pub fn open_items(&self, side: Side, as_of: Option<NaiveDate>) -> Result<OpenItems, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let items = self
            .inner
            .open_items(convert::side_to_core(side), convert::as_of(as_of))
            .map_err(convert::error)?;
        Ok(convert::open_items(&items, &lookup))
    }

    /// Accounts Receivable or Accounts Payable aging summary by customer or vendor.
    pub fn aging(
        &self,
        side: Side,
        as_of: Option<NaiveDate>,
        basis: Option<AgingBasis>,
        #[bimetal(default = 3)] buckets: u16,
        #[bimetal(default = 30)] days: u16,
    ) -> Result<Aging, QbwError> {
        let lookup = convert::Lookup::from_company(&self.inner)?;
        let mut options = qbw::aging::AgingOptions::new(convert::as_of(as_of));
        if let Some(basis) = basis {
            options = options.basis(convert::aging_basis_to_core(basis));
        }
        options =
            options
                .try_buckets(buckets, days)
                .map_err(|error| QbwError::InvalidArgument {
                    message: error.to_string(),
                })?;
        let aging = self
            .inner
            .aging(convert::side_to_core(side), &options)
            .map_err(convert::error)?;
        Ok(convert::aging(&aging, &lookup))
    }

    /// Cross-check computed accounting totals against QuickBooks' stored figures.
    pub fn check(&self) -> Result<CheckReport, QbwError> {
        let report = self.inner.check().map_err(convert::error)?;
        Ok(convert::check_report(&report))
    }

    /// Export the chart of accounts as a zero-copy `pyarrow.Table`.
    #[bimetal(returns(arrow))]
    pub fn accounts_arrow(
        &self,
        #[bimetal(default = false)] active_only: bool,
    ) -> Result<ArrowTable, QbwError> {
        arrow_bridge::accounts_table(&self.inner, active_only)
    }

    /// Export customers, vendors, employees, and other names as a zero-copy `pyarrow.Table`.
    #[bimetal(returns(arrow))]
    pub fn names_arrow(
        &self,
        kind: Option<NameKind>,
        #[bimetal(default = false)] active_only: bool,
    ) -> Result<ArrowTable, QbwError> {
        arrow_bridge::names_table(&self.inner, kind, active_only)
    }

    /// Export items (products, services, taxes) as a zero-copy `pyarrow.Table`.
    #[bimetal(returns(arrow))]
    pub fn items_arrow(
        &self,
        #[bimetal(default = false)] active_only: bool,
    ) -> Result<ArrowTable, QbwError> {
        arrow_bridge::items_table(&self.inner, active_only)
    }

    /// Export transaction headers/summaries as a zero-copy `pyarrow.Table`.
    #[bimetal(returns(arrow))]
    pub fn transactions_arrow(
        &self,
        start: Option<NaiveDate>,
        end: Option<NaiveDate>,
    ) -> Result<ArrowTable, QbwError> {
        arrow_bridge::transactions_table(&self.inner, start, end)
    }

    /// Stream general ledger postings as a lazy zero-copy `pyarrow.RecordBatchReader`,
    /// usable directly by PyArrow, Polars, and DuckDB.
    #[bimetal(returns(arrow))]
    pub fn ledger_arrow(
        &self,
        start: Option<NaiveDate>,
        end: Option<NaiveDate>,
        account_id: Option<i32>,
        #[bimetal(default = 65_536)] batch_size: u32,
    ) -> Result<RecordBatchStream, QbwError> {
        arrow_bridge::ledger_stream(Arc::clone(&self.inner), start, end, account_id, batch_size)
    }

    /// Export open receivables or payables as a zero-copy `pyarrow.Table`.
    #[bimetal(returns(arrow))]
    pub fn open_items_arrow(
        &self,
        side: Side,
        as_of: Option<NaiveDate>,
    ) -> Result<ArrowTable, QbwError> {
        arrow_bridge::open_items_table(&self.inner, side, as_of)
    }

    /// Export current inventory stock rows as a zero-copy `pyarrow.Table`.
    #[bimetal(returns(arrow))]
    pub fn stock_arrow(&self) -> Result<ArrowTable, QbwError> {
        arrow_bridge::stock_table(&self.inner)
    }

    /// Developer-oriented string representation.
    #[bimetal(repr)]
    #[must_use]
    pub fn repr_string(&self) -> String {
        let name = self
            .inner
            .info()
            .map(|info| info.name.as_str())
            .unwrap_or("");
        format!(
            "Company(name={name:?}, tables={})",
            self.inner.source().catalog().len()
        )
    }
}

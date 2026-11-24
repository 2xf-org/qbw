//! Conversions between `qbw` core types and the Bimetal Python boundary types.

use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

use chrono::NaiveDate;
use serde_json::{Number, Value as JsonValue};

use crate::{
    Account, Address, Aging, AgingBasis, AgingBucket, AgingRow, Assets, BalanceSheet,
    CheckComparison, CheckFinding, CheckNotCompared, CheckReport, Cleared, CompanyInfo, Contact,
    Currency, DamageInfo, Equity, ExchangeRate, FileInfo, Item, KeySign, KindCount, LeftOut,
    Liabilities, Money, NameEntry, NameKind, NormalSide, OpenItem, OpenItems, OpenOptions,
    OpenOrderLine, OpenOrders, Origin, PaidState, Period, PostingState, Preferences, Price,
    PriceKind, ProductVersion, ProfitAndLoss, QbwError, Register, RegisterRow, ReportSection,
    SectionAccount, Side, SortDirection, Statement, Stock, StockRow, StoreKey, StoredBalance,
    Terms, TextEncoding, Transaction, TransactionFilter, TransactionLine, TransactionLink,
    TransactionLinks, TransactionOrder, TransactionPage, TransactionSummary, TrialBalance,
    TrialBalanceRow, Valuation, ValuationRow,
};

/// Fast cached lookup tables for resolving currency codes and list full names.
pub(crate) struct Lookup<'company> {
    currencies: HashMap<qbw::CurrencyId, String>,
    accounts: &'company qbw::List<qbw::accounts::Account>,
    names: &'company qbw::names::Names,
    items: &'company qbw::List<qbw::items::Item>,
}

impl<'company> Lookup<'company> {
    pub(crate) fn from_company(company: &'company qbw::Company) -> Result<Self, QbwError> {
        let currencies_list = company.currencies().map_err(error)?;
        let currencies = currencies_list
            .iter()
            .map(|currency| (currency.id, currency.code.clone()))
            .collect();
        let accounts = company.accounts().map_err(error)?;
        let names = company.names().map_err(error)?;
        let items = company.items().map_err(error)?;
        Ok(Self {
            currencies,
            accounts,
            names,
            items,
        })
    }

    pub(crate) fn currency_code(&self, id: qbw::CurrencyId) -> Option<String> {
        self.currencies.get(&id).cloned()
    }

    pub(crate) fn money(&self, money: qbw::Money) -> Money {
        Money {
            amount: money.to_string(),
            currency_id: money.currency().number(),
            currency_code: self.currency_code(money.currency()),
        }
    }

    pub(crate) fn price(&self, price: qbw::Price) -> Price {
        match price {
            qbw::Price::Each(money) => Price {
                kind: PriceKind::Each,
                amount: money.to_string(),
                currency_id: Some(money.currency().number()),
                currency_code: self.currency_code(money.currency()),
            },
            qbw::Price::Percent(percent) => Price {
                kind: PriceKind::Percent,
                amount: percent.to_string(),
                currency_id: None,
                currency_code: None,
            },
            _ => Price {
                kind: PriceKind::Each,
                amount: String::new(),
                currency_id: None,
                currency_code: None,
            },
        }
    }

    pub(crate) fn stored_balance(&self, stored: qbw::accounts::StoredBalance) -> StoredBalance {
        StoredBalance {
            amount: self.money(stored.amount),
            home_amount: stored.home_amount.map(|amount| self.money(amount)),
        }
    }

    pub(crate) fn account(&self, id: qbw::AccountId) -> Option<&qbw::accounts::Account> {
        self.accounts.get(id)
    }

    pub(crate) fn account_name(&self, id: qbw::AccountId) -> Option<String> {
        self.accounts.full_name(id).map(str::to_owned)
    }

    pub(crate) fn name(&self, id: qbw::NameId) -> Option<String> {
        self.names.full_name(id).map(str::to_owned)
    }

    pub(crate) fn item_name(&self, id: qbw::ItemId) -> Option<String> {
        self.items.full_name(id).map(str::to_owned)
    }
}

pub(crate) fn open_options(options: Option<OpenOptions>) -> sa17::OpenOptions {
    let mut built = sa17::OpenOptions::new();
    let Some(options) = options else {
        return built;
    };
    if let Some(key) = options.key {
        built = built.key(store_key_to_core(key));
    }
    if let Some(longest) = options.longest_value {
        built = built.longest_value(longest);
    }
    built
}

pub(crate) fn store_key_to_core(key: StoreKey) -> sa17::StoreKey {
    let sign = match key.sign {
        KeySign::Positive => sa17::KeySign::Positive,
        KeySign::Negative => sa17::KeySign::Negative,
    };
    sa17::StoreKey::new(key.mask, key.addend, sign)
}

pub(crate) fn store_key_from_core(key: sa17::StoreKey) -> StoreKey {
    let sign = match key.sign() {
        sa17::KeySign::Positive => KeySign::Positive,
        sa17::KeySign::Negative => KeySign::Negative,
    };
    StoreKey {
        mask: key.mask(),
        addend: key.addend(),
        sign,
    }
}

pub(crate) fn file_info(db: &sa17::Database) -> FileInfo {
    let store = db.store();
    let superblock = store.superblock();
    let identity = store.identity();
    let encoding = match store.text_encoding() {
        sa17::TextEncoding::Windows1252 => TextEncoding::Windows1252,
        sa17::TextEncoding::Utf8 | _ => TextEncoding::Utf8,
    };
    FileInfo {
        format_version: superblock.version().to_string(),
        page_size: store.page_size() as u32,
        page_count: store.page_count(),
        encoding,
        table_count: db.catalog().len() as u32,
        lsn: superblock.lsn(),
        digest: identity.to_string(),
    }
}

pub(crate) fn origin(origin: qbw::Origin) -> Origin {
    let (page, slot) = match origin.row {
        sa17::RowId::Stored(address) => (address.page(), address.slot()),
        _ => (0, 0),
    };
    Origin {
        table: origin.table.to_owned(),
        page,
        slot,
    }
}

pub(crate) fn left_out(left_out: qbw::LeftOut) -> LeftOut {
    LeftOut {
        rows: left_out.rows,
        pages: left_out.pages,
        unread_tables: left_out.unread_tables,
    }
}

pub(crate) fn damage_info(damage: &qbw::Damage) -> DamageInfo {
    DamageInfo {
        table: damage.table.to_owned(),
        row: damage.row.map(|row| row.to_string()),
        problem: damage.problem.to_string(),
    }
}

pub(crate) fn address(address: &qbw::Address) -> Address {
    Address {
        printed: address.printed.clone(),
        street: address.street.clone(),
        city: address.city.clone(),
        state: address.state.clone(),
        postal_code: address.postal_code.clone(),
        country: address.country.clone(),
    }
}

pub(crate) fn period(period: qbw::Period) -> Period {
    Period {
        start: period.first(),
        end: period.last(),
    }
}

pub(crate) fn period_to_core(start: Option<NaiveDate>, end: Option<NaiveDate>) -> qbw::Period {
    match (start, end) {
        (Some(start), Some(end)) => qbw::Period::between(start, end),
        (Some(start), None) => qbw::Period::starting(start),
        (None, Some(end)) => qbw::Period::until(end),
        (None, None) => qbw::Period::all(),
    }
}

pub(crate) fn as_of(date: Option<NaiveDate>) -> qbw::AsOf {
    date.map_or(qbw::AsOf::Latest, qbw::AsOf::Date)
}

pub(crate) fn as_of_from_core(moment: qbw::AsOf) -> Option<NaiveDate> {
    match moment {
        qbw::AsOf::Date(date) => Some(date),
        qbw::AsOf::Latest | _ => None,
    }
}

pub(crate) fn side_to_core(side: Side) -> qbw::Side {
    match side {
        Side::Receivable => qbw::Side::Receivable,
        Side::Payable => qbw::Side::Payable,
    }
}

pub(crate) fn side_from_core(side: qbw::Side) -> Side {
    match side {
        qbw::Side::Receivable => Side::Receivable,
        qbw::Side::Payable => Side::Payable,
        _ => Side::Receivable,
    }
}

pub(crate) fn aging_basis_to_core(basis: AgingBasis) -> qbw::settings::AgingBasis {
    match basis {
        AgingBasis::DueDate => qbw::settings::AgingBasis::DueDate,
        AgingBasis::TransactionDate => qbw::settings::AgingBasis::TransactionDate,
    }
}

pub(crate) fn aging_basis_from_core(basis: qbw::settings::AgingBasis) -> AgingBasis {
    match basis {
        qbw::settings::AgingBasis::DueDate => AgingBasis::DueDate,
        qbw::settings::AgingBasis::TransactionDate => AgingBasis::TransactionDate,
        _ => AgingBasis::DueDate,
    }
}

pub(crate) fn company_info(info: &qbw::settings::CompanyInfo) -> CompanyInfo {
    CompanyInfo {
        name: info.name.clone(),
        legal_name: info.legal_name.clone(),
        legal_address: address(&info.legal_address),
        address_lines: info.address_lines.clone(),
        phone: info.phone.clone(),
        fax: info.fax.clone(),
        email: info.email.clone(),
        website: info.website.clone(),
        started: info.started,
        created: info.created.and_then(qbw::UnixTime::to_utc),
        version: ProductVersion {
            year: info.version.year,
            update: info.version.update,
            edition: info.version.edition.clone(),
            newest_used: info.version.newest_used.clone(),
        },
        left_out: left_out(info.left_out),
    }
}

pub(crate) fn preferences(prefs: &qbw::settings::Preferences, lookup: &Lookup<'_>) -> Preferences {
    Preferences {
        home_currency_id: prefs.home_currency.number(),
        home_currency_code: lookup.currency_code(prefs.home_currency),
        uses_multicurrency: prefs.uses_multicurrency,
        fiscal_year_start: prefs.fiscal_year_start.number_from_month(),
        tax_year_start: prefs.tax_year_start.map(|month| month.number_from_month()),
        aging_basis: aging_basis_from_core(prefs.aging_basis),
        uses_account_numbers: prefs.uses_account_numbers,
        uses_classes: prefs.uses_classes,
        uses_inventory: prefs.uses_inventory,
        uses_sales_tax: prefs.uses_sales_tax,
        uses_sales_orders: prefs.uses_sales_orders,
        uses_sites: prefs.uses_sites,
        uses_bins: prefs.uses_bins,
        uses_serial_numbers: prefs.uses_serial_numbers,
        available_less_sales_orders: prefs.available_less_sales_orders,
        available_less_builds: prefs.available_less_builds,
        default_tax_item_id: prefs.default_tax_item.map(qbw::ItemId::number),
        closing_date: prefs.closing_date.clone(),
        left_out: left_out(prefs.left_out),
    }
}

pub(crate) fn account_depth(
    accounts: &qbw::List<qbw::accounts::Account>,
    id: qbw::AccountId,
) -> usize {
    let mut depth = 0usize;
    let mut current = accounts.get(id).and_then(|account| account.parent);
    while let Some(parent) = current {
        if depth > 64 {
            break;
        }
        depth += 1;
        current = accounts.get(parent).and_then(|account| account.parent);
    }
    depth
}

pub(crate) fn account(
    account: &qbw::accounts::Account,
    depth: usize,
    accounts: &qbw::List<qbw::accounts::Account>,
    lookup: &Lookup<'_>,
) -> Account {
    let normal_side = account.account_type.normal_side().map(|side| match side {
        qbw::accounts::NormalSide::Debit => NormalSide::Debit,
        qbw::accounts::NormalSide::Credit => NormalSide::Credit,
        _ => NormalSide::Debit,
    });
    let statement = account.account_type.statement().map(|stmt| match stmt {
        qbw::accounts::Statement::BalanceSheet => Statement::BalanceSheet,
        qbw::accounts::Statement::ProfitAndLoss => Statement::ProfitAndLoss,
    });
    Account {
        id: account.id.number(),
        name: account.name.clone(),
        full_name: accounts
            .full_name(account.id)
            .unwrap_or(&account.name)
            .to_owned(),
        parent_id: account.parent.map(qbw::AccountId::number),
        depth: depth as u32,
        account_type: account.account_type.to_string(),
        account_type_code: account.account_type.code(),
        normal_side,
        statement,
        special: account.special.map(|special| format!("{special:?}")),
        number: account.number.clone(),
        description: account.description.clone(),
        currency_id: account.currency.number(),
        currency_code: lookup.currency_code(account.currency),
        tax_line: account.tax_line,
        is_active: account.is_active,
        stored: account.stored.map(|stored| lookup.stored_balance(stored)),
        origin: origin(account.origin),
    }
}

pub(crate) fn name_kind(kind: qbw::names::NameKind) -> NameKind {
    match kind {
        qbw::names::NameKind::Customer => NameKind::Customer,
        qbw::names::NameKind::Vendor => NameKind::Vendor,
        qbw::names::NameKind::Employee => NameKind::Employee,
        qbw::names::NameKind::Other => NameKind::Other,
        _ => NameKind::Other,
    }
}

pub(crate) fn contact(contact: &qbw::names::Contact) -> Contact {
    Contact {
        id: contact.id.as_str().to_owned(),
        company: contact.company.clone(),
        salutation: contact.salutation.clone(),
        first_name: contact.first_name.clone(),
        middle_name: contact.middle_name.clone(),
        last_name: contact.last_name.clone(),
        job_title: contact.job_title.clone(),
        contact: contact.contact.clone(),
        alt_contact: contact.alt_contact.clone(),
        phone: contact.phone.clone(),
        alt_phone: contact.alt_phone.clone(),
        mobile: contact.mobile.clone(),
        fax: contact.fax.clone(),
        email: contact.email.clone(),
        cc: contact.cc.clone(),
        account_number: contact.account_number.clone(),
        address: address(&contact.address),
    }
}

pub(crate) fn name_entry(
    entry: qbw::names::Name<'_>,
    names: &qbw::names::Names,
    terms: &qbw::List<qbw::lists::Terms>,
    lookup: &Lookup<'_>,
) -> NameEntry {
    let id = entry.id();
    let full_name = names.full_name(id).unwrap_or(entry.name()).to_owned();
    let contact_model = entry.contact().map(contact);
    match entry {
        qbw::names::Name::Customer(customer) => NameEntry {
            id: customer.id.number(),
            kind: NameKind::Customer,
            name: customer.name.clone(),
            full_name,
            parent_id: customer.parent.map(qbw::NameId::number),
            is_active: customer.is_active,
            currency_id: Some(customer.currency.number()),
            currency_code: lookup.currency_code(customer.currency),
            terms_id: customer.terms.map(qbw::TermsId::number),
            terms_name: customer
                .terms
                .and_then(|id| terms.get(id))
                .map(|t| t.name.clone()),
            credit_limit: customer.credit_limit.map(|money| lookup.money(money)),
            is_1099: false,
            name_on_check: None,
            resale_number: customer.resale_number.clone(),
            hired: None,
            opened: customer.opened,
            stored: customer.stored.map(|stored| lookup.stored_balance(stored)),
            contact: contact_model,
            list_id: customer.list_id.clone(),
            origin: origin(customer.origin),
        },
        qbw::names::Name::Vendor(vendor) => NameEntry {
            id: vendor.id.number(),
            kind: NameKind::Vendor,
            name: vendor.name.clone(),
            full_name,
            parent_id: None,
            is_active: vendor.is_active,
            currency_id: Some(vendor.currency.number()),
            currency_code: lookup.currency_code(vendor.currency),
            terms_id: vendor.terms.map(qbw::TermsId::number),
            terms_name: vendor
                .terms
                .and_then(|id| terms.get(id))
                .map(|t| t.name.clone()),
            credit_limit: vendor.credit_limit.map(|money| lookup.money(money)),
            is_1099: vendor.is_1099,
            name_on_check: vendor.name_on_check.clone(),
            resale_number: None,
            hired: None,
            opened: vendor.opened,
            stored: vendor.stored.map(|stored| lookup.stored_balance(stored)),
            contact: contact_model,
            list_id: vendor.list_id.clone(),
            origin: origin(vendor.origin),
        },
        qbw::names::Name::Employee(employee) => NameEntry {
            id: employee.id.number(),
            kind: NameKind::Employee,
            name: employee.name.clone(),
            full_name,
            parent_id: None,
            is_active: employee.is_active,
            currency_id: None,
            currency_code: None,
            terms_id: None,
            terms_name: None,
            credit_limit: None,
            is_1099: false,
            name_on_check: employee.name_on_check.clone(),
            resale_number: None,
            hired: employee.hired,
            opened: None,
            stored: None,
            contact: contact_model,
            list_id: employee.list_id.clone(),
            origin: origin(employee.origin),
        },
        qbw::names::Name::Other(other) => NameEntry {
            id: other.id.number(),
            kind: NameKind::Other,
            name: other.name.clone(),
            full_name,
            parent_id: None,
            is_active: other.is_active,
            currency_id: None,
            currency_code: None,
            terms_id: None,
            terms_name: None,
            credit_limit: None,
            is_1099: false,
            name_on_check: None,
            resale_number: None,
            hired: None,
            opened: other.opened,
            stored: None,
            contact: contact_model,
            list_id: other.list_id.clone(),
            origin: origin(other.origin),
        },
        _ => NameEntry {
            id: id.number(),
            kind: NameKind::Other,
            name: entry.name().to_owned(),
            full_name,
            parent_id: None,
            is_active: entry.is_active(),
            currency_id: None,
            currency_code: None,
            terms_id: None,
            terms_name: None,
            credit_limit: None,
            is_1099: false,
            name_on_check: None,
            resale_number: None,
            hired: None,
            opened: None,
            stored: None,
            contact: contact_model,
            list_id: None,
            origin: origin(entry.origin()),
        },
    }
}

pub(crate) fn item(
    item: &qbw::items::Item,
    items: &qbw::List<qbw::items::Item>,
    lookup: &Lookup<'_>,
) -> Item {
    Item {
        id: item.id.number(),
        name: item.name.clone(),
        full_name: items.full_name(item.id).unwrap_or(&item.name).to_owned(),
        parent_id: item.parent.map(qbw::ItemId::number),
        kind: item.kind.to_string(),
        kind_code: item.kind.code(),
        is_stocked: item.kind.is_stocked(),
        is_active: item.is_active,
        description: item.sales.description.clone(),
        price: item.sales.price.map(|price| lookup.price(price)),
        income_account_id: item.sales.account.map(qbw::AccountId::number),
        income_account_name: item.sales.account.and_then(|id| lookup.account_name(id)),
        purchase_description: item
            .purchase
            .as_ref()
            .and_then(|purchase| purchase.description.clone()),
        cost: item
            .purchase
            .as_ref()
            .and_then(|purchase| purchase.cost)
            .map(|money| lookup.money(money)),
        expense_account_id: item
            .purchase
            .as_ref()
            .and_then(|purchase| purchase.account)
            .map(qbw::AccountId::number),
        expense_account_name: item
            .purchase
            .as_ref()
            .and_then(|purchase| purchase.account)
            .and_then(|id| lookup.account_name(id)),
        preferred_vendor_id: item
            .purchase
            .as_ref()
            .and_then(|purchase| purchase.vendor)
            .map(qbw::NameId::number),
        preferred_vendor_name: item
            .purchase
            .as_ref()
            .and_then(|purchase| purchase.vendor)
            .and_then(|id| lookup.name(id)),
        asset_account_id: item
            .stock
            .as_ref()
            .and_then(|stock| stock.asset_account)
            .map(qbw::AccountId::number),
        asset_account_name: item
            .stock
            .as_ref()
            .and_then(|stock| stock.asset_account)
            .and_then(|id| lookup.account_name(id)),
        reorder_point: item
            .stock
            .as_ref()
            .and_then(|stock| stock.reorder_point)
            .map(|qty| qty.to_string()),
        build_point: item
            .stock
            .as_ref()
            .and_then(|stock| stock.build_point)
            .map(|qty| qty.to_string()),
        part_number: item.part_number.clone(),
        on_hand: item.stored.map(|stored| stored.on_hand.to_string()),
        stored_value: item
            .stored
            .and_then(|stored| stored.value)
            .map(|money| lookup.money(money)),
        stored_average_cost: item
            .stored
            .and_then(|stored| stored.average_cost)
            .map(|money| lookup.money(money)),
        list_id: item.list_id.clone(),
        origin: origin(item.origin),
    }
}

pub(crate) fn stock(stock: &qbw::inventory::Stock, lookup: &Lookup<'_>) -> Stock {
    Stock {
        rows: stock
            .rows
            .iter()
            .map(|row| StockRow {
                item_id: row.item.number(),
                item_name: lookup.item_name(row.item),
                on_hand: row.on_hand.to_string(),
                value: lookup.money(row.value),
                average_cost: row.average_cost.map(|money| lookup.money(money)),
                counted: row.counted.to_string(),
                on_order: row.on_order.to_string(),
                on_sales_orders: row.on_sales_orders.to_string(),
                to_be_built: row.to_be_built.to_string(),
                required_for_builds: row.required_for_builds.to_string(),
                available: row.available.to_string(),
                reorder_point: row.reorder_point.map(|qty| qty.to_string()),
                needs_reorder: row.needs_reorder(),
            })
            .collect(),
        value: lookup.money(stock.value),
        left_out: left_out(stock.left_out),
    }
}

pub(crate) fn valuation(valuation: &qbw::inventory::Valuation, lookup: &Lookup<'_>) -> Valuation {
    Valuation {
        as_of: as_of_from_core(valuation.as_of),
        rows: valuation
            .rows
            .iter()
            .map(|row| ValuationRow {
                item_id: row.item.number(),
                item_name: lookup.item_name(row.item),
                on_hand: row.on_hand.to_string(),
                value: lookup.money(row.value),
                average_cost: row.average_cost.map(|money| lookup.money(money)),
            })
            .collect(),
        value: lookup.money(valuation.value),
        left_out: left_out(valuation.left_out),
    }
}

pub(crate) fn open_orders(orders: &qbw::orders::OpenOrders, lookup: &Lookup<'_>) -> OpenOrders {
    OpenOrders {
        side: side_from_core(orders.side),
        lines: orders
            .lines
            .iter()
            .map(|line| OpenOrderLine {
                transaction_id: line.transaction.number(),
                date: line.date,
                number: line.number.clone(),
                name_id: line.name.map(qbw::NameId::number),
                name: line.name.and_then(|id| lookup.name(id)),
                is_order_done: line.is_order_done,
                item_id: line.line.item.map(qbw::ItemId::number),
                item_name: line.line.item.and_then(|id| lookup.item_name(id)),
                description: line.line.description.clone(),
                ordered: line.ordered.to_string(),
                filled: line.filled.to_string(),
                open: line.open.to_string(),
                open_amount: line.open_amount.map(|money| lookup.money(money)),
            })
            .collect(),
        left_out: left_out(orders.left_out),
    }
}

pub(crate) fn currency(currency: &qbw::lists::Currency) -> Currency {
    Currency {
        id: currency.id.number(),
        name: currency.name.clone(),
        code: currency.code.clone(),
        symbol: currency.symbol.clone(),
        decimal_places: currency.decimal_places,
        is_active: currency.is_active,
        rates: currency
            .rates
            .iter()
            .map(|rate| ExchangeRate {
                as_of: rate.as_of,
                rate: f64::from(rate.rate),
            })
            .collect(),
    }
}

pub(crate) fn payment_method_kind(kind: qbw::lists::PaymentMethodKind) -> String {
    match kind {
        qbw::lists::PaymentMethodKind::CashOrOther => "Cash or Other".to_owned(),
        qbw::lists::PaymentMethodKind::Check => "Check".to_owned(),
        qbw::lists::PaymentMethodKind::AmericanExpress => "American Express".to_owned(),
        qbw::lists::PaymentMethodKind::Discover => "Discover".to_owned(),
        qbw::lists::PaymentMethodKind::MasterCard => "MasterCard".to_owned(),
        qbw::lists::PaymentMethodKind::Visa => "Visa".to_owned(),
        qbw::lists::PaymentMethodKind::DebitCard => "Debit Card".to_owned(),
        qbw::lists::PaymentMethodKind::GiftCard => "Gift Card".to_owned(),
        qbw::lists::PaymentMethodKind::ElectronicCheck => "Electronic Check".to_owned(),
        qbw::lists::PaymentMethodKind::Other(code) => format!("Other ({code})"),
        _ => "Other".to_owned(),
    }
}

pub(crate) fn terms(entry: &qbw::lists::Terms) -> Terms {
    let kind = match entry.kind {
        qbw::lists::TermsKind::Standard => "Standard".to_owned(),
        qbw::lists::TermsKind::DateDriven => "Date Driven".to_owned(),
        qbw::lists::TermsKind::Other(code) => format!("Other ({code})"),
        _ => "Other".to_owned(),
    };
    Terms {
        id: entry.id.number(),
        name: entry.name.clone(),
        kind,
        due_days: entry.due_days,
        discount_days: entry.discount_days,
        discount_percent: entry.discount_percent.map(|pct| pct.to_string()),
        due_day: entry.due_day,
        next_month_days: entry.next_month_days,
        discount_day: entry.discount_day,
        is_active: entry.is_active,
    }
}

pub(crate) fn cleared(value: qbw::transactions::Cleared) -> Cleared {
    match value {
        qbw::transactions::Cleared::NotCleared => Cleared::NotCleared,
        qbw::transactions::Cleared::Ticked => Cleared::Cleared,
        qbw::transactions::Cleared::Reconciled => Cleared::Reconciled,
        qbw::transactions::Cleared::MatchedByBankFeed => Cleared::Matched,
        _ => Cleared::NotCleared,
    }
}

pub(crate) fn posting_state(value: qbw::transactions::PostingState) -> PostingState {
    match value {
        qbw::transactions::PostingState::Posted => PostingState::Posted,
        qbw::transactions::PostingState::NonPosting | _ => PostingState::NonPosting,
    }
}

pub(crate) fn parse_transaction_kind(
    text: &str,
) -> Result<qbw::transactions::TransactionKind, QbwError> {
    qbw::transactions::TransactionKind::from_str(text).map_err(|error| QbwError::InvalidArgument {
        message: error.to_string(),
    })
}

pub(crate) fn transaction_query(
    filter: Option<TransactionFilter>,
) -> Result<qbw::query::TransactionQuery, QbwError> {
    let mut query = qbw::query::TransactionQuery::new();
    let Some(filter) = filter else {
        return Ok(query);
    };
    if let Some(kinds) = filter.kinds {
        let parsed: Result<Vec<qbw::transactions::TransactionKind>, QbwError> = kinds
            .iter()
            .map(|kind| parse_transaction_kind(kind))
            .collect();
        query = query.kinds(parsed?);
    }
    if filter.start.is_some() || filter.end.is_some() {
        query = query.dated(period_to_core(filter.start, filter.end));
    }
    if let Some(name_id) = filter.name_id {
        query = query.name(qbw::NameId::new(name_id));
        if filter.with_jobs {
            query = query.with_jobs();
        }
    }
    if let Some(account_id) = filter.account_id {
        query = query.account(qbw::AccountId::new(account_id));
    }
    if let Some(account_id) = filter.posts_to_account_id {
        query = query.posts_to(qbw::AccountId::new(account_id));
    }
    if let Some(number) = filter.number {
        query = query.number(&number);
    }
    if let Some(paid) = filter.paid {
        let state = match paid {
            PaidState::Paid => qbw::query::PaidState::Paid,
            PaidState::Unpaid => qbw::query::PaidState::Unpaid,
        };
        query = query.paid(state);
    }
    if let Some(text) = filter.text {
        query = query.text(&text);
    }
    if filter.order.is_some() || filter.direction.is_some() {
        let order = match filter.order.unwrap_or(TransactionOrder::Date) {
            TransactionOrder::Date => qbw::query::TransactionOrder::Date,
            TransactionOrder::Entered => qbw::query::TransactionOrder::Entered,
            TransactionOrder::Number => qbw::query::TransactionOrder::Number,
            TransactionOrder::Name => qbw::query::TransactionOrder::Name,
            TransactionOrder::Total => qbw::query::TransactionOrder::Total,
            TransactionOrder::DueDate => qbw::query::TransactionOrder::DueDate,
        };
        let direction = match filter.direction.unwrap_or(SortDirection::Descending) {
            SortDirection::Ascending => qbw::query::Direction::Ascending,
            SortDirection::Descending => qbw::query::Direction::Descending,
        };
        query = query.order(order, direction);
    }
    Ok(query)
}

pub(crate) fn transaction(
    tx: &qbw::transactions::Transaction,
    classes: &qbw::List<qbw::lists::Class>,
    lookup: &Lookup<'_>,
) -> Transaction {
    let account_id = tx.account();
    let lines = tx
        .lines
        .iter()
        .map(|line| TransactionLine {
            id: line.id.number(),
            is_own: line.is_own,
            role: line.role.to_string(),
            item_id: line.item.map(qbw::ItemId::number),
            item_name: line.item.and_then(|id| lookup.item_name(id)),
            account_id: line.account.map(qbw::AccountId::number),
            account_name: line.account.and_then(|id| lookup.account_name(id)),
            name_id: line.name.map(qbw::NameId::number),
            name: line.name.and_then(|id| lookup.name(id)),
            class_id: line.class.map(qbw::ClassId::number),
            class_name: line
                .class
                .and_then(|id| classes.full_name(id))
                .map(str::to_owned),
            description: line.description.clone(),
            quantity: line.quantity.map(|qty| qty.to_string()),
            rate: line.rate.map(|price| lookup.price(price)),
            amount: line.amount.map(|money| lookup.money(money)),
            home_amount: line.home_amount.map(|money| lookup.money(money)),
            debit: line.debit().map(|money| lookup.money(money)),
            credit: line.credit().map(|money| lookup.money(money)),
            service_date: line.service_date,
            cleared: cleared(line.cleared),
            reconciled_on: line.reconciled_on,
            origin: origin(line.origin),
        })
        .collect();
    Transaction {
        id: tx.id.number(),
        kind: tx.kind.to_string(),
        kind_code: tx.kind.code(),
        date: tx.date,
        number: tx.number.clone(),
        name_id: tx.name.map(qbw::NameId::number),
        name: tx.name.and_then(|id| lookup.name(id)),
        account_id: account_id.map(qbw::AccountId::number),
        account_name: account_id.and_then(|id| lookup.account_name(id)),
        memo: tx.memo.clone(),
        currency_id: tx.currency.number(),
        currency_code: lookup.currency_code(tx.currency),
        exchange_rate: f64::from(tx.exchange_rate),
        total: tx.total.map(|money| lookup.money(money)),
        home_total: tx.home_total.map(|money| lookup.money(money)),
        subtotal: tx.subtotal().map(|money| lookup.money(money)),
        tax_total: tx.tax_total().map(|money| lookup.money(money)),
        due_date: tx.due_date,
        open: tx.open().map(|money| lookup.money(money)),
        is_paid: tx.is_paid(),
        is_voided: tx.is_voided(),
        posting: posting_state(tx.posting),
        terms_name: tx.terms_name.clone(),
        sales_rep_initials: tx.sales_rep_initials.clone(),
        lines,
        damage: tx.damage.iter().map(damage_info).collect(),
        origin: origin(tx.origin),
    }
}

pub(crate) fn transaction_summary(
    summary: &qbw::query::TransactionSummary,
    lookup: &Lookup<'_>,
) -> TransactionSummary {
    TransactionSummary {
        id: summary.id.number(),
        kind: summary.kind.to_string(),
        kind_code: summary.kind.code(),
        date: summary.date,
        number: summary.number.clone(),
        name_id: summary.name.map(qbw::NameId::number),
        name: summary.name.and_then(|id| lookup.name(id)),
        account_id: summary.account.map(qbw::AccountId::number),
        account_name: summary.account.and_then(|id| lookup.account_name(id)),
        other_account_id: summary.other_account.map(qbw::AccountId::number),
        other_account_name: summary.other_account.and_then(|id| lookup.account_name(id)),
        memo: summary.memo.clone(),
        currency_id: summary.currency.number(),
        currency_code: lookup.currency_code(summary.currency),
        amount: summary.amount.map(|money| lookup.money(money)),
        total: summary.total.map(|money| lookup.money(money)),
        home_total: summary.home_total.map(|money| lookup.money(money)),
        due_date: summary.due_date,
        open: summary.open().map(|money| lookup.money(money)),
        is_paid: summary.is_paid(),
        is_voided: summary.is_voided(),
        posting: posting_state(summary.posting),
        origin: origin(summary.origin),
        left_out: left_out(summary.left_out),
    }
}

pub(crate) fn transaction_page(
    page: &qbw::query::TransactionPage,
    offset: u32,
    limit: u32,
    lookup: &Lookup<'_>,
) -> TransactionPage {
    TransactionPage {
        rows: page
            .rows
            .iter()
            .map(|row| transaction_summary(row, lookup))
            .collect(),
        total_count: page.row_count as u32,
        kinds: page
            .kinds
            .iter()
            .map(|entry| KindCount {
                kind: entry.kind.to_string(),
                kind_code: entry.kind.code(),
                count: entry.count as u32,
            })
            .collect(),
        offset,
        limit,
        left_out: left_out(page.left_out),
    }
}

pub(crate) fn register(reg: &qbw::registers::Register, lookup: &Lookup<'_>) -> Register {
    Register {
        account_id: reg.account.number(),
        account_name: lookup.account_name(reg.account),
        opening_balance: lookup.money(reg.opening.amount),
        closing_balance: lookup.money(reg.closing.amount),
        rows: reg
            .rows
            .iter()
            .map(|row| {
                let other_side = match row.other_side {
                    qbw::OtherSide::Account(id) => lookup.account_name(id),
                    qbw::OtherSide::Split => Some("Split".to_owned()),
                    qbw::OtherSide::None | _ => None,
                };
                RegisterRow {
                    date: row.date,
                    kind: row.kind.to_string(),
                    number: row.number.clone(),
                    name_id: row.name.map(qbw::NameId::number),
                    name: row.name.and_then(|id| lookup.name(id)),
                    other_side,
                    memo: row.memo.clone(),
                    cleared: cleared(row.cleared),
                    reconciled_on: row.reconciled_on,
                    amount: lookup.money(row.amount),
                    balance: lookup.money(row.balance),
                    transaction_id: row.transaction.number(),
                    line_id: row.line.number(),
                    origin: origin(row.origin),
                }
            })
            .collect(),
        total_count: reg.row_count as u32,
        left_out: left_out(reg.left_out),
    }
}

fn convert_link(link: &qbw::links::Link, lookup: &Lookup<'_>) -> TransactionLink {
    TransactionLink {
        transaction_id: link.other.id.number(),
        kind: link.other.kind.to_string(),
        date: link.other.date,
        number: link.other.number.clone(),
        name_id: link.other.name.map(qbw::NameId::number),
        name: link.other.name.and_then(|id| lookup.name(id)),
        relation: link.relation.to_string(),
        amount: link.amount.map(|money| lookup.money(money)),
        quantity: link.quantity.map(|qty| qty.to_string()),
    }
}

pub(crate) fn transaction_links(
    links: &qbw::links::Links,
    lookup: &Lookup<'_>,
) -> TransactionLinks {
    TransactionLinks {
        transaction_id: links.transaction.number(),
        currency_id: links.currency.number(),
        currency_code: lookup.currency_code(links.currency),
        links: links
            .links
            .iter()
            .map(|link| convert_link(link, lookup))
            .collect(),
        total: None,
        left_out: left_out(links.left_out),
    }
}

pub(crate) fn settled_links(
    settled: &qbw::links::Settled,
    lookup: &Lookup<'_>,
) -> TransactionLinks {
    let currency = settled.total.currency();
    TransactionLinks {
        transaction_id: settled.transaction.number(),
        currency_id: currency.number(),
        currency_code: lookup.currency_code(currency),
        links: settled
            .links
            .iter()
            .map(|link| convert_link(link, lookup))
            .collect(),
        total: Some(lookup.money(settled.total)),
        left_out: left_out(settled.left_out),
    }
}

fn section_account(node: &qbw::reports::SectionAccount, lookup: &Lookup<'_>) -> SectionAccount {
    let acc = lookup.account(node.account);
    SectionAccount {
        account_id: node.account.number(),
        account_name: acc.map(|account| account.name.clone()),
        account_full_name: lookup.account_name(node.account),
        account_number: acc.and_then(|account| account.number.clone()),
        amount: lookup.money(node.amount),
        total: lookup.money(node.total),
        children: node
            .children
            .iter()
            .map(|child| section_account(child, lookup))
            .collect(),
    }
}

fn report_section(section: &qbw::reports::Section, lookup: &Lookup<'_>) -> ReportSection {
    ReportSection {
        account_type: section.account_type.to_string(),
        accounts: section
            .accounts
            .iter()
            .map(|node| section_account(node, lookup))
            .collect(),
        total: lookup.money(section.total),
    }
}

pub(crate) fn balance_sheet(
    sheet: &qbw::reports::BalanceSheet,
    lookup: &Lookup<'_>,
) -> BalanceSheet {
    let (earlier_profit, retained_earnings_account_id) = match sheet.equity.earlier_profit {
        qbw::reports::EarlierProfit::InAccount { account, amount } => {
            (lookup.money(amount), Some(account.number()))
        }
        qbw::reports::EarlierProfit::Alone(amount) => (lookup.money(amount), None),
        other => (lookup.money(other.amount()), None),
    };
    BalanceSheet {
        as_of: sheet.as_of,
        assets: Assets {
            bank: report_section(&sheet.assets.bank, lookup),
            accounts_receivable: report_section(&sheet.assets.accounts_receivable, lookup),
            other_current_assets: report_section(&sheet.assets.other_current_assets, lookup),
            current_assets: lookup.money(sheet.assets.current_assets),
            fixed_assets: report_section(&sheet.assets.fixed_assets, lookup),
            other_assets: report_section(&sheet.assets.other_assets, lookup),
            total: lookup.money(sheet.assets.total),
        },
        liabilities: Liabilities {
            accounts_payable: report_section(&sheet.liabilities.accounts_payable, lookup),
            credit_cards: report_section(&sheet.liabilities.credit_cards, lookup),
            other_current_liabilities: report_section(
                &sheet.liabilities.other_current_liabilities,
                lookup,
            ),
            current_liabilities: lookup.money(sheet.liabilities.current_liabilities),
            long_term_liabilities: report_section(&sheet.liabilities.long_term_liabilities, lookup),
            total: lookup.money(sheet.liabilities.total),
        },
        equity: Equity {
            accounts: report_section(&sheet.equity.accounts, lookup),
            earlier_profit,
            retained_earnings_account_id,
            net_income: lookup.money(sheet.equity.net_income),
            total: lookup.money(sheet.equity.total),
        },
        liabilities_and_equity: lookup.money(sheet.liabilities_and_equity),
        unplaced: lookup.money(sheet.unplaced),
        left_out: left_out(sheet.left_out),
    }
}

pub(crate) fn profit_and_loss(
    pnl: &qbw::reports::ProfitAndLoss,
    lookup: &Lookup<'_>,
) -> ProfitAndLoss {
    ProfitAndLoss {
        start: pnl.period.first(),
        end: pnl.period.last(),
        income: report_section(&pnl.income, lookup),
        cost_of_goods_sold: report_section(&pnl.cost_of_goods_sold, lookup),
        gross_profit: lookup.money(pnl.gross_profit),
        expense: report_section(&pnl.expense, lookup),
        net_ordinary_income: lookup.money(pnl.net_ordinary_income),
        other_income: report_section(&pnl.other_income, lookup),
        other_expense: report_section(&pnl.other_expense, lookup),
        net_other_income: lookup.money(pnl.net_other_income),
        net_income: lookup.money(pnl.net_income),
        unplaced: lookup.money(pnl.unplaced),
        left_out: left_out(pnl.left_out),
    }
}

pub(crate) fn trial_balance(tb: &qbw::reports::TrialBalance, lookup: &Lookup<'_>) -> TrialBalance {
    TrialBalance {
        as_of: tb.as_of,
        rows: tb
            .rows
            .iter()
            .map(|row| {
                let acc = row.account.and_then(|id| lookup.account(id));
                TrialBalanceRow {
                    account_id: row.account.map(qbw::AccountId::number),
                    account_name: row.account.and_then(|id| lookup.account_name(id)),
                    account_type: acc.map(|account| account.account_type.to_string()),
                    balance: lookup.money(row.balance),
                    debit: row.debit().map(|money| lookup.money(money)),
                    credit: row.credit().map(|money| lookup.money(money)),
                }
            })
            .collect(),
        debits: lookup.money(tb.debits),
        credits: lookup.money(tb.credits),
        left_out: left_out(tb.left_out),
    }
}

pub(crate) fn open_items(items: &qbw::open_items::OpenItems, lookup: &Lookup<'_>) -> OpenItems {
    OpenItems {
        side: side_from_core(items.side),
        as_of: as_of_from_core(items.as_of),
        items: items
            .items
            .iter()
            .map(|item| OpenItem {
                transaction_id: item.transaction.number(),
                kind: item.kind.to_string(),
                date: item.date,
                number: item.number.clone(),
                name_id: item.name.map(qbw::NameId::number),
                name: item.name.and_then(|id| lookup.name(id)),
                account_id: item.account.number(),
                account_name: lookup.account_name(item.account),
                due_date: item.due_date,
                total: lookup.money(item.total),
                open: lookup.money(item.open),
                home_open: lookup.money(item.home_open),
                origin: origin(item.origin),
            })
            .collect(),
        home_total: lookup.money(items.home_total()),
        adjustments: lookup.money(items.adjustments),
        left_out: left_out(items.left_out),
    }
}

pub(crate) fn aging(aging: &qbw::aging::Aging, lookup: &Lookup<'_>) -> Aging {
    let buckets = aging
        .buckets
        .iter()
        .map(|bucket| {
            let label = match (bucket.first, bucket.last) {
                (None, Some(0)) => "Current".to_owned(),
                (Some(first), Some(last)) => format!("{first}-{last}"),
                (Some(first), None) => format!(">{}", first - 1),
                (None, Some(last)) => format!("<={last}"),
                (None, None) => "All".to_owned(),
            };
            AgingBucket {
                label,
                first_day: bucket.first,
                last_day: bucket.last,
            }
        })
        .collect();
    let rows = aging
        .rows
        .iter()
        .map(|row| AgingRow {
            name_id: row.name.map(qbw::NameId::number),
            name: row.name.and_then(|id| lookup.name(id)),
            amounts: row
                .aged
                .amounts
                .iter()
                .map(|money| lookup.money(*money))
                .collect(),
            total: lookup.money(row.aged.total),
            home_amounts: row
                .home_aged
                .amounts
                .iter()
                .map(|money| lookup.money(*money))
                .collect(),
            home_total: lookup.money(row.home_aged.total),
        })
        .collect();
    Aging {
        side: side_from_core(aging.side),
        day: aging.day,
        basis: aging_basis_from_core(aging.basis),
        buckets,
        rows,
        totals: aging
            .totals
            .amounts
            .iter()
            .map(|money| lookup.money(*money))
            .collect(),
        total: lookup.money(aging.totals.total),
        adjustments: lookup.money(aging.adjustments),
        left_out: left_out(aging.left_out),
    }
}

pub(crate) fn check_report(report: &qbw::check::Check) -> CheckReport {
    let compared = report
        .compared
        .iter()
        .map(|item| CheckComparison {
            kind: item.what.to_string(),
            looked_at: item.looked_at as u32,
            agreed: item.agreed as u32,
        })
        .collect();
    let findings = report
        .findings
        .iter()
        .map(|found| {
            let subject = match &found.subject {
                qbw::check::Subject::Books => "books".to_owned(),
                qbw::check::Subject::Account(id) => format!("account {id}"),
                qbw::check::Subject::Name(id) => format!("name {id}"),
                qbw::check::Subject::Transaction(id) => format!("transaction {id}"),
                qbw::check::Subject::Item(id) => format!("item {id}"),
                qbw::check::Subject::ItemAt(item, site) => format!("item {item} at site {site}"),
                qbw::check::Subject::Fact(key) => format!("fact {key}"),
                other => format!("{other:?}"),
            };
            CheckFinding {
                kind: found.what.to_string(),
                subject,
                computed: found.computed.map(|figure| figure.to_string()),
                expected: found.expected.map(|figure| figure.to_string()),
                is_explained: found.is_explained(),
                causes: found.causes.iter().map(ToString::to_string).collect(),
            }
        })
        .collect();
    let not_compared = report
        .not_compared
        .iter()
        .map(|entry| CheckNotCompared {
            kind: entry.what.to_string(),
            table: entry.table.to_owned(),
        })
        .collect();
    CheckReport {
        compared,
        findings,
        unexplained_count: report.unexplained().count() as u32,
        not_compared,
        left_out: left_out(report.left_out),
    }
}

pub(crate) fn value_to_json(value: &sa17::Value) -> JsonValue {
    match value {
        sa17::Value::Null => JsonValue::Null,
        sa17::Value::Boolean(flag) => JsonValue::Bool(*flag),
        sa17::Value::Integer(number) => {
            if let Ok(signed) = i64::try_from(*number) {
                JsonValue::Number(Number::from(signed))
            } else if let Ok(unsigned) = u64::try_from(*number) {
                JsonValue::Number(Number::from(unsigned))
            } else {
                JsonValue::String(number.to_string())
            }
        }
        sa17::Value::Float(number) => {
            Number::from_f64(f64::from(*number)).map_or(JsonValue::Null, JsonValue::Number)
        }
        sa17::Value::Double(number) => {
            Number::from_f64(*number).map_or(JsonValue::Null, JsonValue::Number)
        }
        sa17::Value::Decimal(decimal) => JsonValue::String(decimal.to_string()),
        sa17::Value::Text(text) => JsonValue::String(text.clone()),
        sa17::Value::Bytes(bytes) => JsonValue::String(hex_encode(bytes)),
        sa17::Value::Date(date) => JsonValue::String(date.to_string()),
        sa17::Value::Time(time) => JsonValue::String(time.to_string()),
        sa17::Value::Timestamp(timestamp) => JsonValue::String(timestamp.to_string()),
        sa17::Value::Undecoded(undecoded) => JsonValue::String(hex_encode(undecoded.bytes())),
        _ => JsonValue::Null,
    }
}

pub(crate) fn row_to_dict(row: &sa17::Row) -> BTreeMap<String, JsonValue> {
    row.columns()
        .names()
        .zip(row.values())
        .map(|(name, value)| (name.to_owned(), value_to_json(value)))
        .collect()
}

pub(crate) fn sa17_error(err: sa17::Error) -> QbwError {
    let message = err.to_string();
    match err {
        sa17::Error::Read(_) => QbwError::Io { message },
        sa17::Error::Format(_) => QbwError::Format { message },
        sa17::Error::NoKey => QbwError::NoKey,
        sa17::Error::AmbiguousKey { keys } => QbwError::AmbiguousKey {
            keys: keys.into_iter().map(store_key_from_core).collect(),
        },
        sa17::Error::WrongKey => QbwError::WrongKey,
        sa17::Error::UnsupportedEncoding { label } => QbwError::UnsupportedEncoding { label },
        _ => QbwError::Database { message },
    }
}

pub(crate) fn error(err: qbw::Error) -> QbwError {
    match err {
        qbw::Error::Database(database) => sa17_error(database),
        qbw::Error::NotACompany { table } => QbwError::NotACompany {
            table: table.to_owned(),
        },
        qbw::Error::NoTable { table } => QbwError::NoTable {
            table: table.to_owned(),
        },
        qbw::Error::Schema { table, column } => QbwError::Schema {
            table: table.to_owned(),
            column: column.to_owned(),
        },
        qbw::Error::Damaged(damage) => QbwError::Damaged {
            table: damage.table.to_owned(),
            row: damage.row.map(|row| row.to_string()),
            message: damage.to_string(),
        },
        other => QbwError::Database {
            message: other.to_string(),
        },
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

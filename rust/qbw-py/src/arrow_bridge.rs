//! Zero-copy Apache Arrow table and record-batch stream builders for `qbw`.

use std::sync::Arc;

use bimetal::arrow::array::builder::{
    BooleanBuilder, Date32Builder, Decimal128Builder, Int32Builder, StringBuilder, UInt32Builder,
};
use bimetal::arrow::array::{ArrayRef, RecordBatch, RecordBatchReader};
use bimetal::arrow::schema::{ArrowError, DataType as ArrowDataType, Field, Schema};
use bimetal::arrow::{RecordBatchStream, SchemaRef, Table as ArrowTable};
use chrono::NaiveDate;

use crate::convert::{self, Lookup};
use crate::{NameKind, QbwError, Side};

/// Fixed-point Arrow precision for QuickBooks monetary amounts and quantities.
///
/// QuickBooks Desktop stores numeric columns as `NUMERIC(20, 5)`. Precision 28
/// with scale 5 preserves every stored digit without float rounding and leaves
/// headroom for aggregations in Polars and DuckDB.
const DECIMAL_PRECISION: u8 = 28;
const DECIMAL_SCALE: i8 = 5;

fn decimal_type() -> ArrowDataType {
    ArrowDataType::Decimal128(DECIMAL_PRECISION, DECIMAL_SCALE)
}

fn decimal_builder(capacity: usize) -> Decimal128Builder {
    Decimal128Builder::with_capacity(capacity).with_data_type(decimal_type())
}

fn unix_epoch() -> NaiveDate {
    NaiveDate::from_ymd_opt(1970, 1, 1).expect("1970-01-01 is a valid Gregorian date")
}

fn date_days(date: NaiveDate) -> Result<i32, ArrowError> {
    let days = (date - unix_epoch()).num_days();
    i32::try_from(days).map_err(|_| {
        ArrowError::InvalidArgumentError(format!("date {date} out of Arrow Date32 range"))
    })
}

fn push_date_opt(builder: &mut Date32Builder, date: Option<NaiveDate>) -> Result<(), ArrowError> {
    match date {
        Some(date) => builder.append_value(date_days(date)?),
        None => builder.append_null(),
    }
    Ok(())
}

fn decimal_mantissa(decimal: sa17::Decimal) -> Result<i128, ArrowError> {
    let scaled = decimal.rescale(DECIMAL_SCALE as u16).ok_or_else(|| {
        ArrowError::InvalidArgumentError(format!(
            "decimal {decimal} does not fit Arrow Decimal128({DECIMAL_PRECISION}, {DECIMAL_SCALE})"
        ))
    })?;
    let mantissa = scaled.mantissa();
    let limit = 10_i128.pow(u32::from(DECIMAL_PRECISION));
    if mantissa <= -limit || mantissa >= limit {
        return Err(ArrowError::InvalidArgumentError(format!(
            "decimal {decimal} exceeds Arrow precision {DECIMAL_PRECISION}"
        )));
    }
    Ok(mantissa)
}

fn push_decimal(builder: &mut Decimal128Builder, decimal: sa17::Decimal) -> Result<(), ArrowError> {
    builder.append_value(decimal_mantissa(decimal)?);
    Ok(())
}

fn push_decimal_opt(
    builder: &mut Decimal128Builder,
    decimal: Option<sa17::Decimal>,
) -> Result<(), ArrowError> {
    match decimal {
        Some(decimal) => push_decimal(builder, decimal),
        None => {
            builder.append_null();
            Ok(())
        }
    }
}

fn arrow_error(error: ArrowError) -> QbwError {
    QbwError::Arrow {
        message: error.to_string(),
    }
}

fn single_batch_table(schema: SchemaRef, arrays: Vec<ArrayRef>) -> Result<ArrowTable, QbwError> {
    let batch = RecordBatch::try_new(Arc::clone(&schema), arrays).map_err(arrow_error)?;
    ArrowTable::try_new(schema, vec![batch]).map_err(arrow_error)
}

/// Export the chart of accounts into a zero-copy Arrow table.
pub(crate) fn accounts_table(
    company: &qbw::Company,
    active_only: bool,
) -> Result<ArrowTable, QbwError> {
    let lookup = Lookup::from_company(company)?;
    let accounts = company.accounts().map_err(convert::error)?;
    let rows: Vec<(usize, &qbw::accounts::Account)> = accounts
        .walk()
        .filter(|(_, account)| !active_only || account.is_active)
        .collect();
    let cap = rows.len();

    let mut id = Int32Builder::with_capacity(cap);
    let mut name = StringBuilder::with_capacity(cap, cap * 16);
    let mut full_name = StringBuilder::with_capacity(cap, cap * 24);
    let mut parent_id = Int32Builder::with_capacity(cap);
    let mut depth = UInt32Builder::with_capacity(cap);
    let mut account_type = StringBuilder::with_capacity(cap, cap * 16);
    let mut normal_side = StringBuilder::with_capacity(cap, cap * 8);
    let mut statement = StringBuilder::with_capacity(cap, cap * 16);
    let mut number = StringBuilder::with_capacity(cap, cap * 8);
    let mut description = StringBuilder::with_capacity(cap, cap * 24);
    let mut currency_id = Int32Builder::with_capacity(cap);
    let mut currency_code = StringBuilder::with_capacity(cap, cap * 4);
    let mut is_active = BooleanBuilder::with_capacity(cap);
    let mut stored_balance = decimal_builder(cap);
    let mut stored_home_balance = decimal_builder(cap);

    for (level, account) in rows {
        id.append_value(account.id.number());
        name.append_value(&account.name);
        full_name.append_value(accounts.full_name(account.id).unwrap_or(&account.name));
        parent_id.append_option(account.parent.map(qbw::AccountId::number));
        depth.append_value(level as u32);
        account_type.append_value(account.account_type.to_string());
        normal_side.append_option(account.account_type.normal_side().map(|side| match side {
            qbw::accounts::NormalSide::Debit => "debit",
            qbw::accounts::NormalSide::Credit => "credit",
            _ => "debit",
        }));
        statement.append_option(account.account_type.statement().map(|stmt| match stmt {
            qbw::accounts::Statement::BalanceSheet => "balance_sheet",
            qbw::accounts::Statement::ProfitAndLoss => "profit_and_loss",
        }));
        number.append_option(account.number.as_deref());
        description.append_option(account.description.as_deref());
        currency_id.append_value(account.currency.number());
        currency_code.append_option(lookup.currency_code(account.currency));
        is_active.append_value(account.is_active);
        push_decimal_opt(
            &mut stored_balance,
            account.stored.map(|stored| stored.amount.amount()),
        )
        .map_err(arrow_error)?;
        push_decimal_opt(
            &mut stored_home_balance,
            account
                .stored
                .and_then(|stored| stored.home_amount)
                .map(qbw::Money::amount),
        )
        .map_err(arrow_error)?;
    }

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", ArrowDataType::Int32, false),
        Field::new("name", ArrowDataType::Utf8, false),
        Field::new("full_name", ArrowDataType::Utf8, false),
        Field::new("parent_id", ArrowDataType::Int32, true),
        Field::new("depth", ArrowDataType::UInt32, false),
        Field::new("account_type", ArrowDataType::Utf8, false),
        Field::new("normal_side", ArrowDataType::Utf8, true),
        Field::new("statement", ArrowDataType::Utf8, true),
        Field::new("number", ArrowDataType::Utf8, true),
        Field::new("description", ArrowDataType::Utf8, true),
        Field::new("currency_id", ArrowDataType::Int32, false),
        Field::new("currency_code", ArrowDataType::Utf8, true),
        Field::new("is_active", ArrowDataType::Boolean, false),
        Field::new("stored_balance", decimal_type(), true),
        Field::new("stored_home_balance", decimal_type(), true),
    ]));

    single_batch_table(
        schema,
        vec![
            Arc::new(id.finish()),
            Arc::new(name.finish()),
            Arc::new(full_name.finish()),
            Arc::new(parent_id.finish()),
            Arc::new(depth.finish()),
            Arc::new(account_type.finish()),
            Arc::new(normal_side.finish()),
            Arc::new(statement.finish()),
            Arc::new(number.finish()),
            Arc::new(description.finish()),
            Arc::new(currency_id.finish()),
            Arc::new(currency_code.finish()),
            Arc::new(is_active.finish()),
            Arc::new(stored_balance.finish()),
            Arc::new(stored_home_balance.finish()),
        ],
    )
}

/// Export names (customers, vendors, employees, other names) into an Arrow table.
pub(crate) fn names_table(
    company: &qbw::Company,
    kind: Option<NameKind>,
    active_only: bool,
) -> Result<ArrowTable, QbwError> {
    let lookup = Lookup::from_company(company)?;
    let names = company.names().map_err(convert::error)?;
    let rows: Vec<qbw::names::Name<'_>> = names
        .iter()
        .filter(|entry| {
            let kind_ok = kind.is_none_or(|wanted| convert::name_kind(entry.kind()) == wanted);
            let active_ok = !active_only || entry.is_active();
            kind_ok && active_ok
        })
        .collect();
    let cap = rows.len();

    let mut id = Int32Builder::with_capacity(cap);
    let mut kind_col = StringBuilder::with_capacity(cap, cap * 10);
    let mut name = StringBuilder::with_capacity(cap, cap * 16);
    let mut full_name = StringBuilder::with_capacity(cap, cap * 24);
    let mut parent_id = Int32Builder::with_capacity(cap);
    let mut is_active = BooleanBuilder::with_capacity(cap);
    let mut currency_id = Int32Builder::with_capacity(cap);
    let mut currency_code = StringBuilder::with_capacity(cap, cap * 4);
    let mut company_col = StringBuilder::with_capacity(cap, cap * 16);
    let mut email = StringBuilder::with_capacity(cap, cap * 20);
    let mut phone = StringBuilder::with_capacity(cap, cap * 14);
    let mut stored_balance = decimal_builder(cap);
    let mut stored_home_balance = decimal_builder(cap);
    let mut opened = Date32Builder::with_capacity(cap);

    for entry in rows {
        let entry_id = entry.id();
        id.append_value(entry_id.number());
        kind_col.append_value(match convert::name_kind(entry.kind()) {
            NameKind::Customer => "customer",
            NameKind::Vendor => "vendor",
            NameKind::Employee => "employee",
            NameKind::Other => "other",
        });
        name.append_value(entry.name());
        full_name.append_value(names.full_name(entry_id).unwrap_or(entry.name()));
        is_active.append_value(entry.is_active());
        let contact = entry.contact();
        company_col.append_option(contact.and_then(|c| c.company.as_deref()));
        email.append_option(contact.and_then(|c| c.email.as_deref()));
        phone.append_option(contact.and_then(|c| c.phone.as_deref()));

        let (parent, currency, stored, opened_date) = match entry {
            qbw::names::Name::Customer(c) => (
                c.parent.map(qbw::NameId::number),
                Some(c.currency),
                c.stored,
                c.opened,
            ),
            qbw::names::Name::Vendor(v) => (None, Some(v.currency), v.stored, v.opened),
            qbw::names::Name::Employee(e) => (None, None, None, e.hired),
            qbw::names::Name::Other(o) => (None, None, None, o.opened),
            _ => (None, None, None, None),
        };
        parent_id.append_option(parent);
        currency_id.append_option(currency.map(qbw::CurrencyId::number));
        currency_code.append_option(currency.and_then(|c| lookup.currency_code(c)));
        push_decimal_opt(&mut stored_balance, stored.map(|s| s.amount.amount()))
            .map_err(arrow_error)?;
        push_decimal_opt(
            &mut stored_home_balance,
            stored.and_then(|s| s.home_amount).map(qbw::Money::amount),
        )
        .map_err(arrow_error)?;
        push_date_opt(&mut opened, opened_date).map_err(arrow_error)?;
    }

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", ArrowDataType::Int32, false),
        Field::new("kind", ArrowDataType::Utf8, false),
        Field::new("name", ArrowDataType::Utf8, false),
        Field::new("full_name", ArrowDataType::Utf8, false),
        Field::new("parent_id", ArrowDataType::Int32, true),
        Field::new("is_active", ArrowDataType::Boolean, false),
        Field::new("currency_id", ArrowDataType::Int32, true),
        Field::new("currency_code", ArrowDataType::Utf8, true),
        Field::new("company", ArrowDataType::Utf8, true),
        Field::new("email", ArrowDataType::Utf8, true),
        Field::new("phone", ArrowDataType::Utf8, true),
        Field::new("stored_balance", decimal_type(), true),
        Field::new("stored_home_balance", decimal_type(), true),
        Field::new("opened", ArrowDataType::Date32, true),
    ]));

    single_batch_table(
        schema,
        vec![
            Arc::new(id.finish()),
            Arc::new(kind_col.finish()),
            Arc::new(name.finish()),
            Arc::new(full_name.finish()),
            Arc::new(parent_id.finish()),
            Arc::new(is_active.finish()),
            Arc::new(currency_id.finish()),
            Arc::new(currency_code.finish()),
            Arc::new(company_col.finish()),
            Arc::new(email.finish()),
            Arc::new(phone.finish()),
            Arc::new(stored_balance.finish()),
            Arc::new(stored_home_balance.finish()),
            Arc::new(opened.finish()),
        ],
    )
}

/// Export items (products, services, taxes) into an Arrow table.
pub(crate) fn items_table(
    company: &qbw::Company,
    active_only: bool,
) -> Result<ArrowTable, QbwError> {
    let lookup = Lookup::from_company(company)?;
    let items = company.items().map_err(convert::error)?;
    let rows: Vec<&qbw::items::Item> = items
        .iter()
        .filter(|item| !active_only || item.is_active)
        .collect();
    let cap = rows.len();

    let mut id = Int32Builder::with_capacity(cap);
    let mut name = StringBuilder::with_capacity(cap, cap * 16);
    let mut full_name = StringBuilder::with_capacity(cap, cap * 24);
    let mut parent_id = Int32Builder::with_capacity(cap);
    let mut kind = StringBuilder::with_capacity(cap, cap * 16);
    let mut is_stocked = BooleanBuilder::with_capacity(cap);
    let mut is_active = BooleanBuilder::with_capacity(cap);
    let mut description = StringBuilder::with_capacity(cap, cap * 24);
    let mut price = decimal_builder(cap);
    let mut cost = decimal_builder(cap);
    let mut income_account_id = Int32Builder::with_capacity(cap);
    let mut income_account_name = StringBuilder::with_capacity(cap, cap * 16);
    let mut expense_account_id = Int32Builder::with_capacity(cap);
    let mut expense_account_name = StringBuilder::with_capacity(cap, cap * 16);
    let mut asset_account_id = Int32Builder::with_capacity(cap);
    let mut asset_account_name = StringBuilder::with_capacity(cap, cap * 16);
    let mut part_number = StringBuilder::with_capacity(cap, cap * 12);
    let mut on_hand = decimal_builder(cap);
    let mut stored_value = decimal_builder(cap);
    let mut stored_average_cost = decimal_builder(cap);

    for item in rows {
        id.append_value(item.id.number());
        name.append_value(&item.name);
        full_name.append_value(items.full_name(item.id).unwrap_or(&item.name));
        parent_id.append_option(item.parent.map(qbw::ItemId::number));
        kind.append_value(item.kind.to_string());
        is_stocked.append_value(item.kind.is_stocked());
        is_active.append_value(item.is_active);
        description.append_option(item.sales.description.as_deref());
        let unit_price = item.sales.price.map(|p| match p {
            qbw::Price::Each(money) => money.amount(),
            qbw::Price::Percent(pct) => pct,
            _ => sa17::Decimal::default(),
        });
        push_decimal_opt(&mut price, unit_price).map_err(arrow_error)?;
        push_decimal_opt(
            &mut cost,
            item.purchase
                .as_ref()
                .and_then(|p| p.cost)
                .map(qbw::Money::amount),
        )
        .map_err(arrow_error)?;
        income_account_id.append_option(item.sales.account.map(qbw::AccountId::number));
        income_account_name
            .append_option(item.sales.account.and_then(|id| lookup.account_name(id)));
        let expense_id = item.purchase.as_ref().and_then(|p| p.account);
        expense_account_id.append_option(expense_id.map(qbw::AccountId::number));
        expense_account_name.append_option(expense_id.and_then(|id| lookup.account_name(id)));
        let asset_id = item.stock.as_ref().and_then(|s| s.asset_account);
        asset_account_id.append_option(asset_id.map(qbw::AccountId::number));
        asset_account_name.append_option(asset_id.and_then(|id| lookup.account_name(id)));
        part_number.append_option(item.part_number.as_deref());
        push_decimal_opt(&mut on_hand, item.stored.map(|s| s.on_hand)).map_err(arrow_error)?;
        push_decimal_opt(
            &mut stored_value,
            item.stored.and_then(|s| s.value).map(qbw::Money::amount),
        )
        .map_err(arrow_error)?;
        push_decimal_opt(
            &mut stored_average_cost,
            item.stored
                .and_then(|s| s.average_cost)
                .map(qbw::Money::amount),
        )
        .map_err(arrow_error)?;
    }

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", ArrowDataType::Int32, false),
        Field::new("name", ArrowDataType::Utf8, false),
        Field::new("full_name", ArrowDataType::Utf8, false),
        Field::new("parent_id", ArrowDataType::Int32, true),
        Field::new("kind", ArrowDataType::Utf8, false),
        Field::new("is_stocked", ArrowDataType::Boolean, false),
        Field::new("is_active", ArrowDataType::Boolean, false),
        Field::new("description", ArrowDataType::Utf8, true),
        Field::new("price", decimal_type(), true),
        Field::new("cost", decimal_type(), true),
        Field::new("income_account_id", ArrowDataType::Int32, true),
        Field::new("income_account_name", ArrowDataType::Utf8, true),
        Field::new("expense_account_id", ArrowDataType::Int32, true),
        Field::new("expense_account_name", ArrowDataType::Utf8, true),
        Field::new("asset_account_id", ArrowDataType::Int32, true),
        Field::new("asset_account_name", ArrowDataType::Utf8, true),
        Field::new("part_number", ArrowDataType::Utf8, true),
        Field::new("on_hand", decimal_type(), true),
        Field::new("stored_value", decimal_type(), true),
        Field::new("stored_average_cost", decimal_type(), true),
    ]));

    single_batch_table(
        schema,
        vec![
            Arc::new(id.finish()),
            Arc::new(name.finish()),
            Arc::new(full_name.finish()),
            Arc::new(parent_id.finish()),
            Arc::new(kind.finish()),
            Arc::new(is_stocked.finish()),
            Arc::new(is_active.finish()),
            Arc::new(description.finish()),
            Arc::new(price.finish()),
            Arc::new(cost.finish()),
            Arc::new(income_account_id.finish()),
            Arc::new(income_account_name.finish()),
            Arc::new(expense_account_id.finish()),
            Arc::new(expense_account_name.finish()),
            Arc::new(asset_account_id.finish()),
            Arc::new(asset_account_name.finish()),
            Arc::new(part_number.finish()),
            Arc::new(on_hand.finish()),
            Arc::new(stored_value.finish()),
            Arc::new(stored_average_cost.finish()),
        ],
    )
}

/// Export transaction headers/summaries into an Arrow table.
pub(crate) fn transactions_table(
    company: &qbw::Company,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
) -> Result<ArrowTable, QbwError> {
    let lookup = Lookup::from_company(company)?;
    let query = qbw::query::TransactionQuery::new()
        .dated(convert::period_to_core(start, end))
        .order(
            qbw::query::TransactionOrder::Date,
            qbw::query::Direction::Ascending,
        );
    let page = company
        .transactions(&query, qbw::Page::new(0, usize::MAX))
        .map_err(convert::error)?;
    let cap = page.rows.len();

    let mut id = Int32Builder::with_capacity(cap);
    let mut kind = StringBuilder::with_capacity(cap, cap * 16);
    let mut date = Date32Builder::with_capacity(cap);
    let mut number = StringBuilder::with_capacity(cap, cap * 12);
    let mut name_id = Int32Builder::with_capacity(cap);
    let mut name = StringBuilder::with_capacity(cap, cap * 20);
    let mut account_id = Int32Builder::with_capacity(cap);
    let mut account_name = StringBuilder::with_capacity(cap, cap * 20);
    let mut memo = StringBuilder::with_capacity(cap, cap * 24);
    let mut currency_id = Int32Builder::with_capacity(cap);
    let mut currency_code = StringBuilder::with_capacity(cap, cap * 4);
    let mut amount = decimal_builder(cap);
    let mut total = decimal_builder(cap);
    let mut home_total = decimal_builder(cap);
    let mut due_date = Date32Builder::with_capacity(cap);
    let mut open_amount = decimal_builder(cap);
    let mut is_paid = BooleanBuilder::with_capacity(cap);
    let mut is_voided = BooleanBuilder::with_capacity(cap);
    let mut posting = StringBuilder::with_capacity(cap, cap * 12);

    for row in &page.rows {
        id.append_value(row.id.number());
        kind.append_value(row.kind.to_string());
        date.append_value(date_days(row.date).map_err(arrow_error)?);
        number.append_option(row.number.as_deref());
        name_id.append_option(row.name.map(qbw::NameId::number));
        name.append_option(row.name.and_then(|id| lookup.name(id)));
        account_id.append_option(row.account.map(qbw::AccountId::number));
        account_name.append_option(row.account.and_then(|id| lookup.account_name(id)));
        memo.append_option(row.memo.as_deref());
        currency_id.append_value(row.currency.number());
        currency_code.append_option(lookup.currency_code(row.currency));
        push_decimal_opt(&mut amount, row.amount.map(qbw::Money::amount)).map_err(arrow_error)?;
        push_decimal_opt(&mut total, row.total.map(qbw::Money::amount)).map_err(arrow_error)?;
        push_decimal_opt(&mut home_total, row.home_total.map(qbw::Money::amount))
            .map_err(arrow_error)?;
        push_date_opt(&mut due_date, row.due_date).map_err(arrow_error)?;
        push_decimal_opt(&mut open_amount, row.open().map(qbw::Money::amount))
            .map_err(arrow_error)?;
        is_paid.append_option(row.is_paid());
        is_voided.append_value(row.is_voided());
        posting.append_value(match row.posting {
            qbw::transactions::PostingState::Posted => "posted",
            qbw::transactions::PostingState::NonPosting | _ => "non_posting",
        });
    }

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", ArrowDataType::Int32, false),
        Field::new("kind", ArrowDataType::Utf8, false),
        Field::new("date", ArrowDataType::Date32, false),
        Field::new("number", ArrowDataType::Utf8, true),
        Field::new("name_id", ArrowDataType::Int32, true),
        Field::new("name", ArrowDataType::Utf8, true),
        Field::new("account_id", ArrowDataType::Int32, true),
        Field::new("account_name", ArrowDataType::Utf8, true),
        Field::new("memo", ArrowDataType::Utf8, true),
        Field::new("currency_id", ArrowDataType::Int32, false),
        Field::new("currency_code", ArrowDataType::Utf8, true),
        Field::new("amount", decimal_type(), true),
        Field::new("total", decimal_type(), true),
        Field::new("home_total", decimal_type(), true),
        Field::new("due_date", ArrowDataType::Date32, true),
        Field::new("open_amount", decimal_type(), true),
        Field::new("is_paid", ArrowDataType::Boolean, true),
        Field::new("is_voided", ArrowDataType::Boolean, false),
        Field::new("posting", ArrowDataType::Utf8, false),
    ]));

    single_batch_table(
        schema,
        vec![
            Arc::new(id.finish()),
            Arc::new(kind.finish()),
            Arc::new(date.finish()),
            Arc::new(number.finish()),
            Arc::new(name_id.finish()),
            Arc::new(name.finish()),
            Arc::new(account_id.finish()),
            Arc::new(account_name.finish()),
            Arc::new(memo.finish()),
            Arc::new(currency_id.finish()),
            Arc::new(currency_code.finish()),
            Arc::new(amount.finish()),
            Arc::new(total.finish()),
            Arc::new(home_total.finish()),
            Arc::new(due_date.finish()),
            Arc::new(open_amount.finish()),
            Arc::new(is_paid.finish()),
            Arc::new(is_voided.finish()),
            Arc::new(posting.finish()),
        ],
    )
}

/// Export open receivables or payables into an Arrow table.
pub(crate) fn open_items_table(
    company: &qbw::Company,
    side: Side,
    as_of: Option<NaiveDate>,
) -> Result<ArrowTable, QbwError> {
    let lookup = Lookup::from_company(company)?;
    let open = company
        .open_items(convert::side_to_core(side), convert::as_of(as_of))
        .map_err(convert::error)?;
    let cap = open.items.len();

    let mut transaction_id = Int32Builder::with_capacity(cap);
    let mut kind = StringBuilder::with_capacity(cap, cap * 16);
    let mut date = Date32Builder::with_capacity(cap);
    let mut number = StringBuilder::with_capacity(cap, cap * 12);
    let mut name_id = Int32Builder::with_capacity(cap);
    let mut name = StringBuilder::with_capacity(cap, cap * 20);
    let mut account_id = Int32Builder::with_capacity(cap);
    let mut account_name = StringBuilder::with_capacity(cap, cap * 20);
    let mut due_date = Date32Builder::with_capacity(cap);
    let mut currency_id = Int32Builder::with_capacity(cap);
    let mut currency_code = StringBuilder::with_capacity(cap, cap * 4);
    let mut total = decimal_builder(cap);
    let mut open_amount = decimal_builder(cap);
    let mut home_open = decimal_builder(cap);

    for item in &open.items {
        transaction_id.append_value(item.transaction.number());
        kind.append_value(item.kind.to_string());
        date.append_value(date_days(item.date).map_err(arrow_error)?);
        number.append_option(item.number.as_deref());
        name_id.append_option(item.name.map(qbw::NameId::number));
        name.append_option(item.name.and_then(|id| lookup.name(id)));
        account_id.append_value(item.account.number());
        account_name.append_option(lookup.account_name(item.account));
        push_date_opt(&mut due_date, item.due_date).map_err(arrow_error)?;
        currency_id.append_value(item.open.currency().number());
        currency_code.append_option(lookup.currency_code(item.open.currency()));
        push_decimal(&mut total, item.total.amount()).map_err(arrow_error)?;
        push_decimal(&mut open_amount, item.open.amount()).map_err(arrow_error)?;
        push_decimal(&mut home_open, item.home_open.amount()).map_err(arrow_error)?;
    }

    let schema = Arc::new(Schema::new(vec![
        Field::new("transaction_id", ArrowDataType::Int32, false),
        Field::new("kind", ArrowDataType::Utf8, false),
        Field::new("date", ArrowDataType::Date32, false),
        Field::new("number", ArrowDataType::Utf8, true),
        Field::new("name_id", ArrowDataType::Int32, true),
        Field::new("name", ArrowDataType::Utf8, true),
        Field::new("account_id", ArrowDataType::Int32, false),
        Field::new("account_name", ArrowDataType::Utf8, true),
        Field::new("due_date", ArrowDataType::Date32, true),
        Field::new("currency_id", ArrowDataType::Int32, false),
        Field::new("currency_code", ArrowDataType::Utf8, true),
        Field::new("total", decimal_type(), false),
        Field::new("open", decimal_type(), false),
        Field::new("home_open", decimal_type(), false),
    ]));

    single_batch_table(
        schema,
        vec![
            Arc::new(transaction_id.finish()),
            Arc::new(kind.finish()),
            Arc::new(date.finish()),
            Arc::new(number.finish()),
            Arc::new(name_id.finish()),
            Arc::new(name.finish()),
            Arc::new(account_id.finish()),
            Arc::new(account_name.finish()),
            Arc::new(due_date.finish()),
            Arc::new(currency_id.finish()),
            Arc::new(currency_code.finish()),
            Arc::new(total.finish()),
            Arc::new(open_amount.finish()),
            Arc::new(home_open.finish()),
        ],
    )
}

/// Export current inventory stock rows into an Arrow table.
pub(crate) fn stock_table(company: &qbw::Company) -> Result<ArrowTable, QbwError> {
    let lookup = Lookup::from_company(company)?;
    let stock = company.stock().map_err(convert::error)?;
    let cap = stock.rows.len();

    let mut item_id = Int32Builder::with_capacity(cap);
    let mut item_name = StringBuilder::with_capacity(cap, cap * 20);
    let mut on_hand = decimal_builder(cap);
    let mut value = decimal_builder(cap);
    let mut average_cost = decimal_builder(cap);
    let mut counted = decimal_builder(cap);
    let mut on_order = decimal_builder(cap);
    let mut on_sales_orders = decimal_builder(cap);
    let mut to_be_built = decimal_builder(cap);
    let mut required_for_builds = decimal_builder(cap);
    let mut available = decimal_builder(cap);
    let mut reorder_point = decimal_builder(cap);
    let mut needs_reorder = BooleanBuilder::with_capacity(cap);

    for row in &stock.rows {
        item_id.append_value(row.item.number());
        item_name.append_option(lookup.item_name(row.item));
        push_decimal(&mut on_hand, row.on_hand).map_err(arrow_error)?;
        push_decimal(&mut value, row.value.amount()).map_err(arrow_error)?;
        push_decimal_opt(&mut average_cost, row.average_cost.map(qbw::Money::amount))
            .map_err(arrow_error)?;
        push_decimal(&mut counted, row.counted).map_err(arrow_error)?;
        push_decimal(&mut on_order, row.on_order).map_err(arrow_error)?;
        push_decimal(&mut on_sales_orders, row.on_sales_orders).map_err(arrow_error)?;
        push_decimal(&mut to_be_built, row.to_be_built).map_err(arrow_error)?;
        push_decimal(&mut required_for_builds, row.required_for_builds).map_err(arrow_error)?;
        push_decimal(&mut available, row.available).map_err(arrow_error)?;
        push_decimal_opt(&mut reorder_point, row.reorder_point).map_err(arrow_error)?;
        needs_reorder.append_value(row.needs_reorder());
    }

    let schema = Arc::new(Schema::new(vec![
        Field::new("item_id", ArrowDataType::Int32, false),
        Field::new("item_name", ArrowDataType::Utf8, true),
        Field::new("on_hand", decimal_type(), false),
        Field::new("value", decimal_type(), false),
        Field::new("average_cost", decimal_type(), true),
        Field::new("counted", decimal_type(), false),
        Field::new("on_order", decimal_type(), false),
        Field::new("on_sales_orders", decimal_type(), false),
        Field::new("to_be_built", decimal_type(), false),
        Field::new("required_for_builds", decimal_type(), false),
        Field::new("available", decimal_type(), false),
        Field::new("reorder_point", decimal_type(), true),
        Field::new("needs_reorder", ArrowDataType::Boolean, false),
    ]));

    single_batch_table(
        schema,
        vec![
            Arc::new(item_id.finish()),
            Arc::new(item_name.finish()),
            Arc::new(on_hand.finish()),
            Arc::new(value.finish()),
            Arc::new(average_cost.finish()),
            Arc::new(counted.finish()),
            Arc::new(on_order.finish()),
            Arc::new(on_sales_orders.finish()),
            Arc::new(to_be_built.finish()),
            Arc::new(required_for_builds.finish()),
            Arc::new(available.finish()),
            Arc::new(reorder_point.finish()),
            Arc::new(needs_reorder.finish()),
        ],
    )
}

/// Stream general ledger postings as lazy Arrow `RecordBatch`es.
pub(crate) fn ledger_stream(
    company: Arc<qbw::Company>,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    account_id: Option<i32>,
    batch_size: u32,
) -> Result<RecordBatchStream, QbwError> {
    if batch_size == 0 {
        return Err(QbwError::InvalidArgument {
            message: "batch_size must be at least 1".to_owned(),
        });
    }
    // Warm cached pieces eagerly so errors surface before returning the stream.
    let _ = company.currencies().map_err(convert::error)?;
    let _ = company.accounts().map_err(convert::error)?;
    let _ = company.names().map_err(convert::error)?;
    let _ = company.items().map_err(convert::error)?;
    let ledger = company.ledger().map_err(convert::error)?;

    let period = convert::period_to_core(start, end);
    let postings: Vec<qbw::ledger::Posting> = match account_id {
        Some(id) => ledger.during(qbw::AccountId::new(id), period).collect(),
        None => ledger
            .postings()
            .filter(|posting| period.contains(posting.date))
            .collect(),
    };

    let schema = Arc::new(Schema::new(vec![
        Field::new("date", ArrowDataType::Date32, false),
        Field::new("transaction_id", ArrowDataType::Int32, false),
        Field::new("line_id", ArrowDataType::Int32, false),
        Field::new("kind", ArrowDataType::Utf8, false),
        Field::new("source", ArrowDataType::Utf8, false),
        Field::new("account_id", ArrowDataType::Int32, false),
        Field::new("account_name", ArrowDataType::Utf8, true),
        Field::new("account_type", ArrowDataType::Utf8, true),
        Field::new("name_id", ArrowDataType::Int32, true),
        Field::new("name", ArrowDataType::Utf8, true),
        Field::new("item_id", ArrowDataType::Int32, true),
        Field::new("item_name", ArrowDataType::Utf8, true),
        Field::new("currency_id", ArrowDataType::Int32, false),
        Field::new("currency_code", ArrowDataType::Utf8, true),
        Field::new("amount", decimal_type(), false),
        Field::new("home_amount", decimal_type(), false),
        Field::new("debit", decimal_type(), true),
        Field::new("credit", decimal_type(), true),
        Field::new("cleared", ArrowDataType::Utf8, false),
    ]));

    let reader = LedgerBatchReader {
        company,
        schema,
        postings,
        cursor: 0,
        batch_size: batch_size as usize,
    };
    Ok(RecordBatchStream::from_reader(reader))
}

struct LedgerBatchReader {
    company: Arc<qbw::Company>,
    schema: SchemaRef,
    postings: Vec<qbw::ledger::Posting>,
    cursor: usize,
    batch_size: usize,
}

impl LedgerBatchReader {
    fn next_batch(&mut self) -> Result<Option<RecordBatch>, ArrowError> {
        if self.cursor >= self.postings.len() {
            return Ok(None);
        }
        let end = (self.cursor + self.batch_size).min(self.postings.len());
        let slice = &self.postings[self.cursor..end];
        self.cursor = end;

        let lookup = Lookup::from_company(&self.company)
            .map_err(|error| ArrowError::ExternalError(Box::new(error)))?;
        let cap = slice.len();

        let mut date = Date32Builder::with_capacity(cap);
        let mut transaction_id = Int32Builder::with_capacity(cap);
        let mut line_id = Int32Builder::with_capacity(cap);
        let mut kind = StringBuilder::with_capacity(cap, cap * 16);
        let mut source = StringBuilder::with_capacity(cap, cap * 14);
        let mut account_id = Int32Builder::with_capacity(cap);
        let mut account_name = StringBuilder::with_capacity(cap, cap * 20);
        let mut account_type = StringBuilder::with_capacity(cap, cap * 16);
        let mut name_id = Int32Builder::with_capacity(cap);
        let mut name = StringBuilder::with_capacity(cap, cap * 20);
        let mut item_id = Int32Builder::with_capacity(cap);
        let mut item_name = StringBuilder::with_capacity(cap, cap * 16);
        let mut currency_id = Int32Builder::with_capacity(cap);
        let mut currency_code = StringBuilder::with_capacity(cap, cap * 4);
        let mut amount = decimal_builder(cap);
        let mut home_amount = decimal_builder(cap);
        let mut debit = decimal_builder(cap);
        let mut credit = decimal_builder(cap);
        let mut cleared = StringBuilder::with_capacity(cap, cap * 10);

        for posting in slice {
            date.append_value(date_days(posting.date)?);
            transaction_id.append_value(posting.transaction.number());
            line_id.append_value(posting.line.number());
            kind.append_value(posting.kind.to_string());
            source.append_value(match posting.source {
                qbw::ledger::PostingSource::OwnLine => "own_line",
                qbw::ledger::PostingSource::Line => "line",
                qbw::ledger::PostingSource::StockValue => "stock_value",
                qbw::ledger::PostingSource::CostOfGoodsSold => "cost_of_goods_sold",
                qbw::ledger::PostingSource::OtherStock | _ => "other_stock",
            });
            account_id.append_value(posting.account.number());
            account_name.append_option(lookup.account_name(posting.account));
            account_type.append_option(
                lookup
                    .account(posting.account)
                    .map(|account| account.account_type.to_string()),
            );
            name_id.append_option(posting.name.map(qbw::NameId::number));
            name.append_option(posting.name.and_then(|id| lookup.name(id)));
            item_id.append_option(posting.item.map(qbw::ItemId::number));
            item_name.append_option(posting.item.and_then(|id| lookup.item_name(id)));
            currency_id.append_value(posting.amount.currency().number());
            currency_code.append_option(lookup.currency_code(posting.amount.currency()));
            push_decimal(&mut amount, posting.amount.amount())?;
            push_decimal(&mut home_amount, posting.home_amount.amount())?;
            push_decimal_opt(&mut debit, posting.debit().map(qbw::Money::amount))?;
            push_decimal_opt(&mut credit, posting.credit().map(qbw::Money::amount))?;
            cleared.append_value(match posting.cleared {
                qbw::transactions::Cleared::NotCleared => "not_cleared",
                qbw::transactions::Cleared::Ticked => "cleared",
                qbw::transactions::Cleared::Reconciled => "reconciled",
                qbw::transactions::Cleared::MatchedByBankFeed => "matched",
                _ => "not_cleared",
            });
        }

        let arrays: Vec<ArrayRef> = vec![
            Arc::new(date.finish()),
            Arc::new(transaction_id.finish()),
            Arc::new(line_id.finish()),
            Arc::new(kind.finish()),
            Arc::new(source.finish()),
            Arc::new(account_id.finish()),
            Arc::new(account_name.finish()),
            Arc::new(account_type.finish()),
            Arc::new(name_id.finish()),
            Arc::new(name.finish()),
            Arc::new(item_id.finish()),
            Arc::new(item_name.finish()),
            Arc::new(currency_id.finish()),
            Arc::new(currency_code.finish()),
            Arc::new(amount.finish()),
            Arc::new(home_amount.finish()),
            Arc::new(debit.finish()),
            Arc::new(credit.finish()),
            Arc::new(cleared.finish()),
        ];

        RecordBatch::try_new(Arc::clone(&self.schema), arrays).map(Some)
    }
}

impl Iterator for LedgerBatchReader {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_batch() {
            Ok(Some(batch)) => Some(Ok(batch)),
            Ok(None) => None,
            Err(error) => {
                self.cursor = self.postings.len();
                Some(Err(error))
            }
        }
    }
}

impl RecordBatchReader for LedgerBatchReader {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&self.schema)
    }
}

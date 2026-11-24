//! Small companies in which customers owe and vendors are owed. Each thing that
//! is owed or paid is a line of a journal entry. There are no headers.

use qbw::Company;
use qbw::chrono::NaiveDate;
use qbw::open_items::OpenItem;
use qbw_synthetic::{Cell, SyntheticColumn, SyntheticDatabase, SyntheticRow, SyntheticTable};

use crate::small_lists::{self, Kind, State};
use crate::support;

pub(crate) const RECEIVABLE: i32 = 2;
pub(crate) const RECEIVABLE_EUR: i32 = 3;
pub(crate) const SALES: i32 = 4;
pub(crate) const PAYABLE: i32 = 5;

/// The customers: a builder with a job, and one who trades in euros.
pub(crate) const BUILDER: i32 = 10;
pub(crate) const JOB: i32 = 11;
pub(crate) const EURO: i32 = 12;
/// The vendor.
pub(crate) const SUPPLIER: i32 = 20;

/// A line on an account for what customers owe or what vendors are owed. The
/// first digits of its id are its transaction.
#[derive(Clone, Copy)]
pub(crate) struct Line {
    id: i32,
    day: &'static str,
    account: i32,
    name: i32,
    home: &'static str,
    foreign: &'static str,
    is_paid: bool,
    open: Option<&'static str>,
    due: Option<&'static str>,
}

impl Line {
    /// A line that all of is open.
    pub(crate) const fn new(
        id: i32,
        day: &'static str,
        account: i32,
        name: i32,
        amount: &'static str,
    ) -> Self {
        Self {
            id,
            day,
            account,
            name,
            home: amount,
            foreign: amount,
            is_paid: false,
            open: None,
            due: None,
        }
    }

    /// The same line, with another amount in the company's currency.
    pub(crate) const fn home(self, home: &'static str) -> Self {
        Self { home, ..self }
    }

    /// The same line, of which nothing is kept as open.
    pub(crate) const fn paid(self) -> Self {
        Self {
            is_paid: true,
            ..self
        }
    }

    /// The same line, of which a part is kept as open.
    pub(crate) const fn open(self, open: &'static str) -> Self {
        Self {
            open: Some(open),
            ..self
        }
    }

    /// The same line, with the day that it is due.
    pub(crate) const fn due(self, due: &'static str) -> Self {
        Self {
            due: Some(due),
            ..self
        }
    }
}

/// A link: its code, the line that settles, the line that is settled, and the
/// amount, as the file keeps it.
pub(crate) type Link = (i16, i32, i32, &'static str);

pub(crate) fn company(lines: &[Line], links: &[Link]) -> Company {
    with_preferences(&[], lines, links)
}

/// The company, with preferences besides its currency.
pub(crate) fn with_preferences(
    preferences: &[(&str, &str)],
    lines: &[Line],
    links: &[Link],
) -> Company {
    support::company(database(preferences, lines, links))
}

/// The database of [`with_preferences`].
pub(crate) fn database(
    preferences: &[(&str, &str)],
    lines: &[Line],
    links: &[Link],
) -> SyntheticDatabase {
    let preferences = [&[("HOMECURRENCY", "1")], preferences].concat();
    let tables = [
        accounts(),
        account_totals(),
        customers(),
        totals(
            "abmc_customer_internal",
            "customer_id",
            [BUILDER, JOB, EURO],
        ),
        vendors(),
        totals("abmc_vendor_internal", "vendor_id", [SUPPLIER]),
        no_names("abmc_employee", "employee_id"),
        no_names("abmc_other_user", "other_id"),
        totals("abmc_other_internal", "other_id", []),
        written_lines(lines),
        written_links(links),
    ];
    let database = support::preferences(&preferences);
    tables.into_iter().fold(database, SyntheticDatabase::table)
}

fn accounts() -> SyntheticTable {
    let row = |id, kind, currency| {
        let [hidden, deleted] = State::Shown.flags();
        let (id, kind, currency) = (
            Cell::integer(id),
            Cell::integer(kind),
            Cell::integer(currency),
        );
        [
            id,
            Cell::text("Account"),
            Cell::Null,
            kind,
            currency,
            hidden,
            deleted,
        ]
    };
    small_lists::table(
        "abmc_account_user",
        [
            ("account_id", Kind::Whole),
            ("name", Kind::Text),
            ("parent_id", Kind::Whole),
            ("type", Kind::Whole),
            ("currency_id", Kind::Whole),
            ("is_hidden_bool", Kind::Flag),
            ("is_deleted_bool", Kind::Flag),
        ],
        [
            row(RECEIVABLE, 1, 1),
            row(RECEIVABLE_EUR, 1, 2),
            row(SALES, 10, 1),
            row(PAYABLE, 5, 1),
        ],
    )
}

fn account_totals() -> SyntheticTable {
    small_lists::table(
        "abmc_account_internal",
        [
            ("account_id", Kind::Whole),
            ("special_type", Kind::Whole),
            ("display_index_next", Kind::Whole),
        ],
        [],
    )
}

fn customers() -> SyntheticTable {
    let row = |id, name, parent, currency| {
        let [hidden, deleted] = State::Shown.flags();
        let (parent, currency) = (Cell::integer(parent), Cell::integer(currency));
        [
            Cell::integer(id),
            Cell::text(name),
            parent,
            currency,
            hidden,
            deleted,
        ]
    };
    small_lists::table(
        "abmc_customer_user",
        [
            ("customer_id", Kind::Whole),
            ("name", Kind::Text),
            ("parent_id", Kind::Whole),
            ("currency_id", Kind::Whole),
            ("is_hidden_bool", Kind::Flag),
            ("is_deleted_bool", Kind::Flag),
        ],
        [
            row(BUILDER, "Builder", 0, 1),
            row(JOB, "Garage", BUILDER, 1),
            row(EURO, "Euro", 0, 2),
        ],
    )
}

fn vendors() -> SyntheticTable {
    let [hidden, deleted] = State::Shown.flags();
    let supplier = [
        Cell::integer(SUPPLIER),
        Cell::text("Supplier"),
        Cell::integer(1),
        hidden,
        deleted,
    ];
    small_lists::table(
        "abmc_vendor_user",
        [
            ("vendor_id", Kind::Whole),
            ("name", Kind::Text),
            ("currency_id", Kind::Whole),
            ("is_hidden_bool", Kind::Flag),
            ("is_deleted_bool", Kind::Flag),
        ],
        [supplier],
    )
}

fn totals<const ROWS: usize>(table: &str, key: &str, ids: [i32; ROWS]) -> SyntheticTable {
    let rows = ids.map(|id| [Cell::integer(id), Cell::integer(0)]);
    let columns = [(key, Kind::Whole), ("display_index_next", Kind::Whole)];
    small_lists::table(table, columns, rows)
}

/// A list of names with nobody in it.
fn no_names(table: &str, key: &str) -> SyntheticTable {
    let columns = [
        (key, Kind::Whole),
        ("name", Kind::Text),
        ("is_hidden_bool", Kind::Flag),
        ("is_deleted_bool", Kind::Flag),
    ];
    small_lists::table(table, columns, [])
}

/// The lines, each with a line on an account for sales as its other side, which
/// is not there.
fn written_lines(lines: &[Line]) -> SyntheticTable {
    let amount = |name| SyntheticColumn::new(name, 3).width(20).scale(5);
    let table = SyntheticTable::new("abmc_general_journal_lineitem")
        .column(SyntheticColumn::new("target_id", 2))
        .column(SyntheticColumn::new("transaction_id", 2))
        .column(SyntheticColumn::new("transaction_date", 6))
        .column(SyntheticColumn::new("transaction_view_type", 1).width(2))
        .column(SyntheticColumn::new("account_id", 2))
        .column(SyntheticColumn::new("is_source_bool", 24).width(1))
        .column(SyntheticColumn::new("sibling_account_id", 2))
        .column(SyntheticColumn::new("next_target_id", 2))
        .column(amount("amount_amt"))
        .column(amount("foreign_amount_amt"))
        .column(SyntheticColumn::new("customer_id", 2).nullable())
        .column(SyntheticColumn::new("vendor_id", 2).nullable())
        .column(SyntheticColumn::new("is_paid_bool", 24).width(1))
        .column(amount("open_balance_amt").nullable())
        .column(SyntheticColumn::new("due_date", 6).nullable())
        .column(SyntheticColumn::new("exchange_rate", 4));
    lines.iter().fold(table, |table, line| {
        let rate = if line.account == RECEIVABLE_EUR {
            1.1
        } else {
            1.0
        };
        let (customer, vendor) = if line.account == PAYABLE {
            (Cell::Null, Cell::integer(line.name))
        } else {
            (Cell::integer(line.name), Cell::Null)
        };
        table.row(SyntheticRow::new([
            Cell::integer(line.id),
            Cell::integer(line.id / 100),
            Cell::day(line.day),
            Cell::small_int(8),
            Cell::integer(line.account),
            Cell::bit(line.id % 100 == 1),
            Cell::integer(SALES),
            Cell::integer(0),
            Cell::decimal(line.home),
            Cell::decimal(line.foreign),
            customer,
            vendor,
            Cell::bit(line.is_paid),
            line.open.map_or(Cell::Null, Cell::decimal),
            line.due.map_or(Cell::Null, Cell::day),
            Cell::float(rate),
        ]))
    })
}

/// The links, each with its mirror.
fn written_links(links: &[Link]) -> SyntheticTable {
    let table = SyntheticTable::new("abmc_transaction_link")
        .column(SyntheticColumn::new("link_id", 2))
        .column(SyntheticColumn::new("transaction_id", 2))
        .column(SyntheticColumn::new("transaction_2_id", 2))
        .column(SyntheticColumn::new("target_1_id", 2))
        .column(SyntheticColumn::new("target_2_id", 2))
        .column(SyntheticColumn::new("link_type", 1).width(2))
        .column(SyntheticColumn::new("is_t1_source_bool", 24).width(1))
        .column(SyntheticColumn::new("amount_amt", 3).width(20).scale(5));
    let links = (0..).step_by(2).zip(links);
    links.fold(table, |table, (id, &(code, settles, settled, amount))| {
        let row = |id: i32, one: i32, other: i32| {
            SyntheticRow::new([
                Cell::integer(id),
                Cell::integer(one / 100),
                Cell::integer(other / 100),
                Cell::integer(one),
                Cell::integer(other),
                Cell::small_int(code),
                Cell::bit(one == settles),
                Cell::decimal(amount),
            ])
        };
        table
            .row(row(id, settles, settled))
            .row(row(id + 1, settled, settles))
    })
}

pub(crate) fn day(text: &str) -> NaiveDate {
    text.parse().unwrap()
}

/// The transaction, the name and what is open of each item.
pub(crate) fn opened<'items>(
    items: impl IntoIterator<Item = &'items OpenItem>,
) -> Vec<(i32, i32, String)> {
    let items = items.into_iter().map(|item| {
        let name = item.name.unwrap().number();
        (item.transaction.number(), name, item.open.to_string())
    });
    items.collect()
}

pub(crate) fn item(transaction: i32, name: i32, open: &str) -> (i32, i32, String) {
    (transaction, name, open.to_owned())
}

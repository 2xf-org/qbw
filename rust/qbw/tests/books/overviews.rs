//! What there is about an entry, put together.

use qbw::chrono::NaiveDate;
use qbw::names::Name;
use qbw::query::{KindCount, TransactionPage};
use qbw::reports::AccountListingRow;
use qbw::transactions::TransactionKind;
use qbw::{AccountId, AsOf, ItemId, Money, NameId, Side};

use crate::{counted, forms, support};

fn day(text: &str) -> NaiveDate {
    text.parse().unwrap()
}

fn written(money: Option<Money>) -> Option<String> {
    money.map(|money| money.to_string())
}

/// The type, the number, the date, the account and the amount of each row: what
/// the centers show.
fn columns(page: &TransactionPage) -> Vec<(TransactionKind, Option<&str>, String, i32, String)> {
    let rows = page.rows.iter().map(|row| {
        (
            row.kind,
            row.number.as_deref(),
            row.date.to_string(),
            row.account.unwrap().number(),
            row.amount.unwrap().to_string(),
        )
    });
    rows.collect()
}

fn counts(kinds: &[KindCount]) -> Vec<(TransactionKind, usize)> {
    kinds.iter().map(|kind| (kind.kind, kind.count)).collect()
}

#[test]
fn a_customer_has_its_jobs_balance_and_transactions() {
    let company = support::books();
    let ada = company.name_overview(NameId::new(101), day("2024-06-30"), 3);
    let ada = ada.unwrap().unwrap();
    assert!(matches!(ada.record, Name::Customer(customer) if customer.name == "Ada Stores"));
    let balance = ada.balance.unwrap();
    assert_eq!(balance.amount.to_string(), "45.00");
    assert_eq!(balance.home_amount, balance.amount);
    assert_eq!(written(ada.total).as_deref(), Some("45.00"));
    let owed = ada.owed.as_ref().unwrap();
    assert_eq!(
        (owed.name, owed.side),
        (Some(NameId::new(101)), Side::Receivable)
    );
    assert_eq!(owed.home_overdue.to_string(), "45.00");
    // Invoice 1043 is the job's.
    let expected = [
        (
            TransactionKind::Payment,
            None,
            "2024-06-01".to_owned(),
            3,
            "-1.00".to_owned(),
        ),
        (
            TransactionKind::Estimate,
            Some("E-1"),
            "2024-05-01".to_owned(),
            21,
            "129.00".to_owned(),
        ),
        (
            TransactionKind::Invoice,
            Some("1043"),
            "2024-04-25".to_owned(),
            3,
            "50.00".to_owned(),
        ),
    ];
    assert_eq!(columns(&ada.transactions), expected);
    assert_eq!(ada.transactions.row_count, 7);
    let kinds = [
        (TransactionKind::Invoice, 2),
        (TransactionKind::CreditMemo, 1),
        (TransactionKind::Payment, 2),
        (TransactionKind::Estimate, 1),
        (TransactionKind::SalesOrder, 1),
    ];
    assert_eq!(counts(&ada.transactions.kinds), kinds);
    assert!(ada.left_out.is_nothing());
}

#[test]
fn an_overview_is_of_the_end_of_its_day() {
    let company = support::books();
    let ada = company.name_overview(NameId::new(101), day("2024-01-31"), 9);
    let ada = ada.unwrap().unwrap();
    assert_eq!(written(ada.total).as_deref(), Some("100.00"));
    assert_eq!(ada.owed.unwrap().home_total.to_string(), "100.00");
    assert_eq!(ada.transactions.row_count, 2);
}

#[test]
fn a_vendor_is_owed_less_what_it_credited() {
    let company = support::books();
    let vendor = company.name_overview(NameId::new(201), day("2024-06-30"), 1);
    let vendor = vendor.unwrap().unwrap();
    assert!(matches!(vendor.record, Name::Vendor(_)));
    assert_eq!(vendor.balance.unwrap().amount.to_string(), "20.00");
    let owed = vendor.owed.unwrap();
    assert_eq!(owed.side, Side::Payable);
    assert_eq!(owed.home_overdue.to_string(), "40.00");
    assert_eq!(vendor.transactions.rows[0].number.as_deref(), Some("R-9"));
    assert_eq!(vendor.transactions.row_count, 7);
}

#[test]
fn an_employee_and_another_name_have_no_balance() {
    let company = support::books();
    let today = day("2024-06-30");
    let employee = company
        .name_overview(NameId::new(301), today, 9)
        .unwrap()
        .unwrap();
    assert!(matches!(employee.record, Name::Employee(_)));
    assert!(employee.balance.is_none() && employee.total.is_none() && employee.owed.is_none());
    let other = company
        .name_overview(NameId::new(401), today, 9)
        .unwrap()
        .unwrap();
    assert!(matches!(other.record, Name::Other(_)) && other.balance.is_none());
    assert_eq!(other.transactions.rows[0].number.as_deref(), Some("502"));
    assert!(
        company
            .name_overview(NameId::new(99), today, 9)
            .unwrap()
            .is_none()
    );
}

#[test]
fn an_item_has_its_stock_and_the_lines_that_are_part_of_the_books() {
    let company = support::books();
    let valve = company.item_overview(ItemId::new(502), 2).unwrap().unwrap();
    assert_eq!(valve.record.name, "Valve");
    let stock = valve.stock.unwrap();
    assert_eq!(
        (stock.on_hand.to_string(), stock.value.to_string()),
        ("8".to_owned(), "160.00".to_owned())
    );
    let rows = valve.rows.iter().map(|row| {
        let posting = &row.posting;
        (
            posting.kind,
            row.number.as_deref(),
            posting.account.number(),
            posting.amount.to_string(),
        )
    });
    let rows: Vec<_> = rows.collect();
    let expected = [
        (
            TransactionKind::AssemblyBuild,
            Some("B-1"),
            6,
            "-40.00".to_owned(),
        ),
        (
            TransactionKind::InventoryAdjustment,
            Some("A-1"),
            6,
            "20.00".to_owned(),
        ),
    ];
    assert_eq!(rows, expected);
    assert_eq!(valve.row_count, 7);
    assert!(valve.left_out.is_nothing());

    let service = company.item_overview(ItemId::new(501), 9).unwrap().unwrap();
    assert!(service.stock.is_none());
    let amounts = service
        .rows
        .iter()
        .map(|row| row.posting.amount.to_string());
    assert_eq!(amounts.collect::<Vec<_>>(), ["-110.00", "-40.00"]);
    assert!(company.item_overview(ItemId::new(99), 9).unwrap().is_none());
}

#[test]
fn an_account_has_its_balance_and_the_end_of_its_register() {
    let company = support::books();
    let today = day("2024-06-30");
    let checking = company
        .account_overview(AccountId::new(1), today, 2)
        .unwrap()
        .unwrap();
    assert_eq!(checking.record.name, "Checking");
    assert_eq!(checking.balance.unwrap().amount.to_string(), "837.00");
    assert_eq!(written(checking.total).as_deref(), Some("837.00"));
    let register = &checking.register;
    let rows = register
        .rows
        .iter()
        .map(|row| (row.amount.to_string(), row.balance.to_string()));
    let expected = [("0.00", "845.00"), ("-8.00", "837.00")]
        .map(|(amount, balance)| (amount.to_owned(), balance.to_owned()));
    assert_eq!(rows.collect::<Vec<_>>(), expected);
    assert_eq!(
        (register.row_count, register.closing),
        (6, checking.balance.unwrap())
    );

    let euro = company
        .account_overview(AccountId::new(2), today, 9)
        .unwrap()
        .unwrap();
    let (balance, total) = (euro.balance.unwrap(), euro.total.unwrap());
    assert_eq!(
        (balance.amount.to_string(), total.to_string()),
        ("150.00".to_owned(), "180.00".to_owned())
    );

    let sales = company
        .account_overview(AccountId::new(13), today, 9)
        .unwrap()
        .unwrap();
    assert!(sales.balance.is_none() && sales.total.is_none());
    assert_eq!(sales.register.closing.amount.to_string(), "-100.00");
    assert!(
        company
            .account_overview(AccountId::new(99), today, 9)
            .unwrap()
            .is_none()
    );
}

#[test]
fn a_row_of_an_item_has_the_quantity_and_the_rate_of_its_line() {
    let company = support::books();
    let valve = company
        .item_overview(ItemId::new(502), 20)
        .unwrap()
        .unwrap();
    let rows: Vec<(i32, String, String)> = valve
        .rows
        .iter()
        .map(|row| {
            (
                row.posting.transaction.number(),
                row.quantity
                    .map(|quantity| quantity.to_string())
                    .unwrap_or_default(),
                row.rate.map(forms::rate).unwrap_or_default(),
            )
        })
        .collect();
    let row = |id, quantity: &str, rate: &str| (id, quantity.to_owned(), rate.to_owned());
    let expected = [
        row(2200, "2", ""),
        row(2100, "1", ""),
        row(1500, "2", "20.00"),
        row(1200, "1", "20.00"),
        row(700, "1", "50.00"),
        row(400, "3", "50.00"),
        row(200, "10", "20.00"),
    ];
    assert_eq!(rows, expected);
    assert!(valve.left_out.is_nothing());
}

#[test]
fn the_company_has_what_is_in_the_bank_and_what_is_owed() {
    let company = support::books();
    let overview = company.overview(day("2024-06-30"), 2).unwrap();
    let balance = |row: &AccountListingRow| {
        (
            row.account.number(),
            row.balance.unwrap().amount.to_string(),
        )
    };
    let bank: Vec<_> = overview.bank.iter().map(balance).collect();
    assert_eq!(bank, [(1, "837.00".to_owned()), (2, "150.00".to_owned())]);
    let (receivable, payable) = (&overview.receivable, &overview.payable);
    assert_eq!(
        (receivable.name, receivable.home_total.to_string()),
        (None, "45.00".to_owned())
    );
    assert_eq!(receivable.home_overdue.to_string(), "45.00");
    assert_eq!(
        (
            payable.home_total.to_string(),
            payable.home_overdue.to_string()
        ),
        ("20.00".to_owned(), "40.00".to_owned())
    );
    let latest: Vec<i32> = overview
        .transactions
        .rows
        .iter()
        .map(|row| row.id.number())
        .collect();
    assert_eq!(latest, [2500, 2400]);

    // Nothing in an overview is worked out in a way of its own.
    let open = company
        .open_items(Side::Payable, day("2024-06-30"))
        .unwrap();
    assert_eq!(
        (payable.home_total, &payable.items),
        (open.home_total(), &open.items)
    );
    let listing = company
        .account_listing(AsOf::Date(day("2024-06-30")))
        .unwrap();
    assert!(listing.rows.contains(&overview.bank[0]));
}

#[test]
fn an_overview_reads_only_the_rows_that_it_shows() {
    let company = counted::books();
    let today = day("2024-06-30");
    let fetched = || company.source().fetched();
    company.ledger().unwrap();
    company
        .name_overview(NameId::new(101), today, 0)
        .unwrap()
        .unwrap();
    let before = fetched();
    company
        .name_overview(NameId::new(101), today, 2)
        .unwrap()
        .unwrap();
    // The memo of each row is in its header.
    assert_eq!(fetched() - before, 2);
    let before = fetched();
    company
        .account_overview(AccountId::new(1), today, 3)
        .unwrap()
        .unwrap();
    assert_eq!(fetched() - before, 3);
    let before = fetched();
    company.item_overview(ItemId::new(502), 5).unwrap().unwrap();
    // The quantity and the rate of each row are in its line.
    assert_eq!(fetched() - before, 5);
}

#[test]
fn an_overview_of_damaged_books_says_what_it_lacks() {
    let company = support::damaged_books();
    let today = day("2024-06-30");
    let overview = company.overview(today, 5).unwrap();
    assert!(!overview.receivable.left_out.is_nothing());
    assert!(!overview.transactions.left_out.is_nothing());
    let ada = company.name_overview(NameId::new(101), today, 5).unwrap();
    assert!(!ada.unwrap().left_out.is_nothing());
    let checking = company.account_overview(AccountId::new(1), today, 5);
    assert!(!checking.unwrap().unwrap().register.left_out.is_nothing());
    let valve = company.item_overview(ItemId::new(502), 5).unwrap();
    assert!(!valve.unwrap().left_out.is_nothing());
}

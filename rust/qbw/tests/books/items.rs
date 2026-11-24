//! Items.

use qbw::items::{Item, ItemKind, SpecialItem};
use qbw::{
    AccountId, CategoryId, CurrencyId, Error, ItemId, List, NameId, Price, Problem, SalesTaxCodeId,
    SiteId,
};
use qbw_synthetic::{Cell, SyntheticTable};

use crate::small_lists::{Kind, State};
use crate::{small_lists, support};

/// The price of an item as it is typed into the tests: a percentage has a
/// sign.
fn price(item: &Item) -> Option<String> {
    Some(match item.sales.price? {
        Price::Each(money) => money.to_string(),
        Price::Percent(percent) => format!("{}%", percent),
        _ => return None,
    })
}

#[test]
fn items_are_in_quickbooks_order() {
    let company = support::books();
    let items = company.items().unwrap();
    let in_order: Vec<(&str, ItemKind)> = items
        .iter()
        .map(|item| (item.name.as_str(), item.kind))
        .collect();
    assert_eq!(
        in_order,
        [
            ("Installation", ItemKind::Service),
            ("Valve", ItemKind::InventoryPart),
            ("Twin valve", ItemKind::Assembly),
            ("Subtotal", ItemKind::Subtotal),
            ("Valve kit", ItemKind::Group),
            ("Discount", ItemKind::Discount),
            ("State tax", ItemKind::SalesTax),
        ]
    );
    assert!(items.left_out().is_nothing());
    // The deleted one.
    assert!(items.get(ItemId::new(508)).is_none());
}

/// The numbers of an item's accounts: of its sales, of its cost and of its
/// stock.
fn accounts(item: &Item) -> [Option<i32>; 3] {
    let purchase = item.purchase.as_ref().and_then(|purchase| purchase.account);
    let stock = item.stock.as_ref().and_then(|stock| stock.asset_account);
    let accounts = [item.sales.account, purchase, stock];
    accounts.map(|account| account.map(AccountId::number))
}

#[test]
fn the_kind_decides_which_accounts_an_item_has() {
    let company = support::books();
    let items = company.items().unwrap();
    let accounts = |name| accounts(items.named(name).unwrap());
    assert_eq!(accounts("Installation"), [Some(12), None, None]);
    assert_eq!(accounts("Valve"), [Some(13), Some(14), Some(6)]);
    assert_eq!(accounts("Twin valve"), [Some(13), Some(14), Some(6)]);
    assert_eq!(accounts("Subtotal"), [None, None, None]);
    assert_eq!(accounts("Valve kit"), [None, None, None]);
    assert_eq!(accounts("Discount"), [Some(18), None, None]);
    assert_eq!(accounts("State tax"), [Some(9), None, None]);
}

#[test]
fn a_price_is_in_the_column_that_has_it() {
    let company = support::books();
    let items = company.items().unwrap();
    let price = |name| price(items.named(name).unwrap());
    assert_eq!(price("Installation").as_deref(), Some("40.00"));
    assert_eq!(price("Valve").as_deref(), Some("50.00"));
    assert_eq!(price("State tax").as_deref(), Some("8%"));
    assert_eq!(price("Discount").as_deref(), Some("0.00"));
    assert_eq!((price("Subtotal"), price("Valve kit")), (None, None));
    let home = company.preferences().unwrap().home_currency;
    let valve = items.named("Valve").unwrap();
    let Some(Price::Each(each)) = valve.sales.price else {
        panic!("so much for one: {:?}", valve.sales);
    };
    assert_eq!(each.currency(), home);
}

#[test]
fn an_item_in_stock_has_what_the_window_shows() {
    let company = support::books();
    let items = company.items().unwrap();
    let valve = items.named("valve").unwrap();
    assert_eq!((valve.id, valve.special), (ItemId::new(502), None));
    assert_eq!(valve.part_number.as_deref(), Some("CV-20"));
    assert_eq!(valve.tax_code, Some(SalesTaxCodeId::new(1)));
    assert_eq!(valve.category, Some(CategoryId::new(1)));
    assert_eq!(valve.sales.description.as_deref(), Some("Brass valve"));
    assert_eq!(valve.list_id.as_deref(), Some("800001F6-1"));
    assert!(valve.is_active && valve.members.is_empty() && valve.tax.is_none());

    let purchase = valve.purchase.as_ref().unwrap();
    assert_eq!(purchase.cost.unwrap().to_string(), "20.00");
    assert_eq!(purchase.vendor, Some(NameId::new(201)));
    assert_eq!(purchase.description.as_deref(), Some("Valve, brass"));
    let stock = valve.stock.as_ref().unwrap();
    assert_eq!(stock.reorder_point, "12".parse().ok());
    assert_eq!(stock.build_point, None);
    let twin = items.named("Twin valve").unwrap();
    assert_eq!(twin.stock.as_ref().unwrap().build_point, "2".parse().ok());
    let cost = twin.purchase.as_ref().unwrap().cost;
    assert_eq!(cost.unwrap().to_string(), "40.00");
}

#[test]
fn the_stock_that_quickbooks_keeps_is_there_for_items_in_stock() {
    let company = support::books();
    let items = company.items().unwrap();
    let stored = items.named("Valve").unwrap().stored.unwrap();
    assert_eq!(stored.on_hand.to_string(), "8");
    assert_eq!(stored.value.unwrap().to_string(), "160.00");
    assert_eq!(stored.average_cost.unwrap().to_string(), "20.00");
    assert_eq!(stored.on_order, "3".parse().ok());
    assert_eq!(stored.on_sales_orders, "4".parse().ok());
    assert_eq!(stored.to_be_built, "0".parse().ok());
    assert_eq!(stored.required_for_builds, "0".parse().ok());
    let with_stock = items.iter().filter(|item| item.stored.is_some());
    assert_eq!(with_stock.count(), 2);
}

/// The members of an item, with how many there are of each.
fn members(item: &Item) -> Vec<(i32, Option<String>)> {
    let members = item.members.iter();
    members
        .map(|member| {
            let quantity = member.quantity;
            let quantity = quantity.map(|quantity| quantity.to_string());
            (member.item.number(), quantity)
        })
        .collect()
}

#[test]
fn groups_and_assemblies_have_members_in_order() {
    let company = support::books();
    let items = company.items().unwrap();
    let kit = items.named("Valve kit").unwrap();
    let count = |text: &str| Some(text.to_owned());
    assert_eq!(members(kit), [(502, count("2")), (501, count("1"))]);
    assert!(kit.prints_members);
    assert_eq!(kit.special, Some(SpecialItem::ReimbursableGroup));
    let twin = items.named("Twin valve").unwrap();
    assert_eq!(members(twin), [(502, count("2"))]);
    assert!(!twin.prints_members);
}

#[test]
fn a_sales_tax_has_the_vendor_it_is_paid_to() {
    let company = support::books();
    let items = company.items().unwrap();
    let tax = items.named("State tax").unwrap();
    assert_eq!(tax.tax.unwrap().agency, Some(NameId::new(202)));
    let with_tax = items.iter().filter(|item| item.tax.is_some());
    assert_eq!(with_tax.count(), 1);
}

#[test]
fn every_item_leads_to_its_stored_row() {
    let company = support::books();
    for item in company.items().unwrap() {
        let row = company.stored_row(item.origin).unwrap().unwrap();
        let key = row.get("item_id").unwrap().as_i64();
        assert_eq!(key, Some(item.id.number().into()));
        for member in &item.members {
            let row = company.stored_row(member.origin).unwrap().unwrap();
            let of = row.get("item_id").unwrap().as_i64();
            assert_eq!(of, Some(item.id.number().into()));
        }
    }
}

/// An item as it is typed below: its id, its name, the item above it and its
/// kind.
type Row = (i32, &'static str, i32, Option<i32>);

/// The tables of items with the columns that every file has, and `rows`.
fn item_tables(rows: &[Row]) -> Vec<SyntheticTable> {
    let rows = rows.iter().map(|&(id, name, parent, kind)| {
        let [is_hidden, is_deleted] = State::Shown.flags();
        [
            Cell::integer(id),
            Cell::text(name),
            Cell::integer(parent),
            kind.map_or(Cell::Null, Cell::integer),
            is_hidden,
            is_deleted,
        ]
    });
    let items = small_lists::table(
        "abmc_item_user",
        [
            ("item_id", Kind::Whole),
            ("name", Kind::Text),
            ("parent_id", Kind::Whole),
            ("item_type", Kind::Whole),
            ("is_hidden_bool", Kind::Flag),
            ("is_deleted_bool", Kind::Flag),
        ],
        rows,
    );
    let totals = small_lists::table("abmc_item_internal", [("item_id", Kind::Whole)], []);
    vec![items, totals]
}

fn full_names(items: &List<Item>) -> Vec<&str> {
    let names = items.iter().map(|item| items.full_name(item.id));
    names.map(Option::unwrap).collect()
}

#[test]
fn an_older_file_has_items_without_what_came_later() {
    let tables = item_tables(&[(1, "Bolts", 0, Some(3)), (2, "Brass", 1, Some(3))]);
    let company = small_lists::company(tables);
    let items = company.items().unwrap();
    assert_eq!(full_names(items), ["Bolts", "Bolts:Brass"]);
    let brass = items.named("Bolts:Brass").unwrap();
    assert_eq!(brass.kind, ItemKind::NonInventoryPart);
    assert_eq!(accounts(brass), [None, None, None]);
    assert!(brass.sales.price.is_none() && brass.stored.is_none());
    assert!(items.left_out().is_nothing() && brass.is_active);
}

#[test]
fn an_item_that_cannot_be_read_is_left_out_and_counted() {
    let tables = item_tables(&[(1, "Bolts", 0, Some(3)), (2, "Lost", 0, None)]);
    let company = small_lists::company(tables);
    let items = company.items().unwrap();
    assert_eq!(full_names(items), ["Bolts"]);
    let [damage] = items.damage() else {
        panic!("one item is damaged: {:?}", items.damage());
    };
    assert!(matches!(
        damage.problem,
        Problem::Column {
            column: "item_type",
            ..
        }
    ));
}

#[test]
fn a_file_without_items_has_no_list_of_them() {
    let company = small_lists::company([]);
    let error = company.items().map(|_| ()).unwrap_err();
    assert!(matches!(
        error,
        Error::NoTable {
            table: "abmc_item_internal"
        }
    ));
}

#[test]
fn the_stock_of_a_site_is_with_its_item() {
    let row = |site, on_hand, state: State| {
        let [_, is_deleted] = state.flags();
        [Cell::integer(1), site, Cell::decimal(on_hand), is_deleted]
    };
    let sites = small_lists::table(
        "abmc_item_site",
        [
            ("item_id", Kind::Whole),
            ("site_id", Kind::Whole),
            ("qnty_on_hand", Kind::Number),
            ("is_deleted_bool", Kind::Flag),
        ],
        [
            row(Cell::integer(12), "3.00", State::Shown),
            row(Cell::integer(11), "5.00", State::Shown),
            row(Cell::Null, "0.00", State::Shown),
            row(Cell::integer(13), "9.00", State::Deleted),
        ],
    );
    let mut tables = item_tables(&[(1, "Bolts", 0, Some(1)), (2, "Nuts", 0, Some(1))]);
    tables.push(sites);
    let company = small_lists::company(tables);
    let items = company.items().unwrap();
    let bolts = items.named("Bolts").unwrap();
    let on_hand: Vec<(Option<SiteId>, String)> = bolts
        .sites
        .iter()
        .map(|site| (site.site, site.on_hand.unwrap().to_string()))
        .collect();
    let on_hand: Vec<(Option<i32>, &str)> = on_hand
        .iter()
        .map(|(site, on_hand)| (site.map(SiteId::number), on_hand.as_str()))
        .collect();
    assert_eq!(on_hand, [(None, "0"), (Some(11), "5"), (Some(12), "3")]);
    assert!(items.named("Nuts").unwrap().sites.is_empty());
}

/// Which sides of trade an item is on.
#[derive(Clone, Copy)]
enum Trade {
    Sold,
    BoughtAndSold,
}

/// The table of items with the columns that their costs and prices are in.
fn priced_items(rows: &[(i32, i32, Trade, [Cell; 3])]) -> Vec<SyntheticTable> {
    let rows = rows.iter().map(|(id, kind, trade, figures)| {
        let [is_hidden, is_deleted] = State::Shown.flags();
        let [cost, exact_cost, percent] = figures.clone();
        [
            Cell::integer(*id),
            Cell::text("Part"),
            Cell::Null,
            Cell::integer(*kind),
            is_hidden,
            is_deleted,
            Cell::integer(7),
            Cell::bit(matches!(trade, Trade::BoughtAndSold)),
            cost,
            exact_cost,
            percent,
        ]
    });
    let items = small_lists::table(
        "abmc_item_user",
        [
            ("item_id", Kind::Whole),
            ("name", Kind::Text),
            ("parent_id", Kind::Whole),
            ("item_type", Kind::Whole),
            ("is_hidden_bool", Kind::Flag),
            ("is_deleted_bool", Kind::Flag),
            ("cogs_account_id", Kind::Whole),
            ("is_pass_thru_bool", Kind::Flag),
            ("unit_cost_amt", Kind::Number),
            ("unit_cost_qnty", Kind::Number),
            ("sales_price_pct", Kind::Number),
        ],
        rows,
    );
    let totals = small_lists::table("abmc_item_internal", [("item_id", Kind::Whole)], []);
    vec![items, totals]
}

#[test]
fn a_part_with_more_digits_and_one_that_is_bought_and_sold() {
    let cost = |text| [Cell::decimal(text), Cell::Null, Cell::Null];
    let exact_cost = |text| [Cell::Null, Cell::decimal(text), Cell::Null];
    let percent = |text| [Cell::Null, Cell::Null, Cell::decimal(text)];
    let company = small_lists::company(priced_items(&[
        (1, 1, Trade::Sold, exact_cost("0.01234")),
        (2, 0, Trade::BoughtAndSold, cost("5.00")),
        (3, 0, Trade::Sold, cost("5.00")),
        (4, 8, Trade::Sold, percent("-10.00")),
    ]));
    let items = company.items().unwrap();
    let item = |number| items.get(ItemId::new(number)).unwrap();
    let cost = |number| {
        let purchase = item(number).purchase.as_ref()?;
        purchase.cost.map(|cost| cost.to_string())
    };
    assert_eq!(cost(1).as_deref(), Some("0.01234"));
    // A service that is bought and sold, and one that is not.
    assert_eq!(cost(2).as_deref(), Some("5.00"));
    let account = item(2).purchase.as_ref().unwrap().account;
    assert_eq!(account, Some(AccountId::new(7)));
    assert!(item(2).stock.is_none() && item(3).purchase.is_none());
    assert_eq!(price(item(4)).as_deref(), Some("-10%"));
    let part = item(1).purchase.as_ref().unwrap().cost.unwrap();
    assert_eq!(part.currency(), CurrencyId::COMPANY);
}

//! The small lists.

use qbw::lists::{ListKind, PaymentMethodKind, PriceLevelKind, TemplateForm, TermsKind};
use qbw::{
    ClassId, CurrencyId, ItemId, List, ListEntry, ListId, NameId, Price, Problem, SiteId, TermsId,
    TransactionId,
};
use qbw_synthetic::{Cell, SyntheticTable};

use crate::small_lists::{Kind, State};
use crate::{small_lists, support};

fn names<T: ListEntry>(list: &List<T>) -> Vec<&str> {
    list.iter().map(ListEntry::name).collect()
}

fn full_names<T: ListEntry>(list: &List<T>) -> Vec<&str> {
    let names = list.iter().map(|entry| list.full_name(entry.id()));
    names.map(Option::unwrap).collect()
}

#[test]
fn every_list_has_the_entries_of_the_books() {
    let company = support::books();
    assert_eq!(names(company.customer_types().unwrap()), ["Retail"]);
    assert_eq!(names(company.vendor_types().unwrap()), ["Supplier"]);
    assert_eq!(names(company.job_types().unwrap()), ["Building"]);
    assert_eq!(names(company.categories().unwrap()), ["Uncategorized"]);
    assert_eq!(names(company.ship_methods().unwrap()), ["Courier"]);
    assert_eq!(names(company.payment_methods().unwrap()), ["Check", "Wire"]);
    assert_eq!(names(company.sales_reps().unwrap()), ["EM"]);
    assert_eq!(names(company.sales_tax_codes().unwrap()), ["Non", "Tax"]);
    assert_eq!(names(company.price_levels().unwrap()), ["Dealers", "Trade"]);
    let terms = names(company.terms().unwrap());
    assert_eq!(terms, ["2% 10 Net 30", "Due on the 5th"]);
    let currencies = names(company.currencies().unwrap());
    assert_eq!(currencies, ["Euro", "US Dollar", "Yen"]);
    let messages = names(company.customer_messages().unwrap());
    assert_eq!(messages, ["Thank you for your order."]);
    assert_eq!(names(company.templates().unwrap()), ["Product invoice"]);
    assert_eq!(names(company.memorized().unwrap()), ["Monthly fuel"]);
    // The books use neither.
    assert!(company.classes().unwrap().is_empty());
    assert!(company.sites().unwrap().is_empty());
}

#[test]
fn every_small_list_can_be_shown_the_same_way() {
    let company = support::books();
    let mut shown = 0;
    for &kind in ListKind::ALL {
        let list = company.small_list(kind).unwrap();
        assert_eq!(list.kind, kind);
        assert!(list.left_out.is_nothing(), "{kind}");
        for entry in &list.entries {
            assert_eq!(entry.depth, 0, "{kind}");
            let row = company.stored_row(entry.origin).unwrap();
            assert!(row.is_some(), "{kind}");
            shown += 1;
        }
    }
    assert_eq!(shown, 20);
    let currencies = company.small_list(ListKind::Currencies).unwrap();
    let entries = currencies.entries.iter();
    let names: Vec<_> = entries.map(|entry| (entry.name, entry.is_active)).collect();
    assert_eq!(names, [("Euro", true), ("US Dollar", true), ("Yen", false)]);
    let mut names: Vec<String> = ListKind::ALL.iter().map(ListKind::to_string).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), ListKind::ALL.len());
}

#[test]
fn a_list_is_walked_as_a_tree() {
    let company = support::books();
    let accounts = company.accounts().unwrap();
    let walked: Vec<(usize, &str)> = accounts
        .walk()
        .map(|(depth, account)| (depth, account.name.as_str()))
        .collect();
    assert_eq!(walked.len(), accounts.len());
    let sales = walked.iter().position(|row| *row == (0, "Sales")).unwrap();
    assert_eq!(walked[sales + 1], (1, "Parts"));
    let parts = accounts.named("Sales:Parts").unwrap().id;
    assert_eq!(accounts.own_name(parts), Some("Parts"));
    assert_eq!(accounts.full_name(parts), Some("Sales:Parts"));
    let names = company.names().unwrap();
    let annex = NameId::new(103);
    assert_eq!(names.own_name(annex), Some("Annex"));
    assert_eq!(names.full_name(annex), Some("Ada Stores:Annex"));
    assert_eq!(names.own_name(NameId::new(201)), Some("Cobalt Parts"));
    assert_eq!(names.own_name(NameId::new(99)), None);
    let origins = accounts
        .iter()
        .map(|account| (account.origin(), account.origin));
    assert!(origins.into_iter().all(|(asked, field)| asked == field));
}

/// The numbers of the entries of a list, whichever list it is.
fn numbers<T: ListEntry>(list: &List<T>) -> Vec<i32> {
    list.iter().map(|entry| entry.id().number()).collect()
}

#[test]
fn an_id_of_any_list_is_a_number() {
    let company = support::books();
    assert_eq!(numbers(company.terms().unwrap()), [1, 2]);
    assert_eq!(numbers(company.currencies().unwrap()), [2, 1, 3]);
    let customers = company.names().unwrap().customers();
    assert_eq!(numbers(customers), [101, 103, 102]);
}

#[test]
fn every_entry_leads_to_its_stored_row() {
    let company = support::books();
    let origins = [
        company.terms().unwrap().iter().next().unwrap().origin,
        company.currencies().unwrap().iter().next().unwrap().origin,
        company.categories().unwrap().iter().next().unwrap().origin,
        company
            .price_levels()
            .unwrap()
            .iter()
            .next()
            .unwrap()
            .origin,
    ];
    for origin in origins {
        let row = company.stored_row(origin).unwrap().unwrap();
        assert_eq!(row.id(), origin.row);
    }
    let terms = company.terms().unwrap().named("2% 10 Net 30").unwrap();
    let row = company.stored_row(terms.origin).unwrap().unwrap();
    assert_eq!(row.get("terms_id").unwrap().as_i64(), Some(1));
}

#[test]
fn terms_say_when_a_form_is_due() {
    let company = support::books();
    let terms = company.terms().unwrap();
    let net = terms.get(TermsId::new(1)).unwrap();
    assert_eq!((net.kind, net.due_days), (TermsKind::Standard, 30));
    assert_eq!(net.discount_days, 10);
    assert_eq!(net.discount_percent, "2".parse().ok());
    let day = |text: &str| text.parse().unwrap();
    assert_eq!(net.due_date(day("2023-11-15")), Some(day("2023-12-15")));
    assert_eq!(net.due_date(day("2024-01-31")), Some(day("2024-03-01")));

    let fifth = terms.get(TermsId::new(2)).unwrap();
    assert_eq!((fifth.kind, fifth.due_day), (TermsKind::DateDriven, 5));
    assert_eq!((fifth.next_month_days, fifth.discount_day), (10, 1));
    assert_eq!(fifth.due_date(day("2024-01-20")), Some(day("2024-02-05")));
    assert_eq!(fifth.due_date(day("2024-01-30")), Some(day("2024-03-05")));
}

#[test]
fn methods_codes_and_templates_have_their_kinds() {
    let company = support::books();
    let methods = company.payment_methods().unwrap();
    let kinds = methods.iter().map(|method| method.kind);
    let kinds: Vec<PaymentMethodKind> = kinds.collect();
    assert_eq!(
        kinds,
        [PaymentMethodKind::Check, PaymentMethodKind::CashOrOther]
    );
    let codes = company.sales_tax_codes().unwrap();
    let taxed = codes.named("tax").unwrap();
    assert!(taxed.is_taxable && !codes.named("non").unwrap().is_taxable);
    assert_eq!(taxed.description.as_deref(), Some("Taxable sales"));
    let template = company.templates().unwrap().named("Product invoice");
    assert_eq!(template.unwrap().form, TemplateForm::Invoice);
    let fuel = company.memorized().unwrap().named("Monthly fuel").unwrap();
    assert_eq!(fuel.transaction, Some(TransactionId::new(1900)));
    assert_eq!((fuel.parent, fuel.is_active), (None, true));
}

#[test]
fn a_sales_rep_is_one_of_the_names() {
    let company = support::books();
    let rep = company.sales_reps().unwrap().named("EM").unwrap();
    assert_eq!(rep.person, Some(NameId::new(301)));
    let names = company.names().unwrap();
    let person = rep.person.and_then(|person| names.full_name(person));
    assert_eq!(person, Some("Eve Moss"));
}

#[test]
fn a_price_level_changes_every_price_or_has_its_own() {
    let company = support::books();
    let levels = company.price_levels().unwrap();
    let trade = levels.named("Trade").unwrap();
    assert_eq!(trade.kind, PriceLevelKind::FixedPercentage);
    assert_eq!(trade.percent, "-5".parse().ok());
    assert!(trade.items.is_empty());

    let dealers = levels.named("Dealers").unwrap();
    assert_eq!(dealers.kind, PriceLevelKind::PerItem);
    assert_eq!(
        (dealers.percent, dealers.currency),
        (None, CurrencyId::new(1))
    );
    let [valve] = dealers.items.as_slice() else {
        panic!("one price: {:?}", dealers.items);
    };
    assert_eq!(valve.item, ItemId::new(502));
    let Some(Price::Each(price)) = valve.price else {
        panic!("so much for one: {valve:?}");
    };
    assert_eq!(
        (price.to_string(), price.currency()),
        ("45.00".to_owned(), CurrencyId::new(1))
    );
}

#[test]
fn a_price_of_a_level_is_in_the_level_s_currency_or_a_share() {
    let levels = small_lists::table(
        "abmc_price_level",
        [
            ("price_level_id", Kind::Whole),
            ("name", Kind::Text),
            ("price_level_type", Kind::Whole),
            ("currency_id", Kind::Whole),
        ],
        [[
            Cell::integer(1),
            Cell::text("Abroad"),
            Cell::integer(1),
            Cell::integer(2),
        ]],
    );
    let row = |item, price, percent| [Cell::integer(1), Cell::integer(item), price, percent];
    let items = small_lists::table(
        "abmc_price_level_item",
        [
            ("price_level_id", Kind::Whole),
            ("item_id", Kind::Whole),
            ("computed_price_amt", Kind::Number),
            ("computed_price_pct", Kind::Number),
        ],
        [
            row(9, Cell::Null, Cell::decimal("-3.00")),
            row(8, Cell::decimal("7.50"), Cell::Null),
        ],
    );
    let company = small_lists::company([levels, items]);
    let abroad = company.price_levels().unwrap().named("Abroad").unwrap();
    assert_eq!(abroad.currency, CurrencyId::new(2));
    let [first, second] = abroad.items.as_slice() else {
        panic!("two prices: {:?}", abroad.items);
    };
    let Some(Price::Each(price)) = first.price else {
        panic!("so much for one: {first:?}");
    };
    assert_eq!(
        (first.item.number(), price.currency()),
        (8, abroad.currency)
    );
    let percent = "-3".parse().unwrap();
    assert_eq!(second.price, Some(Price::Percent(percent)));
}

#[test]
fn a_currency_has_its_symbol_and_its_rates() {
    let company = support::books();
    let currencies = company.currencies().unwrap();
    let dollar = currencies.get(CurrencyId::new(1)).unwrap();
    assert_eq!((dollar.code.as_str(), dollar.is_active), ("USD", true));
    assert_eq!(dollar.symbol.as_deref(), Some("$"));
    assert!(dollar.rates.is_empty());

    let euro = currencies.named("euro").unwrap();
    assert_eq!((euro.id, euro.code.as_str()), (CurrencyId::new(2), "EUR"));
    // The bytes are UTF-8 in a file of windows-1252.
    assert_eq!(euro.symbol.as_deref(), Some("\u{20ac}"));
    let rates: Vec<(String, f32)> = euro
        .rates
        .iter()
        .map(|rate| (rate.as_of.to_string(), rate.rate))
        .collect();
    assert_eq!(
        rates,
        [
            ("2024-02-10".to_owned(), 1.10),
            ("2024-03-10".to_owned(), 1.20)
        ]
    );
    let rate_on = |day: &str| euro.rate_on(day.parse().unwrap()).map(|rate| rate.rate);
    assert_eq!(rate_on("2024-02-09"), None);
    assert_eq!(rate_on("2024-02-10"), Some(1.10));
    assert_eq!(rate_on("2024-03-09"), Some(1.10));
    assert_eq!(rate_on("2024-12-31"), Some(1.20));

    let yen = currencies.named("Yen").unwrap();
    assert_eq!(yen.symbol.as_deref(), Some("\u{a5}"));
    assert_eq!((yen.decimal_places, yen.is_active), (Some(0), false));
    assert!(
        company
            .currencies()
            .unwrap()
            .get(CurrencyId::COMPANY)
            .is_none()
    );
}

#[test]
fn a_symbol_that_is_not_utf_8_stays_as_it_is_stored() {
    let row = |id, code, symbol: &[u8]| {
        [
            Cell::integer(id),
            Cell::text(code),
            Cell::text(code),
            Cell::bytes(symbol),
        ]
    };
    let currencies = small_lists::table(
        "abmc_currency",
        [
            ("currency_id", Kind::Whole),
            ("name", Kind::Text),
            ("code", Kind::Text),
            ("symbol", Kind::Text),
        ],
        [
            // One byte of windows-1252, which is no text in UTF-8.
            row(1, "GBP", b"\xa3"),
            row(2, "INR", b"Rs"),
            row(3, "KRW", b"\xe2\x82\xa9"),
        ],
    );
    let company = small_lists::company([currencies]);
    let symbols = company.currencies().unwrap().iter();
    let symbols: Vec<Option<&str>> = symbols.map(|entry| entry.symbol.as_deref()).collect();
    assert_eq!(symbols, [Some("\u{a3}"), Some("Rs"), Some("\u{20a9}")]);
}

#[test]
fn a_deleted_rate_is_left_out() {
    let currencies = small_lists::table(
        "abmc_currency",
        [
            ("currency_id", Kind::Whole),
            ("name", Kind::Text),
            ("code", Kind::Text),
        ],
        [[Cell::integer(2), Cell::text("Euro"), Cell::text("EUR")]],
    );
    let row = |currency, state: State| {
        let [_, is_deleted] = state.flags();
        [currency, Cell::day("2024-02-10"), Cell::Null, is_deleted]
    };
    let rates = small_lists::table(
        "abmc_exchange_rate_history",
        [
            ("currency_id", Kind::Whole),
            ("as_of_date", Kind::Day),
            ("exchange_rate", Kind::Number),
            ("is_deleted_bool", Kind::Flag),
        ],
        [
            row(Cell::integer(2), State::Deleted),
            row(Cell::Null, State::Shown),
        ],
    );
    let company = small_lists::company([currencies, rates]);
    let currencies = company.currencies().unwrap();
    assert!(currencies.named("Euro").unwrap().rates.is_empty());
    assert!(currencies.left_out().is_nothing());
}

/// The table of classes, whose rows have an id, a name, the class above, the
/// class after, and their state.
fn classes(rows: &[(i32, &str, i32, i32, State)]) -> SyntheticTable {
    let rows = rows.iter().map(|&(id, name, parent, next, state)| {
        let [is_hidden, is_deleted] = state.flags();
        [
            Cell::integer(id),
            Cell::text(name),
            Cell::integer(parent),
            Cell::integer(next),
            is_hidden,
            is_deleted,
        ]
    });
    let columns = [
        ("class_id", Kind::Whole),
        ("name", Kind::Text),
        ("parent_id", Kind::Whole),
        ("display_index_next", Kind::Whole),
        ("is_hidden_bool", Kind::Flag),
        ("is_deleted_bool", Kind::Flag),
    ];
    small_lists::table("abmc_class", columns, rows)
}

#[test]
fn classes_nest_and_keep_their_order() {
    let company = small_lists::company([classes(&[
        (1, "North", 0, -4, State::Shown),
        (2, "South", 0, 3, State::Shown),
        (3, "", 0, 1, State::Deleted),
        (4, "Coast", 1, 5, State::Hidden),
        (5, "Hills", 1, 2, State::Shown),
    ])]);
    let classes = company.classes().unwrap();
    let in_order = ["North", "North:Coast", "North:Hills", "South"];
    assert_eq!(full_names(classes), in_order);
    let active: Vec<&str> = classes.active().map(ListEntry::name).collect();
    assert_eq!(active, ["North", "Hills", "South"]);
    let north = ClassId::new(1);
    assert_eq!(classes.children(Some(north)).count(), 2);
    assert!(classes.get(ClassId::new(3)).is_none());
    let shown = company.small_list(ListKind::Classes).unwrap().entries;
    let shown: Vec<_> = shown
        .iter()
        .map(|entry| (entry.depth, entry.name, entry.is_active))
        .collect();
    let expected = [
        (0, "North", true),
        (1, "Coast", false),
        (1, "Hills", true),
        (0, "South", true),
    ];
    assert_eq!(shown, expected);
    let entries = company.small_list(ListKind::Classes).unwrap().entries;
    let entries = entries.iter();
    let names: Vec<_> = entries
        .map(|entry| (entry.number, entry.full_name, entry.parent))
        .collect();
    let expected = [
        (1, "North", None),
        (4, "North:Coast", Some(1)),
        (5, "North:Hills", Some(1)),
        (2, "South", None),
    ];
    assert_eq!(names, expected);
}

#[test]
fn a_list_without_a_stored_order_is_ordered_by_name() {
    let row = |id, name, state: State| {
        let [is_hidden, is_deleted] = state.flags();
        [Cell::integer(id), Cell::text(name), is_hidden, is_deleted]
    };
    let columns = [
        ("shipping_id", Kind::Whole),
        ("name", Kind::Text),
        ("is_hidden_bool", Kind::Flag),
        ("is_deleted_bool", Kind::Flag),
    ];
    let methods = small_lists::table(
        "abmc_shipping",
        columns,
        [
            row(1, "truck", State::Shown),
            row(2, "Rail", State::Hidden),
            row(3, "Air", State::Deleted),
            row(4, "Boat", State::Shown),
        ],
    );
    let company = small_lists::company([methods]);
    let methods = company.ship_methods().unwrap();
    assert_eq!(names(methods), ["Boat", "Rail", "truck"]);
    assert!(!methods.named("rail").unwrap().is_active);
}

#[test]
fn bins_are_below_their_site() {
    let row = |id, name, parent, special| {
        [
            Cell::integer(id),
            Cell::text(name),
            parent,
            Cell::integer(special),
        ]
    };
    let columns = [
        ("site_id", Kind::Whole),
        ("name", Kind::Text),
        ("parent_id", Kind::Whole),
        ("special_type", Kind::Whole),
    ];
    let sites = small_lists::table(
        "abmc_site",
        columns,
        [
            row(1, "Yard", Cell::Null, 0),
            row(2, "Shelf", Cell::integer(1), 0),
            row(3, "Road", Cell::Null, 2),
        ],
    );
    let company = small_lists::company([sites]);
    let sites = company.sites().unwrap();
    assert_eq!(full_names(sites), ["Yard", "Yard:Shelf", "Road"]);
    let shelf = sites.named("Shelf").unwrap();
    assert_eq!((shelf.parent, shelf.special), (Some(SiteId::new(1)), None));
    assert_eq!(sites.named("Road").unwrap().special, Some(2));
}

#[test]
fn a_file_that_lacks_a_list_has_an_empty_one() {
    let company = small_lists::company([]);
    assert!(company.terms().unwrap().is_empty());
    assert!(company.currencies().unwrap().is_empty());
    assert!(company.classes().unwrap().is_empty());
    assert!(company.price_levels().unwrap().is_empty());
    assert!(company.memorized().unwrap().left_out().is_nothing());
}

#[test]
fn an_entry_that_cannot_be_read_is_left_out_and_counted() {
    let row = |id, form| [Cell::integer(id), Cell::text("Plain"), form];
    let columns = [
        ("custom_style_id", Kind::Whole),
        ("name", Kind::Text),
        ("style_view_type", Kind::Whole),
    ];
    let rows = [row(1, Cell::integer(5)), row(2, Cell::Null)];
    let templates = small_lists::table("abmc_custom_style", columns, rows);
    let company = small_lists::company([templates]);
    let templates = company.templates().unwrap();
    assert_eq!((templates.len(), templates.left_out().rows), (1, 1));
    let [damage] = templates.damage() else {
        panic!("one template is damaged: {:?}", templates.damage());
    };
    assert!(matches!(
        damage.problem,
        Problem::Column {
            column: "style_view_type",
            ..
        }
    ));
}

#[test]
fn a_damaged_price_counts_against_the_price_levels() {
    let levels = small_lists::table(
        "abmc_price_level",
        [("price_level_id", Kind::Whole), ("name", Kind::Text)],
        [[Cell::integer(1), Cell::text("Trade")]],
    );
    let items = small_lists::table(
        "abmc_price_level_item",
        [("price_level_id", Kind::Whole), ("item_id", Kind::Whole)],
        [[Cell::integer(1), Cell::Null]],
    );
    let company = small_lists::company([levels, items]);
    let levels = company.price_levels().unwrap();
    assert_eq!((levels.len(), levels.left_out().rows), (1, 1));
    assert_eq!(levels.damage()[0].table, "abmc_price_level_item");
}

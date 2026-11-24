//! The order of the lists, against what it is known to be in QuickBooks.

use qbw::ListEntry;
use qbw::items::ItemKind;

use crate::agreement::Agreement;
use crate::books;

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn accounts_are_in_the_order_of_their_types() {
    for company in books::books() {
        let accounts = company.accounts().unwrap();
        let types: Vec<i64> = accounts
            .iter()
            .map(|account| account.account_type.code())
            .collect();
        let mut pairs = Agreement::default();
        for pair in types.windows(2) {
            pairs.add(pair[0] <= pair[1]);
        }
        pairs.check("accounts after one of their type or an earlier one");
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn items_of_a_kind_are_together() {
    for company in books::books() {
        let items = company.items().unwrap();
        let kinds: Vec<ItemKind> = items.iter().map(|item| item.kind).collect();
        let mut first_of_each = kinds.clone();
        first_of_each.dedup();
        let mut distinct: Vec<i64> = kinds.iter().map(|kind| kind.code()).collect();
        distinct.sort_unstable();
        distinct.dedup();
        println!("{} kinds of items", distinct.len());
        assert_eq!(first_of_each.len(), distinct.len());
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn labels_at_the_top_are_in_the_order_of_their_names() {
    for company in books::books() {
        let customer_types = company.customer_types().unwrap().children(None);
        let vendor_types = company.vendor_types().unwrap().children(None);
        let categories = company.categories().unwrap().children(None);
        let mut pairs = Agreement::default();
        let lists: [Vec<&str>; 3] = [
            customer_types.map(ListEntry::name).collect(),
            vendor_types.map(ListEntry::name).collect(),
            categories.map(ListEntry::name).collect(),
        ];
        for pair in lists.iter().flat_map(|names| names.windows(2)) {
            pairs.add(pair[0].to_ascii_lowercase() <= pair[1].to_ascii_lowercase());
        }
        pairs.check("labels after one of a lower name");
    }
}

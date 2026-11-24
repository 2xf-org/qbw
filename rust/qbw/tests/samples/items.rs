//! Items.

use qbw::ItemId;
use qbw::items::{Item, ItemKind};

use crate::agreement::Agreement;
use crate::share::Share;
use crate::{agreement, books, chain};

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn items_read_and_lead_to_their_rows() {
    for company in books::books() {
        let items = company.items().unwrap();
        println!("{} items", items.len());
        assert!(items.left_out().is_nothing() && !items.is_empty());
        for item in items {
            assert!(!item.name.is_empty());
            assert!(!matches!(item.kind, ItemKind::Other(_)));
            let row = company.stored_row(item.origin).unwrap().unwrap();
            assert_eq!(row.id(), item.origin.row);
        }
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn one_chain_goes_through_every_item() {
    for company in books::books() {
        let links = chain::links(company, "abmc_item_internal", "item_id");
        let chain = chain::chain(&links);
        println!("{} of {} items are in the chain", chain.len(), links.len());
        assert_eq!(chain.len(), links.len());
        let items = company.items().unwrap();
        let chain = chain.into_iter().map(ItemId::new);
        let chain: Vec<ItemId> = chain.filter(|&id| items.get(id).is_some()).collect();
        let order: Vec<ItemId> = items.iter().map(|item| item.id).collect();
        assert!(chain == order, "the list is in another order");
    }
}

/// Whether the accounts of an item are the ones that its kind has.
fn has_the_accounts_of_its_kind(item: &Item, is_pass_through: bool) -> bool {
    let stock = item.stock.as_ref().and_then(|stock| stock.asset_account);
    let purchase = item.purchase.as_ref().and_then(|purchase| purchase.account);
    let is_bought = item.kind.is_stocked() || is_pass_through;
    let posts = !matches!(
        item.kind,
        ItemKind::Subtotal | ItemKind::Group | ItemKind::SalesTaxGroup
    );
    stock.is_some() == item.kind.is_stocked()
        && purchase.is_some() == is_bought
        && (posts || item.sales.account.is_none())
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn the_accounts_of_an_item_follow_its_kind() {
    let columns = [
        "item_id",
        "is_pass_thru_bool",
        "cogs_account_id",
        "asset_account_id",
    ];
    for company in books::books() {
        let items = company.items().unwrap();
        let mut accounts = Agreement::default();
        let mut without_sales_account = 0;
        for row in agreement::rows(company, "abmc_item_user", &columns) {
            let id = ItemId::new(row.read(0).unwrap());
            let is_pass_through: bool = row.read(1).unwrap();
            let Some(item) = items.get(id) else {
                continue;
            };
            // The record leaves out an account that its kind does not have.
            let stored = [2, 3].map(|column| row.read::<i32>(column).unwrap() != 0);
            let is_bought = item.kind.is_stocked() || is_pass_through;
            let as_stored = stored == [is_bought, item.kind.is_stocked()];
            accounts.add(as_stored && has_the_accounts_of_its_kind(item, is_pass_through));
            without_sales_account += usize::from(item.sales.account.is_none());
        }
        println!("{without_sales_account} items without an account for sales");
        accounts.check("items with the accounts of their kind");
    }
}

/// The items of the three books that have no price: subtotals, groups, payment
/// items and some that are sold without one.
const WITHOUT_PRICE: usize = 28;

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn one_column_at_most_has_the_price_of_an_item() {
    let columns = [
        "item_id",
        "sales_price_amt",
        "sales_price_qnty",
        "sales_price_pct",
    ];
    let mut without_price = 0;
    for company in books::books() {
        let items = company.items().unwrap();
        let mut prices = Agreement::default();
        for row in agreement::rows(company, "abmc_item_user", &columns) {
            let id = ItemId::new(row.read(0).unwrap());
            let Some(item) = items.get(id) else {
                continue;
            };
            let filled = (1..4).filter(|&column| !row.value(column).unwrap().is_null());
            let filled = filled.count();
            prices.add(filled <= 1 && item.sales.price.is_some() == (filled == 1));
            without_price += usize::from(filled == 0);
        }
        prices.check("items with one price at most");
    }
    println!("{without_price} items without a price");
    assert!(without_price <= WITHOUT_PRICE);
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn items_in_stock_have_the_stock_that_quickbooks_keeps() {
    for company in books::books() {
        let mut stock = Agreement::default();
        let mut sites = Share::default();
        let site_list = company.sites().unwrap();
        for item in company.items().unwrap() {
            stock.add(item.stored.is_some() == item.kind.is_stocked());
            let (Some(stored), false) = (item.stored, item.sites.is_empty()) else {
                continue;
            };
            let at_sites = item.sites.iter().filter(|at| {
                let site = at.site.and_then(|site| site_list.get(site));
                site.is_some_and(|site| site.parent.is_none())
            });
            let mut on_hand = at_sites.filter_map(|at| at.on_hand);
            let first = on_hand.next().unwrap_or_default();
            let sum = on_hand.fold(first, |sum, more| sum.checked_add(more).unwrap());
            sites.add(sum == stored.on_hand);
        }
        stock.check("items whose stock follows their kind");
        sites.check("items whose sites add up to the stock", 99);
    }
}

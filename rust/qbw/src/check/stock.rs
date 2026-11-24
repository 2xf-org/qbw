//! Comparing the stock with what QuickBooks keeps and with the ledger.

use std::collections::{BTreeMap, HashMap};

use sa17::Decimal;

use crate::amount::Total;
use crate::as_of::AsOf;
use crate::check::tally::Figures;
use crate::check::{Cause, CheckKind, Subject, Tally};
use crate::codes::TransactionKind;
use crate::ids::{AccountId, CurrencyId, ItemId};
use crate::inventory::{SiteStock, Stock, StockRow};
use crate::items::{Item, ItemKind, StoredStock};
use crate::ledger::{Entry, Ledger, PostingSource};
use crate::left_out::LeftOut;
use crate::list::List;
use crate::lists::Site;
use crate::money::Money;

/// What comparing the stock takes.
pub(crate) struct StockCheck<'company> {
    pub(crate) stock: &'company Stock,
    pub(crate) items: &'company List<Item>,
    pub(crate) sites: &'company List<Site>,
    pub(crate) ledger: &'company Ledger,
}

impl StockCheck<'_> {
    /// Whether the stock is what QuickBooks keeps, and is worth what was posted.
    pub(crate) fn check(&self, tally: &mut Tally) {
        for row in &self.stock.rows {
            self.item(tally, row);
            for at in &row.sites {
                self.site(tally, row, at);
            }
        }
        self.postings(tally);
    }

    /// Compares an item with what QuickBooks keeps with it.
    fn item(&self, tally: &mut Tally, row: &StockRow) {
        let left_out = self.stock.left_out;
        let mut compare = |what, figures, causes: fn(LeftOut) -> Vec<Cause>| {
            let subject = Subject::Item(row.item);
            tally.compare(what, subject, figures, || causes(left_out));
        };
        let counted = Figures::new(row.counted, row.on_hand);
        compare(CheckKind::ItemCount, counted, Cause::of_missing);
        let Some(stored) = row.stored else {
            return;
        };
        let quantity = Figures::new(row.on_hand, stored.on_hand);
        compare(CheckKind::ItemQuantity, quantity, Cause::of_missing);
        if let Some(value) = stored.value {
            let value = Figures::new(row.value, value);
            compare(CheckKind::ItemValue, value, Cause::of_missing);
        }
        for (what, computed, expected) in orders(row, &stored) {
            let figures = Figures {
                computed: Some(computed.into()),
                expected: expected.map(Into::into),
            };
            compare(what, figures, Cause::of_kept);
        }
    }

    /// Compares what there is of an item at a site with what QuickBooks keeps.
    fn site(&self, tally: &mut Tally, row: &StockRow, at: &SiteStock) {
        let causes = || {
            let kept = row.stored.map(|stored| stored.on_hand);
            let disagree = kept_at_sites(&row.sites, self.sites) != kept;
            let mut causes = Cause::of_missing(self.stock.left_out);
            causes.extend(disagree.then_some(Cause::SitesDisagree));
            causes
        };
        let subject = Subject::ItemAt(row.item, at.site);
        let figures = Figures::new(at.on_hand, at.stored.unwrap_or_default());
        tally.compare(CheckKind::SiteQuantity, subject, figures, causes);
    }

    /// Compares what the items are worth with what was posted for them, and with
    /// the balances of the accounts that they are kept on.
    fn postings(&self, tally: &mut Tally) {
        let home = self.ledger.home();
        let (posted, others) = self.posted();
        let mut assets: BTreeMap<AccountId, Vec<Money>> = BTreeMap::new();
        for row in &self.stock.rows {
            let posted = posted.get(&row.item).copied().unwrap_or_default();
            self.value(tally, row, posted.money(home));
            let item = self.items.get(row.item);
            let settings = item.and_then(|item| item.stock.as_ref());
            if let Some(account) = settings.and_then(|settings| settings.asset_account) {
                assets.entry(account).or_default().push(row.value);
            }
        }
        for (account, values) in assets {
            let others = others.get(&account).copied().unwrap_or_default();
            self.asset_account(tally, account, values, others.money(home));
        }
    }

    /// Compares what an item is worth with what was posted for it.
    fn value(&self, tally: &mut Tally, row: &StockRow, posted: Money) {
        let item = self.items.get(row.item);
        let is_built = item.is_some_and(|item| item.kind == ItemKind::Assembly);
        let subject = Subject::Item(row.item);
        let figures = Figures::new(posted, row.value);
        tally.compare(CheckKind::ItemPostings, subject, figures, || {
            let mut causes = Cause::of_missing(self.ledger.left_out());
            causes.extend(is_built.then_some(Cause::BuildCosts));
            causes
        });
    }

    /// Compares the balance of an account with the `values` that are kept on it.
    /// `others` is what was posted to it without the stock.
    fn asset_account(
        &self,
        tally: &mut Tally,
        account: AccountId,
        values: Vec<Money>,
        others: Money,
    ) {
        let home = self.ledger.home();
        let balance = self.ledger.balance(account, AsOf::Latest);
        let balance = balance.map_or(Money::zero(home), |balance| balance.home_amount);
        let figures = Figures {
            computed: sum(values, home).map(Into::into),
            expected: Some(balance.into()),
        };
        let subject = Subject::Account(account);
        tally.compare(CheckKind::InventoryAsset, subject, figures, || {
            let mut causes = Cause::of_missing(self.ledger.left_out());
            let others = Some(others).filter(|others| !others.is_zero());
            causes.extend(others.map(Cause::OtherPostings));
            causes
        });
    }

    /// What was posted for each item in stock, and what else was posted to each
    /// account.
    fn posted(&self) -> (HashMap<ItemId, Total>, HashMap<AccountId, Total>) {
        let mut items: HashMap<ItemId, Total> = HashMap::new();
        let mut others: HashMap<AccountId, Total> = HashMap::new();
        for entry in self.ledger.entries_by_transaction() {
            let sum = match valued_item(entry, self.items) {
                Some(item) => items.entry(item).or_default(),
                None => others.entry(entry.account).or_default(),
            };
            *sum += entry.home.into();
        }
        (items, others)
    }
}

/// The item in stock whose value an entry changes.
fn valued_item(entry: &Entry, items: &List<Item>) -> Option<ItemId> {
    let item = items.get(entry.item()?)?;
    (item.kind.is_stocked() && is_of_the_stock(entry)).then_some(item.id)
}

/// Whether an entry with an item in stock is of the stock's value. A sale posts
/// what the customer pays with its line, and the value besides. On the other
/// forms the line itself is of the value.
fn is_of_the_stock(entry: &Entry) -> bool {
    match entry.source {
        PostingSource::StockValue => true,
        PostingSource::OwnLine | PostingSource::Line => matches!(
            entry.kind(),
            TransactionKind::Bill
                | TransactionKind::VendorCredit
                | TransactionKind::ItemReceipt
                | TransactionKind::Check
                | TransactionKind::CreditCardCharge
                | TransactionKind::CreditCardCredit
                | TransactionKind::InventoryAdjustment
                | TransactionKind::InventoryTransfer
                | TransactionKind::AssemblyBuild
        ),
        PostingSource::CostOfGoodsSold | PostingSource::OtherStock => false,
    }
}

/// The figures of an item's orders and builds, each with the one that QuickBooks
/// keeps.
fn orders(row: &StockRow, kept: &StoredStock) -> [(CheckKind, Decimal, Option<Decimal>); 4] {
    [
        (CheckKind::OnOrder, row.on_order, kept.on_order),
        (
            CheckKind::OnSalesOrders,
            row.on_sales_orders,
            kept.on_sales_orders,
        ),
        (CheckKind::ToBeBuilt, row.to_be_built, kept.to_be_built),
        (
            CheckKind::RequiredForBuilds,
            row.required_for_builds,
            kept.required_for_builds,
        ),
    ]
}

/// What amounts in `home` add up to, unless that is too large.
fn sum(amounts: Vec<Money>, home: CurrencyId) -> Option<Money> {
    let mut amounts = amounts.into_iter();
    amounts.try_fold(Money::zero(home), Money::checked_add).ok()
}

/// What QuickBooks keeps for the sites of an item, added up. What it keeps for
/// bins is part of their sites'.
fn kept_at_sites(stock: &[SiteStock], sites: &List<Site>) -> Option<Decimal> {
    let is_site = |at: &&SiteStock| {
        let site = sites.get(at.site);
        site.is_some_and(|site| site.parent.is_none())
    };
    let kept = stock.iter().filter(is_site);
    let mut kept = kept.map(|at| at.stored.unwrap_or_default());
    kept.try_fold(Decimal::default(), Decimal::checked_add)
}

#[cfg(test)]
mod tests {
    use sa17::{RowAddress, RowId};

    use super::*;
    use crate::ids::SiteId;
    use crate::origin::Origin;

    fn site(id: i32, parent: Option<i32>) -> Site {
        let row = RowId::Stored(RowAddress::new(1, 1));
        Site {
            id: SiteId::new(id),
            name: "Yard".to_owned(),
            parent: parent.map(SiteId::new),
            description: None,
            special: None,
            is_active: true,
            origin: Origin::new("sites", row),
        }
    }

    fn at(site: i32, stored: Option<&str>) -> SiteStock {
        SiteStock {
            site: SiteId::new(site),
            on_hand: Decimal::default(),
            reorder_point: None,
            stored: stored.map(|stored| stored.parse().unwrap()),
        }
    }

    #[test]
    fn a_site_has_what_is_in_its_bins() {
        let sites = [site(1, None), site(2, Some(1)), site(3, None)];
        let sites = List::new(sites.into(), Vec::new());
        let stock = [at(1, Some("3")), at(2, Some("3")), at(3, Some("1.5"))];
        assert_eq!(kept_at_sites(&stock, &sites), "4.5".parse().ok());
        assert_eq!(
            kept_at_sites(&[at(3, None)], &sites),
            Some(Decimal::default())
        );
    }
}

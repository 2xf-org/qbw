//! An item.

use crate::codes::{ItemKind, SpecialItem};
use crate::ids::{CategoryId, ClassId, ItemId, OwnerKey, SalesTaxCodeId};
use crate::items::{ItemSite, Member, Purchase, Sales, StockSettings, StoredStock, TaxSettings};
use crate::list::{ListEntry, sealed};
use crate::origin::Origin;
use crate::unix_time::UnixTime;

/// An item: a product, a service, or what else a line of a form can be of.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Item {
    /// What the item is known by.
    pub id: ItemId,
    /// Its own name, without those of the items above it.
    pub name: String,
    /// The item it is a sub-item of.
    pub parent: Option<ItemId>,
    /// Its kind.
    pub kind: ItemKind,
    /// What QuickBooks uses it for, when it made the item itself.
    pub special: Option<SpecialItem>,
    /// What a line of it says and posts to.
    pub sales: Sales,
    /// What it costs, for an item in stock and for one that is bought and
    /// sold.
    pub purchase: Option<Purchase>,
    /// How its stock is kept, for an item in stock.
    pub stock: Option<StockSettings>,
    /// Whose tax it is, for a sales tax.
    pub tax: Option<TaxSettings>,
    /// What a group, an assembly or a tax group is made of, in order.
    pub members: Vec<Member>,
    /// Whether a group prints with its members.
    pub prints_members: bool,
    /// The manufacturer's number for it.
    pub part_number: Option<String>,
    /// Whether it is taxed.
    pub tax_code: Option<SalesTaxCodeId>,
    /// The category it is sorted into.
    pub category: Option<CategoryId>,
    /// Its class. Unchecked.
    pub class: Option<ClassId>,
    /// Whether it can be used in new transactions.
    pub is_active: bool,
    /// The stock that QuickBooks keeps with it, for an item in stock.
    pub stored: Option<StoredStock>,
    /// The stock that QuickBooks keeps with it for each site and bin.
    pub sites: Vec<ItemSite>,
    /// What its custom fields are kept by.
    pub owner_key: Option<OwnerKey>,
    /// What QuickBooks calls it towards other programs.
    pub list_id: Option<String>,
    /// When it was made.
    pub created: Option<UnixTime>,
    /// When it was last changed.
    pub modified: Option<UnixTime>,
    /// The stored row.
    pub origin: Origin,
}

impl sealed::Sealed for Item {}

impl ListEntry for Item {
    type Id = ItemId;

    fn id(&self) -> ItemId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn parent(&self) -> Option<ItemId> {
        self.parent
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn origin(&self) -> Origin {
        self.origin
    }
}

//! Items: what a company buys, sells and taxes.

use sa17::Decimal;

use crate::codes::{ItemKind, SpecialItem};
use crate::ids::{
    AccountId, CategoryId, ClassId, ItemId, NameId, OwnerKey, SalesTaxCodeId, SiteId,
};
#[cfg(test)]
use crate::tables::tests::Declared;
use crate::tables::{Table, stored_row};
use crate::unix_time::UnixTime;

pub(crate) const ITEMS: Table<ItemRow> = Table::new("abmc_item_user");

pub(crate) const ITEM_TOTALS: Table<ItemTotalsRow> = Table::new("abmc_item_internal");

pub(crate) const MEMBERS: Table<MemberRow> = Table::new("abmc_grouped_items");

pub(crate) const ITEM_SITES: Table<ItemSiteRow> = Table::new("abmc_item_site");

stored_row! {
    /// An item as people keep it.
    ///
    /// A price is in one of its three columns at most.
    pub(crate) struct ItemRow {
        required {
            id: ItemId = "item_id",
            name: Option<String> = "name",
            parent: Option<ItemId> = "parent_id",
            kind: ItemKind = "item_type",
            is_hidden: bool = "is_hidden_bool",
            is_deleted: bool = "is_deleted_bool",
        }
        optional {
            /// The income account, or the only account.
            sales_account: Option<AccountId> = "post_account_id",
            /// The cost account of an item in stock, and the expense account
            /// of one that is bought and sold.
            purchase_account: Option<AccountId> = "cogs_account_id",
            asset_account: Option<AccountId> = "asset_account_id",
            /// Whether an item that is not in stock is both bought and sold.
            is_pass_through: bool = "is_pass_thru_bool",
            prints_members: bool = "print_grouped_bool",
            sales_description: Option<String> = "description",
            purchase_description: Option<String> = "purchase_description",
            /// A price of two digits after the point.
            price: Option<Decimal> = "sales_price_amt",
            /// A price of more.
            exact_price: Option<Decimal> = "sales_price_qnty",
            price_percent: Option<Decimal> = "sales_price_pct",
            /// A cost of two digits after the point.
            cost: Option<Decimal> = "unit_cost_amt",
            /// A cost of more.
            exact_cost: Option<Decimal> = "unit_cost_qnty",
            vendor: Option<NameId> = "preferred_vendor_id",
            tax_agency: Option<NameId> = "tax_agency_id",
            tax_code: Option<SalesTaxCodeId> = "sales_tax_code_id",
            part_number: Option<String> = "manufacturer_part_num",
            reorder_point: Option<Decimal> = "reorder_point_qnty",
            build_point: Option<Decimal> = "assembly_build_point_qnty",
            class: Option<ClassId> = "class_id",
            category: Option<CategoryId> = "category_id",
            owner_key: Option<OwnerKey> = "item_abm_id",
            created: Option<UnixTime> = "timestamp_tms",
            modified: Option<UnixTime> = "modtimestamp_tms",
        }
    }
}

stored_row! {
    /// What QuickBooks keeps about an item. The figures of the stock are
    /// nothing on an item that is not kept in stock.
    pub(crate) struct ItemTotalsRow {
        required {
            id: ItemId = "item_id",
        }
        optional {
            special: Option<SpecialItem> = "special_type",
            next: Option<i32> = "display_index_next",
            on_hand: Option<Decimal> = "on_hand_qnty",
            value: Option<Decimal> = "value_on_hand_amt",
            average_cost: Option<Decimal> = "average_cost_qnty",
            on_order: Option<Decimal> = "on_order_qnty",
            on_sales_order: Option<Decimal> = "sales_order_qnty",
            to_be_built: Option<Decimal> = "to_be_built_by_pending_build_txns_qnty",
            required_for_builds: Option<Decimal> = "required_by_pending_build_txns_qnty",
            list_id: Option<String> = "list_ident",
        }
    }
}

stored_row! {
    /// An item that a group, an assembly or a tax group is made of.
    pub(crate) struct MemberRow {
        required {
            of: ItemId = "item_id",
            position: i32 = "position",
            /// Nothing in a row that was left blank.
            item: Option<ItemId> = "sub_item_id",
        }
        optional {
            quantity: Option<Decimal> = "quantity_qnty",
        }
    }
}

stored_row! {
    /// What there is of an item at a site or in a bin.
    pub(crate) struct ItemSiteRow {
        required {
            item: Option<ItemId> = "item_id",
            site: Option<SiteId> = "site_id",
        }
        optional {
            on_hand: Option<Decimal> = "qnty_on_hand",
            reorder_point: Option<Decimal> = "reorder_level",
            is_deleted: bool = "is_deleted_bool",
        }
    }
}

#[cfg(test)]
pub(super) fn declared() -> Vec<Box<dyn Declared>> {
    vec![
        Box::new(ITEMS),
        Box::new(ITEM_TOTALS),
        Box::new(MEMBERS),
        Box::new(ITEM_SITES),
    ]
}

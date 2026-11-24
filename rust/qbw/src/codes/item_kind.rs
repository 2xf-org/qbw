//! The kinds of items.

use crate::code::{codes, words};

codes! {
    /// The kind of an item, which decides what it can be used for.
    pub enum ItemKind {
        /// Work.
        Service = 0,
        /// A part kept in stock.
        InventoryPart = 1,
        /// Something put together out of other items, kept in stock.
        Assembly = 2,
        /// A part not kept in stock.
        NonInventoryPart = 3,
        /// Something the company owns for years. Unchecked.
        FixedAsset = 4,
        /// Another charge, such as for delivery.
        OtherCharge = 5,
        /// The sum of the lines above it.
        Subtotal = 6,
        /// Several items that are put on a form together.
        Group = 7,
        /// A discount.
        Discount = 8,
        /// A payment made at the time of the sale.
        Payment = 9,
        /// A sales tax, owed to one agency.
        SalesTax = 10,
        /// Several sales taxes that are charged together.
        SalesTaxGroup = 11,
    }
}

impl ItemKind {
    /// Whether items of this kind are kept in stock.
    #[must_use]
    pub const fn is_stocked(self) -> bool {
        matches!(self, Self::InventoryPart | Self::Assembly)
    }
}

words! {
    /// Writes the type as the item list has it.
    ItemKind, a "kind of item", or else "Item type" {
        Service => "Service",
        InventoryPart => "Inventory Part",
        Assembly => "Inventory Assembly",
        NonInventoryPart => "Non-inventory Part",
        FixedAsset => "Fixed Asset",
        OtherCharge => "Other Charge",
        Subtotal => "Subtotal",
        Group => "Group",
        Discount => "Discount",
        Payment => "Payment",
        SalesTax => "Sales Tax Item",
        SalesTaxGroup => "Sales Tax Group",
    }
}

//! A field that the company defined.

use crate::codes::TransactionKind;
use crate::ids::CustomFieldId;
use crate::origin::Origin;
use crate::tables::custom_fields::CustomFieldDefinitionRow;

/// Where the positions of the fields of items start.
const FIRST_OF_ITEMS: i32 = 30;

/// A field that the company defined.
///
/// A transaction has the fields of the customers or of the vendors, and a line
/// has those of the items.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct CustomFieldDefinition {
    /// What the field is known by.
    pub id: CustomFieldId,
    /// What the company calls it.
    pub name: String,
    /// Whether customers and jobs have it.
    pub is_for_customers: bool,
    /// Whether vendors have it.
    pub is_for_vendors: bool,
    /// Whether employees have it.
    pub is_for_employees: bool,
    /// Whether items have it.
    pub is_for_items: bool,
    /// The position that the file gives it. The fields of items start at 30.
    pub position: i32,
    /// The stored row.
    pub origin: Origin,
}

impl CustomFieldDefinition {
    pub(super) fn new(origin: Origin, row: CustomFieldDefinitionRow) -> Self {
        Self {
            id: row.id,
            name: row.name.unwrap_or_default(),
            is_for_customers: row.is_for_customers,
            is_for_vendors: row.is_for_vendors,
            is_for_employees: row.is_for_employees,
            is_for_items: row.is_for_items,
            position: row.position,
            origin,
        }
    }

    /// Whether transactions of a kind have it: those of customers are on invoices,
    /// credit memos, estimates and sales orders, and on sales receipts
    /// (unchecked), and those of vendors on purchase orders.
    #[must_use]
    pub const fn is_for_transactions(&self, kind: TransactionKind) -> bool {
        match kind {
            TransactionKind::Invoice
            | TransactionKind::CreditMemo
            | TransactionKind::Estimate
            | TransactionKind::SalesOrder
            | TransactionKind::SalesReceipt => self.is_for_customers,
            TransactionKind::PurchaseOrder => self.is_for_vendors,
            _ => false,
        }
    }

    /// Whether the lines of transactions of a kind have it: those of items are on
    /// the lines of the sales and orders above, of bills and of builds.
    #[must_use]
    pub const fn is_for_lines(&self, kind: TransactionKind) -> bool {
        let has_items = matches!(
            kind,
            TransactionKind::Invoice
                | TransactionKind::CreditMemo
                | TransactionKind::Estimate
                | TransactionKind::SalesOrder
                | TransactionKind::SalesReceipt
                | TransactionKind::PurchaseOrder
                | TransactionKind::Bill
                | TransactionKind::AssemblyBuild
        );
        has_items && self.is_for_items
    }

    /// The position of the field in the records that have it.
    pub(super) const fn position_in_records(&self) -> i32 {
        if self.position < FIRST_OF_ITEMS {
            self.position
        } else {
            self.position - FIRST_OF_ITEMS
        }
    }
}

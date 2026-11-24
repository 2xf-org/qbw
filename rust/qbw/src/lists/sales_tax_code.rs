//! Sales tax codes.

use sa17::Source;

use crate::error::Result;
use crate::ids::SalesTaxCodeId;
use crate::list::{List, ListEntry, sealed};
use crate::lists::reading;
use crate::origin::Origin;
use crate::tables::lists::SALES_TAX_CODES;

/// Whether the customer, the item or the line that it is on is taxed.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SalesTaxCode {
    /// What the code is known by.
    pub id: SalesTaxCodeId,
    /// The code, such as `Tax`, which the list calls it by.
    pub code: String,
    /// What it is for.
    pub description: Option<String>,
    /// Whether what it is on is taxed.
    pub is_taxable: bool,
    /// Whether it can be used in new transactions.
    pub is_active: bool,
    /// The stored row.
    pub origin: Origin,
}

impl sealed::Sealed for SalesTaxCode {}

impl ListEntry for SalesTaxCode {
    type Id = SalesTaxCodeId;

    fn id(&self) -> SalesTaxCodeId {
        self.id
    }

    fn name(&self) -> &str {
        &self.code
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn origin(&self) -> Origin {
        self.origin
    }
}

/// Reads the sales tax codes.
///
/// # Errors
///
/// Returns an error when their table lacks a column.
pub(crate) fn read(source: &dyn Source) -> Result<List<SalesTaxCode>> {
    reading::by_name(source, &SALES_TAX_CODES, |origin, row| {
        (!row.is_deleted).then(|| SalesTaxCode {
            id: row.id,
            code: row.code.unwrap_or_default(),
            description: row.description,
            is_taxable: row.is_taxable,
            is_active: !row.is_hidden,
            origin,
        })
    })
}

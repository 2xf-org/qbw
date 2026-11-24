//! Payment methods.

use sa17::Source;

use crate::codes::PaymentMethodKind;
use crate::error::Result;
use crate::ids::PaymentMethodId;
use crate::list::{List, ListEntry, sealed};
use crate::lists::reading;
use crate::origin::Origin;
use crate::tables::lists::PAYMENT_METHODS;

/// A way that customers pay.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct PaymentMethod {
    /// What the method is known by.
    pub id: PaymentMethodId,
    /// Its name.
    pub name: String,
    /// Its kind.
    pub kind: PaymentMethodKind,
    /// Whether it can be used in new transactions.
    pub is_active: bool,
    /// The stored row.
    pub origin: Origin,
}

impl sealed::Sealed for PaymentMethod {}

impl ListEntry for PaymentMethod {
    type Id = PaymentMethodId;

    fn id(&self) -> PaymentMethodId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn origin(&self) -> Origin {
        self.origin
    }
}

/// Reads the payment methods.
///
/// # Errors
///
/// Returns an error when their table lacks a column.
pub(crate) fn read(source: &dyn Source) -> Result<List<PaymentMethod>> {
    reading::by_name(source, &PAYMENT_METHODS, |origin, row| {
        (!row.is_deleted).then(|| PaymentMethod {
            id: row.id,
            name: row.name.unwrap_or_default(),
            kind: row.kind.unwrap_or_default(),
            is_active: !row.is_hidden,
            origin,
        })
    })
}

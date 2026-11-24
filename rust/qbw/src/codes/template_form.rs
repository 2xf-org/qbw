//! The forms that templates are for.

use crate::code::codes;

codes! {
    /// The form that a template is for.
    pub enum TemplateForm {
        /// An invoice.
        Invoice = 0,
        /// A credit memo.
        CreditMemo = 1,
        /// A purchase order.
        PurchaseOrder = 3,
        /// An estimate.
        Estimate = 5,
        /// A sales order.
        SalesOrder = 6,
        /// A build of an assembly.
        AssemblyBuild = 7,
    }
}

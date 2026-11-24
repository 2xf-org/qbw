//! The list that a name is in.

/// The list that a name is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum NameKind {
    /// The customers and their jobs.
    Customer,
    /// The vendors.
    Vendor,
    /// The employees.
    Employee,
    /// The other names.
    Other,
}

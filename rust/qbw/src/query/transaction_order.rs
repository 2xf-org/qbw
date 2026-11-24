//! The order of listed transactions.

/// The order of listed transactions. Those that it puts in the same place are
/// ordered by date, and those of one day as they were entered.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TransactionOrder {
    /// By date.
    #[default]
    Date,
    /// As they were entered.
    Entered,
    /// By number, where `A-9` comes before `A-10`.
    Number,
    /// By the full name of who they are with.
    Name,
    /// By the total in the company's currency.
    Total,
    /// By when they are due.
    DueDate,
}

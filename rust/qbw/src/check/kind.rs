//! What is compared.

use std::fmt;

/// What is compared. The kinds are in the order in which one rests on those
/// before it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CheckKind {
    /// The postings of a transaction come to zero.
    TransactionBalance,
    /// All postings come to zero.
    LedgerBalance,
    /// The lines of a transaction have the date and the kind of its header, and
    /// post when it does.
    HeaderAndLines,
    /// The lines of a transaction are in one chain, from its own line on.
    LineChain,
    /// The lines of a form add up to its total, in its currency.
    FormTotal,
    /// The postings to an account come to the balance that QuickBooks keeps, in
    /// both currencies.
    AccountBalance,
    /// The postings of a customer come to the balance that QuickBooks keeps, in
    /// both currencies.
    CustomerBalance,
    /// The postings of a vendor come to the balance that QuickBooks keeps, in
    /// both currencies.
    VendorBalance,
    /// What a form says is left of it is what its links leave.
    OpenForm,
    /// What the file keeps as open of something that a customer owes, or as not
    /// used of a payment, is its amount less what its links say was settled.
    OpenReceivable,
    /// The same of something that a vendor is owed.
    OpenPayable,
    /// The open items of an account for what customers owe come to its balance,
    /// in its currency.
    ReceivableAccount,
    /// The open items of an account for what vendors are owed come to its
    /// balance, in its currency.
    PayableAccount,
    /// The history of an item ends with the quantity that QuickBooks keeps.
    ItemQuantity,
    /// The lines of an item add up to the quantity that its history ends with.
    ItemCount,
    /// The history of an item ends with the value that QuickBooks keeps.
    ItemValue,
    /// The postings of an item come to the value that its history ends with.
    ItemPostings,
    /// The lines of an item at a site or in a bin add up to the quantity that
    /// QuickBooks keeps.
    SiteQuantity,
    /// What is left of an item's purchase orders is what QuickBooks keeps.
    OnOrder,
    /// What is left of an item's sales orders is what QuickBooks keeps.
    OnSalesOrders,
    /// What pending builds add of an item is what QuickBooks keeps.
    ToBeBuilt,
    /// What pending builds take of an item is what QuickBooks keeps.
    RequiredForBuilds,
    /// What the items in stock are worth is the balance of the account that they
    /// are kept on.
    InventoryAsset,
    /// A figure that QuickBooks worked out of a year is one of the figures of the
    /// fiscal years.
    YearlyFigure,
}

impl CheckKind {
    /// What is compared of the names.
    pub(crate) const NAMES: &'static [Self] = &[Self::CustomerBalance, Self::VendorBalance];

    /// What is compared by the links.
    pub(crate) const LINKS: &'static [Self] = &[
        Self::OpenForm,
        Self::OpenReceivable,
        Self::OpenPayable,
        Self::ReceivableAccount,
        Self::PayableAccount,
    ];

    /// What is compared of the stock.
    pub(crate) const STOCK: &'static [Self] = &[
        Self::ItemQuantity,
        Self::ItemCount,
        Self::ItemValue,
        Self::ItemPostings,
        Self::SiteQuantity,
        Self::OnOrder,
        Self::OnSalesOrders,
        Self::ToBeBuilt,
        Self::RequiredForBuilds,
        Self::InventoryAsset,
    ];
}

impl fmt::Display for CheckKind {
    /// Writes what agrees, such as `transactions that come to zero`.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.pad(match self {
            Self::TransactionBalance => "transactions that come to zero",
            Self::LedgerBalance => "ledgers that come to zero",
            Self::HeaderAndLines => "transactions whose lines agree with the header",
            Self::LineChain => "transactions whose lines are in one chain",
            Self::FormTotal => "forms whose lines add up to the total",
            Self::AccountBalance => "accounts with the balance QuickBooks keeps",
            Self::CustomerBalance => "customers with the balance QuickBooks keeps",
            Self::VendorBalance => "vendors with the balance QuickBooks keeps",
            Self::OpenForm => "forms that say what their links leave",
            Self::OpenReceivable => "receivables kept open as their links leave them",
            Self::OpenPayable => "payables kept open as their links leave them",
            Self::ReceivableAccount => "receivable accounts that open items come to",
            Self::PayableAccount => "payable accounts that open items come to",
            Self::ItemQuantity => "items with the quantity QuickBooks keeps",
            Self::ItemCount => "items whose lines add up to the quantity",
            Self::ItemValue => "items with the value QuickBooks keeps",
            Self::ItemPostings => "items whose postings come to the value",
            Self::SiteQuantity => "quantities at sites that QuickBooks keeps",
            Self::OnOrder => "quantities on order that QuickBooks keeps",
            Self::OnSalesOrders => "quantities on sales orders QuickBooks keeps",
            Self::ToBeBuilt => "quantities to be built that QuickBooks keeps",
            Self::RequiredForBuilds => "quantities for builds that QuickBooks keeps",
            Self::InventoryAsset => "asset accounts that their items come to",
            Self::YearlyFigure => "yearly figures that QuickBooks worked out",
        })
    }
}

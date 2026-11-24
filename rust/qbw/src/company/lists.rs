//! The small lists of a company.

use sa17::Source;

use crate::company::Company;
use crate::company::pieces::Piece;
use crate::error::Result;
use crate::list::List;
use crate::lists;
use crate::lists::{
    Category, Class, Currency, CustomerMessage, CustomerType, JobType, ListKind, Memorized,
    PaymentMethod, PriceLevel, SalesRep, SalesTaxCode, ShipMethod, Site, SmallList, Template,
    Terms, VendorType,
};

/// The small lists that a company has read so far.
#[derive(Debug, Default)]
pub(super) struct Lists {
    categories: Piece<List<Category>>,
    classes: Piece<List<Class>>,
    currencies: Piece<List<Currency>>,
    customer_messages: Piece<List<CustomerMessage>>,
    customer_types: Piece<List<CustomerType>>,
    job_types: Piece<List<JobType>>,
    memorized: Piece<List<Memorized>>,
    payment_methods: Piece<List<PaymentMethod>>,
    price_levels: Piece<List<PriceLevel>>,
    sales_reps: Piece<List<SalesRep>>,
    sales_tax_codes: Piece<List<SalesTaxCode>>,
    ship_methods: Piece<List<ShipMethod>>,
    sites: Piece<List<Site>>,
    templates: Piece<List<Template>>,
    terms: Piece<List<Terms>>,
    vendor_types: Piece<List<VendorType>>,
}

/// # Small lists
impl<S: Source> Company<S> {
    /// One of the small lists, as a tree, with what the entries of every list
    /// have.
    ///
    /// # Errors
    ///
    /// Returns an error when the table of the list lacks a column.
    pub fn small_list(&self, kind: ListKind) -> Result<SmallList<'_>> {
        Ok(match kind {
            ListKind::Categories => SmallList::new(kind, self.categories()?),
            ListKind::Classes => SmallList::new(kind, self.classes()?),
            ListKind::Currencies => SmallList::new(kind, self.currencies()?),
            ListKind::CustomerMessages => SmallList::new(kind, self.customer_messages()?),
            ListKind::CustomerTypes => SmallList::new(kind, self.customer_types()?),
            ListKind::JobTypes => SmallList::new(kind, self.job_types()?),
            ListKind::Memorized => SmallList::new(kind, self.memorized()?),
            ListKind::PaymentMethods => SmallList::new(kind, self.payment_methods()?),
            ListKind::PriceLevels => SmallList::new(kind, self.price_levels()?),
            ListKind::SalesReps => SmallList::new(kind, self.sales_reps()?),
            ListKind::SalesTaxCodes => SmallList::new(kind, self.sales_tax_codes()?),
            ListKind::ShipMethods => SmallList::new(kind, self.ship_methods()?),
            ListKind::Sites => SmallList::new(kind, self.sites()?),
            ListKind::Templates => SmallList::new(kind, self.templates()?),
            ListKind::Terms => SmallList::new(kind, self.terms()?),
            ListKind::VendorTypes => SmallList::new(kind, self.vendor_types()?),
        })
    }

    /// The categories that items are sorted into.
    ///
    /// # Errors
    ///
    /// Returns an error when the table of them lacks a column.
    pub fn categories(&self) -> Result<&List<Category>> {
        let lists = &self.pieces().lists;
        lists
            .categories
            .get_or_build(|| lists::categories(self.source()))
    }

    /// The classes. Unchecked.
    ///
    /// # Errors
    ///
    /// Returns an error when the table of them lacks a column.
    pub fn classes(&self) -> Result<&List<Class>> {
        let lists = &self.pieces().lists;
        lists.classes.get_or_build(|| lists::classes(self.source()))
    }

    /// The currencies, with what they were worth. Those that the company does
    /// not use are inactive. The currency of a company that uses one,
    /// [`CurrencyId::COMPANY`](crate::CurrencyId::COMPANY), is not among them.
    ///
    /// # Errors
    ///
    /// Returns an error when a table of them lacks a column.
    pub fn currencies(&self) -> Result<&List<Currency>> {
        let lists = &self.pieces().lists;
        lists
            .currencies
            .get_or_build(|| lists::currencies(self.source()))
    }

    /// The messages that print on the forms that customers get.
    ///
    /// # Errors
    ///
    /// Returns an error when the table of them lacks a column.
    pub fn customer_messages(&self) -> Result<&List<CustomerMessage>> {
        let lists = &self.pieces().lists;
        lists
            .customer_messages
            .get_or_build(|| lists::customer_messages(self.source()))
    }

    /// The types of customers.
    ///
    /// # Errors
    ///
    /// Returns an error when the table of them lacks a column.
    pub fn customer_types(&self) -> Result<&List<CustomerType>> {
        let lists = &self.pieces().lists;
        lists
            .customer_types
            .get_or_build(|| lists::customer_types(self.source()))
    }

    /// The types of jobs.
    ///
    /// # Errors
    ///
    /// Returns an error when the table of them lacks a column.
    pub fn job_types(&self) -> Result<&List<JobType>> {
        let lists = &self.pieces().lists;
        lists
            .job_types
            .get_or_build(|| lists::job_types(self.source()))
    }

    /// The transactions that are kept to be made again.
    ///
    /// # Errors
    ///
    /// Returns an error when the table of them lacks a column.
    pub fn memorized(&self) -> Result<&List<Memorized>> {
        let lists = &self.pieces().lists;
        lists
            .memorized
            .get_or_build(|| lists::memorized(self.source()))
    }

    /// The ways that customers pay.
    ///
    /// # Errors
    ///
    /// Returns an error when the table of them lacks a column.
    pub fn payment_methods(&self) -> Result<&List<PaymentMethod>> {
        let lists = &self.pieces().lists;
        lists
            .payment_methods
            .get_or_build(|| lists::payment_methods(self.source()))
    }

    /// The price levels, with the prices of those that have their own.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::preferences`], and an error when a table
    /// of them lacks a column.
    pub fn price_levels(&self) -> Result<&List<PriceLevel>> {
        let lists = &self.pieces().lists;
        lists.price_levels.get_or_build(|| {
            let home = self.preferences()?.home_currency;
            lists::price_levels(self.source(), home)
        })
    }

    /// The sales reps.
    ///
    /// # Errors
    ///
    /// Returns an error when the table of them lacks a column.
    pub fn sales_reps(&self) -> Result<&List<SalesRep>> {
        let lists = &self.pieces().lists;
        lists
            .sales_reps
            .get_or_build(|| lists::sales_reps(self.source()))
    }

    /// The sales tax codes.
    ///
    /// # Errors
    ///
    /// Returns an error when the table of them lacks a column.
    pub fn sales_tax_codes(&self) -> Result<&List<SalesTaxCode>> {
        let lists = &self.pieces().lists;
        lists
            .sales_tax_codes
            .get_or_build(|| lists::sales_tax_codes(self.source()))
    }

    /// The ways that goods are shipped.
    ///
    /// # Errors
    ///
    /// Returns an error when the table of them lacks a column.
    pub fn ship_methods(&self) -> Result<&List<ShipMethod>> {
        let lists = &self.pieces().lists;
        lists
            .ship_methods
            .get_or_build(|| lists::ship_methods(self.source()))
    }

    /// The sites that stock is kept at, and their bins. A company that keeps it
    /// at one has none.
    ///
    /// # Errors
    ///
    /// Returns an error when the table of them lacks a column.
    pub fn sites(&self) -> Result<&List<Site>> {
        let lists = &self.pieces().lists;
        lists.sites.get_or_build(|| lists::sites(self.source()))
    }

    /// The templates that forms are printed with.
    ///
    /// # Errors
    ///
    /// Returns an error when the table of them lacks a column.
    pub fn templates(&self) -> Result<&List<Template>> {
        let lists = &self.pieces().lists;
        lists
            .templates
            .get_or_build(|| lists::templates(self.source()))
    }

    /// The payment terms.
    ///
    /// # Errors
    ///
    /// Returns an error when the table of them lacks a column.
    pub fn terms(&self) -> Result<&List<Terms>> {
        let lists = &self.pieces().lists;
        lists.terms.get_or_build(|| lists::terms(self.source()))
    }

    /// The types of vendors.
    ///
    /// # Errors
    ///
    /// Returns an error when the table of them lacks a column.
    pub fn vendor_types(&self) -> Result<&List<VendorType>> {
        let lists = &self.pieces().lists;
        lists
            .vendor_types
            .get_or_build(|| lists::vendor_types(self.source()))
    }
}

//! The forms that transactions are kept on.

use sa17::Source;

use crate::codes::TransactionKind;
use crate::error::Result;
use crate::tables::postings::InventoryLineRow;
use crate::tables::transactions::{ALL, KindSource, TransactionTables};
use crate::tables::{OpenTable, OpenedTables, StoredRow, Table};

/// The number of forms.
pub(crate) const FORMS: usize = ALL.len();

/// A form of transaction, which has tables of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Form {
    /// The form's place in [`ALL`].
    place: usize,
}

impl Form {
    /// Every form.
    pub(crate) fn all() -> impl Iterator<Item = Self> {
        (0..FORMS).map(|place| Self { place })
    }

    /// The first form that a transaction of `kind` can be on.
    pub(crate) fn of_kind(kind: TransactionKind) -> Option<Self> {
        Self::all().find(|form| form.can_hold(kind))
    }

    /// The form's place among all of them.
    pub(crate) const fn place(self) -> usize {
        self.place
    }

    /// The table of the headers, read as `R`.
    pub(crate) const fn headers<R: StoredRow>(self) -> Table<R> {
        Table::new(self.tables().headers)
    }

    /// The table of the lines, read as `R`.
    pub(crate) const fn lines<R: StoredRow>(self) -> Table<R> {
        Table::new(self.tables().lines)
    }

    /// The table of what the lines of items in stock post besides, for a form
    /// that can move stock.
    pub(crate) fn stock_lines(self) -> Option<Table<InventoryLineRow>> {
        self.tables().stock_lines.map(Table::new)
    }

    /// The headers in `source`, read as `R`. `None` for a form of payroll that
    /// the source has no tables for.
    ///
    /// # Errors
    ///
    /// Returns an error when the source lacks the table of another form, or a
    /// column that `R` requires.
    pub(crate) fn headers_in<R: StoredRow>(
        self,
        source: &dyn Source,
    ) -> Result<Option<OpenTable<'_, R>>> {
        self.found(&self.headers(), source)
    }

    /// The lines in `source`, read as `R`. `None` where [`Self::headers_in`] is.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::headers_in`].
    pub(crate) fn lines_in<R: StoredRow>(
        self,
        source: &dyn Source,
    ) -> Result<Option<OpenTable<'_, R>>> {
        self.found(&self.lines(), source)
    }

    /// The lines as [`Self::lines_in`] gives them, through tables that were opened
    /// before.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::lines_in`].
    pub(crate) fn lines_among<'source, R: StoredRow + 'static>(
        self,
        tables: &OpenedTables,
        source: &'source dyn Source,
    ) -> Result<Option<OpenTable<'source, R>>> {
        let lines = self.lines();
        if self.tables().is_of_payroll {
            tables.find(&lines, source)
        } else {
            tables.open(&lines, source).map(Some)
        }
    }

    /// What the lines of items in stock post besides, in `source`. `None` for a
    /// form that moves no stock.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::headers_in`].
    pub(crate) fn stock_lines_in(
        self,
        source: &dyn Source,
    ) -> Result<Option<OpenTable<'_, InventoryLineRow>>> {
        match self.stock_lines() {
            Some(table) => self.found(&table, source),
            None => Ok(None),
        }
    }

    fn found<'source, R: StoredRow>(
        self,
        table: &Table<R>,
        source: &'source dyn Source,
    ) -> Result<Option<OpenTable<'source, R>>> {
        if self.tables().is_of_payroll {
            table.find(source)
        } else {
            table.open(source).map(Some)
        }
    }

    /// The kind of a transaction whose header or lines have `code`.
    pub(crate) fn kind(self, code: Option<TransactionKind>) -> TransactionKind {
        match self.tables().kind {
            KindSource::Table(kind) => kind,
            KindSource::Header(_) => code.unwrap_or(TransactionKind::Other(0)),
        }
    }

    /// Whether a transaction of `kind` can be on the form. One of a kind that
    /// no file had yet can be on every form that has several.
    pub(crate) fn can_hold(self, kind: TransactionKind) -> bool {
        match self.tables().kind {
            KindSource::Table(own) => own == kind,
            KindSource::Header(kinds) => {
                kinds.contains(&kind) || !ALL.iter().any(|tables| tables.is_of(kind))
            }
        }
    }

    /// Whether an order or a transfer between accounts can be on the form.
    pub(crate) fn has_orders_or_transfers(self) -> bool {
        let kinds = [
            TransactionKind::PurchaseOrder,
            TransactionKind::SalesOrder,
            TransactionKind::Transfer,
        ];
        kinds.into_iter().any(|kind| self.can_hold(kind))
    }

    const fn tables(self) -> &'static TransactionTables {
        &ALL[self.place]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form(headers: &str) -> Form {
        let mut forms = Form::all();
        forms
            .find(|form| form.tables().headers == headers)
            .expect("there is such a form")
    }

    #[test]
    fn a_table_or_a_header_decides_the_kind() {
        let code = Some(TransactionKind::SalesTaxPayment);
        let checks = form("abmc_check_header");
        assert_eq!(checks.kind(code), TransactionKind::SalesTaxPayment);
        assert_eq!(checks.kind(None), TransactionKind::Other(0));
        // The code that is there on card payments of bills is not to be read.
        let card_payments = form("abmc_bill_ccard_header");
        assert_eq!(
            card_payments.kind(code),
            TransactionKind::BillPaymentCreditCard
        );
    }

    #[test]
    fn a_kind_is_in_its_own_tables_or_in_those_of_several() {
        let holding = |kind| -> usize { Form::all().filter(|form| form.can_hold(kind)).count() };
        assert_eq!(holding(TransactionKind::Invoice), 1);
        assert!(form("abmc_invoice_header").can_hold(TransactionKind::Invoice));
        assert_eq!(holding(TransactionKind::ItemReceipt), 1);
        assert!(form("abmc_bill_header").can_hold(TransactionKind::ItemReceipt));
        // Bills, checks, card charges and the three forms of payroll.
        assert_eq!(holding(TransactionKind::Carryover), 6);
        assert!(!form("abmc_invoice_header").can_hold(TransactionKind::Carryover));
    }
}

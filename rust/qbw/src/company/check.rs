//! Checking a company's books.

use chrono::NaiveDate;
use sa17::Source;

use crate::as_of::AsOf;
use crate::check;
use crate::check::{Check, CheckKind, FormCheck, Open, StockCheck, Tally};
use crate::company::Company;
use crate::error::{Error, Result};
use crate::left_out::LeftOut;
use crate::side::Side;
use crate::tables::Form;

/// # How far the figures can be trusted
impl<S: Source> Company<S> {
    /// Compares what is worked out from the books with what QuickBooks keeps, and
    /// with the rest of the books. What differs is a finding and no error, and
    /// what the file lacks a table for is in [`Check::not_compared`].
    ///
    /// # Errors
    ///
    /// Returns an error when the file lacks a table or a column of the accounts
    /// or the transactions, or when the company's currency cannot be understood.
    pub fn check(&self) -> Result<Check> {
        let (accounts, ledger) = (self.accounts()?, self.ledger()?);
        let mut tally = Tally::default();
        tally.lose(accounts.damage());
        tally.lose(ledger.damage());
        let unread_tables = ledger.left_out().unread_tables;
        tally.leave_out(LeftOut::of_unread_tables(unread_tables));
        check::ledger(&mut tally, ledger);
        let open = compared(&mut tally, CheckKind::LINKS, self.open_sides())?;
        self.check_forms(&mut tally, &FormCheck::new(ledger, open.as_ref()))?;
        check::accounts(&mut tally, ledger, accounts);
        if let Some(names) = compared(&mut tally, CheckKind::NAMES, self.names())? {
            tally.lose(names.damage());
            check::names(&mut tally, ledger, names);
        }
        for open in open.iter().flatten() {
            open.check(&mut tally, ledger, accounts);
        }
        if open.is_some() {
            let links = self.link_index()?;
            tally.lose(links.damage());
            tally.leave_out(links.unknown());
        }
        if let Some(stock) = compared(&mut tally, CheckKind::STOCK, self.stock())? {
            let stock_tables = [self.items()?.damage(), self.sites()?.damage()];
            tally.lose(stock_tables.into_iter().flatten());
            tally.lose(self.stock_history()?.damage());
            tally.lose(self.item_lines()?.damage());
            let stock_check = StockCheck {
                stock: &stock,
                items: self.items()?,
                sites: self.sites()?,
                ledger,
            };
            stock_check.check(&mut tally);
        }
        tally.lose(self.facts()?.damage());
        check::facts(&mut tally, self.facts()?, self.books()?);
        Ok(tally.into_check())
    }

    /// What is open on both sides, by two ways of working it out.
    fn open_sides(&self) -> Result<[Open; 2]> {
        let open = |side| -> Result<Open> {
            Ok(Open {
                kept: self.open_items(side, AsOf::Latest)?,
                linked: self.open_items(side, NaiveDate::MAX)?,
            })
        };
        Ok([open(Side::Receivable)?, open(Side::Payable)?])
    }

    /// Checks every transaction, one after the other.
    fn check_forms(&self, tally: &mut Tally, check: &FormCheck<'_>) -> Result<()> {
        let headers = self.header_index()?;
        for header in headers.iter() {
            match self.transaction(header.id) {
                Ok(Some(form)) => {
                    tally.lose(&form.damage);
                    check.check(tally, &form);
                }
                Ok(None) => {}
                Err(Error::Damaged(damage)) => tally.lose([&damage]),
                Err(error) => return Err(error),
            }
        }
        for form in Form::all() {
            tally.lose(headers.damage(form));
        }
        Ok(())
    }
}

/// A part of the check, unless the file lacks a table for it. Then `kinds` are
/// not compared.
fn compared<T>(tally: &mut Tally, kinds: &[CheckKind], part: Result<T>) -> Result<Option<T>> {
    match part {
        Ok(part) => Ok(Some(part)),
        Err(Error::NoTable { table }) => {
            tally.skip(kinds, table);
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

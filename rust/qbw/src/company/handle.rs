//! The handle that everything is asked of.

use std::fmt;
use std::mem;
use std::path::Path;

use sa17::{Columns, Database, OpenOptions, Row, Source};

use crate::company::pieces::Pieces;
use crate::error::{Error, Result};
use crate::origin::Origin;
use crate::tables::accounts::ACCOUNTS;
use crate::tables::settings::PREFERENCES;
use crate::tables::table;

/// A company file, or a source that answers in its place.
///
/// Everything is asked of a company. It reads what an answer needs when it is
/// first asked for, and keeps it.
///
/// ```
/// use qbw::Company;
///
/// # let file = qbw_synthetic::company::books().build();
/// let company = Company::from_bytes(file)?;
/// for account in company.accounts()?.active() {
///     println!("{} ({})", account.name, account.account_type);
/// }
/// # Ok::<(), qbw::Error>(())
/// ```
pub struct Company<S: Source = Database> {
    source: S,
    pieces: Pieces,
}

/// # Opening a file
impl Company {
    /// Opens the company file at `path`. The whole file is read into memory.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Database`] when the file cannot be read or is not a
    /// database, and the errors of [`Self::try_new`].
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::try_new(Database::open(path)?)
    }

    /// Opens the company file at `path` with explicit [`OpenOptions`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Database`] when the file cannot be read or is not a
    /// database, and the errors of [`Self::try_new`].
    pub fn open_with(path: impl AsRef<Path>, options: &OpenOptions) -> Result<Self> {
        Self::try_new(Database::open_with(path, options)?)
    }

    /// Opens a company file from its bytes.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::open`].
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        Self::try_new(Database::from_bytes(bytes)?)
    }

    /// Opens a company file from its bytes with explicit [`OpenOptions`].
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::open_with`].
    pub fn from_bytes_with(bytes: Vec<u8>, options: &OpenOptions) -> Result<Self> {
        Self::try_new(Database::from_bytes_with(bytes, options)?)
    }
}

/// # The file below
impl<S: Source> Company<S> {
    /// The company whose tables `source` answers for.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotACompany`] when the source lacks the table of the
    /// accounts or of the preferences.
    pub fn try_new(source: S) -> Result<Self> {
        for name in [ACCOUNTS.name(), PREFERENCES.name()] {
            if table::definition(&source, name)?.is_none() {
                return Err(Error::NotACompany { table: name });
            }
        }
        Ok(Self {
            source,
            pieces: Pieces::default(),
        })
    }

    /// The database below, with every table the file has.
    #[must_use]
    pub fn source(&self) -> &S {
        &self.source
    }

    /// Changes the source, and forgets everything that was worked out from it.
    ///
    /// What is worked out from the postings follows the change, such as balances,
    /// statements and open items of a day. What QuickBooks keeps as a figure does
    /// not, and is read as it is: `stored` and everything else that is called so,
    /// what is open now, quantities on hand, values and average costs.
    /// [`Self::check`] shows where the two differ.
    pub fn edit<T>(&mut self, change: impl FnOnce(&mut S) -> T) -> T {
        self.pieces = Pieces::default();
        change(&mut self.source)
    }

    /// Changes the source as [`Self::edit`] does, unless it cannot be changed:
    /// nothing is forgotten then.
    ///
    /// ```
    /// use qbw::Company;
    /// # use std::convert::Infallible;
    ///
    /// # let file = qbw_synthetic::company::books().build();
    /// # let mut company = Company::from_bytes(file)?;
    /// let refused = company.try_edit(|_| Err::<Infallible, _>("the row is locked"));
    /// assert_eq!(refused, Err("the row is locked"));
    /// # Ok::<(), qbw::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns the error of `change`, which must mean that the source is as it
    /// was.
    pub fn try_edit<T, E>(
        &mut self,
        change: impl FnOnce(&mut S) -> std::result::Result<T, E>,
    ) -> std::result::Result<T, E> {
        // Nothing is left when `change` panics half-way.
        let pieces = mem::take(&mut self.pieces);
        let changed = change(&mut self.source);
        if changed.is_err() {
            self.pieces = pieces;
        }
        changed
    }

    /// Changes the source as [`Self::edit`] does, and forgets nothing when the
    /// change changed nothing: when `revision` gives of the source what it gave
    /// before.
    ///
    /// ```
    /// use qbw::Company;
    /// use qbw::sa17::{Database, Source};
    ///
    /// # let file = qbw_synthetic::company::books().build();
    /// # let mut company = Company::from_bytes(file)?;
    /// # fn revision(source: &Database) -> usize { source.definitions().len() }
    /// let accounts = company.accounts()?.len();
    /// let refused: Result<(), &str> = company.edit_by_revision(revision, |_| Err("locked"));
    /// assert_eq!((refused, company.accounts()?.len()), (Err("locked"), accounts));
    /// # Ok::<(), qbw::Error>(())
    /// ```
    pub fn edit_by_revision<T, R: PartialEq>(
        &mut self,
        revision: impl Fn(&S) -> R,
        change: impl FnOnce(&mut S) -> T,
    ) -> T {
        let before = revision(&self.source);
        // Nothing is left when `change` panics half-way.
        let pieces = mem::take(&mut self.pieces);
        let changed = change(&mut self.source);
        if revision(&self.source) == before {
            self.pieces = pieces;
        }
        changed
    }

    /// Gives up the company for its source.
    #[must_use]
    pub fn into_source(self) -> S {
        self.source
    }

    /// The whole stored row that a record came from, with the columns that the
    /// record leaves out. `None` when the source has no such row, or no longer.
    ///
    /// `origin` must be of a record of this company: in another one it names
    /// another row, or none.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Database`] when the row cannot be read.
    pub fn stored_row(&self, origin: Origin) -> Result<Option<Row>> {
        let Some(table) = table::definition(&self.source, origin.table)? else {
            return Ok(None);
        };
        let columns = Columns::all(table);
        Ok(self.source.fetch(table, &columns, origin.row)?)
    }

    pub(super) fn pieces(&self) -> &Pieces {
        &self.pieces
    }
}

impl<S: Source> fmt::Debug for Company<S> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Company")
            .field("tables", &self.source.definitions().len())
            .finish_non_exhaustive()
    }
}

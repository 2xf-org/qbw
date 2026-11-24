//! How much an answer lacks.

use std::collections::HashSet;
use std::fmt;

use crate::damage::{Damage, Place, Problem};
use crate::left_out::places::Places;

mod places;

/// How much an answer had to leave out.
///
/// Every list, balance, report and overview has one, with the rows of everything
/// that decides it. A damaged row may have belonged to any account or name, so
/// where anything is left out, every figure worked out from the same tables may
/// be short.
///
/// Two answers that rest on the same tables lack the same rows. [`Self::union`]
/// is what they lack together, and [`Self::with`] adds up what two answers lack
/// that rest on tables of their own. Two are equal when their counts are.
#[derive(Clone, Copy, Default)]
#[non_exhaustive]
pub struct LeftOut {
    /// Rows that could not be read.
    pub rows: u32,
    /// Pages that could not be read, with every row they hold.
    pub pages: u32,
    /// Tables with rows that this version does not know how to read.
    pub unread_tables: u32,
    /// Where some of the rows and pages are.
    places: Places,
}

impl LeftOut {
    /// What `damage` leaves out.
    pub(crate) fn of(damage: &[Damage]) -> Self {
        let mut left_out = Self::default();
        for damage in damage {
            let count = match damage.problem {
                Problem::Page { .. } => &mut left_out.pages,
                _ => &mut left_out.rows,
            };
            *count = count.saturating_add(1);
            left_out.places = left_out.places.with(Places::of(damage.place()));
        }
        left_out
    }

    /// What cannot be read at `places`, each of which counts once.
    pub(crate) fn of_places(places: impl IntoIterator<Item = Place>) -> Self {
        let places: HashSet<Place> = places.into_iter().collect();
        let places = places.into_iter().map(|place| {
            let count = match place.page {
                Some(_) => Self::of_pages(1),
                None => Self::of_rows(1),
            };
            Self {
                places: Places::of(place),
                ..count
            }
        });
        places.fold(Self::default(), Self::with)
    }

    /// So many rows.
    pub(crate) const fn of_rows(rows: u32) -> Self {
        Self {
            rows,
            ..Self::of_nothing()
        }
    }

    /// So many pages.
    const fn of_pages(pages: u32) -> Self {
        Self {
            pages,
            ..Self::of_nothing()
        }
    }

    /// So many tables that this version does not read.
    pub(crate) const fn of_unread_tables(unread_tables: u32) -> Self {
        Self {
            unread_tables,
            ..Self::of_nothing()
        }
    }

    const fn of_nothing() -> Self {
        Self {
            rows: 0,
            pages: 0,
            unread_tables: 0,
            places: Places::NONE,
        }
    }

    /// Whether the answer is complete.
    #[must_use]
    pub const fn is_nothing(self) -> bool {
        self.rows == 0 && self.pages == 0 && self.unread_tables == 0
    }

    /// What two answers leave out together that rest on tables of their own: the
    /// counts are added up.
    #[must_use]
    pub const fn with(self, other: Self) -> Self {
        Self {
            rows: self.rows.saturating_add(other.rows),
            pages: self.pages.saturating_add(other.pages),
            unread_tables: self.unread_tables.saturating_add(other.unread_tables),
            places: self.places.with(other.places),
        }
    }

    /// What two answers leave out together that rest on the same tables, at least
    /// in part: what both lack counts once.
    ///
    /// ```
    /// use qbw::{AsOf, Company, Side};
    ///
    /// # let file = qbw_synthetic::company::damaged_books().build();
    /// let company = Company::from_bytes(file)?;
    /// let sides = [Side::Receivable, Side::Payable];
    /// let [owed, owing] = sides.map(|side| company.open_items(side, AsOf::Latest));
    /// let (owed, owing) = (owed?.left_out, owing?.left_out);
    /// assert_eq!((owed.rows, owing.rows), (1, 1));
    /// assert_eq!(owed.union(owing).rows, 1);
    /// # Ok::<(), qbw::Error>(())
    /// ```
    ///
    /// A `LeftOut` knows where the first few rows and pages are that it counts.
    /// Of two that count more than that, the larger number counts for the rest.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        let both = self.places.with(other.places);
        let known = self.places.together(other.places);
        let unplaced_rows = larger(
            self.rows.saturating_sub(self.places.rows()),
            other.rows.saturating_sub(other.places.rows()),
        );
        let unplaced_pages = larger(
            self.pages.saturating_sub(self.places.pages()),
            other.pages.saturating_sub(other.places.pages()),
        );
        Self {
            rows: known.rows.saturating_add(unplaced_rows),
            pages: known.pages.saturating_add(unplaced_pages),
            unread_tables: larger(self.unread_tables, other.unread_tables),
            places: both,
        }
    }
}

impl PartialEq for LeftOut {
    fn eq(&self, other: &Self) -> bool {
        let counts = |left_out: &Self| (left_out.rows, left_out.pages, left_out.unread_tables);
        counts(self) == counts(other)
    }
}

impl Eq for LeftOut {}

impl fmt::Debug for LeftOut {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LeftOut")
            .field("rows", &self.rows)
            .field("pages", &self.pages)
            .field("unread_tables", &self.unread_tables)
            .finish()
    }
}

const fn larger(left: u32, right: u32) -> u32 {
    if left < right { right } else { left }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use sa17::{RowAddress, RowId};

    use super::*;

    #[test]
    fn rows_and_pages_are_counted_apart() {
        let damage = |problem| Damage::new("lines", None, problem);
        let page = Problem::Page {
            page: 12,
            reason: String::new(),
        };
        let left_out = LeftOut::of(&[
            damage(page),
            damage(Problem::OwnLine),
            damage(Problem::LineChain),
        ]);
        assert_eq!((left_out.rows, left_out.pages), (2, 1));
        assert!(!left_out.is_nothing() && LeftOut::of(&[]).is_nothing());
        let twice = left_out.with(left_out);
        assert_eq!((twice.rows, twice.pages, twice.unread_tables), (4, 2, 0));
    }

    #[test]
    fn a_place_counts_once() {
        let row = |slot| Some(RowId::Stored(RowAddress::new(7, slot)));
        let page = |page| {
            let reason = String::new();
            Damage::new("lines", None, Problem::Page { page, reason })
        };
        let damage = [
            Damage::new("lines", row(1), Problem::OwnLine),
            Damage::new("lines", row(1), Problem::LineChain),
            Damage::new("lines", row(2), Problem::LineChain),
            Damage::new("links", row(1), Problem::LineChain),
            page(12),
            page(12),
            page(13),
        ];
        let left_out = LeftOut::of_places(damage.iter().map(Damage::place));
        assert_eq!((left_out.rows, left_out.pages), (3, 2));
    }

    /// What so many rows of a table leave out, from one slot on.
    fn rows(table: &'static str, slots: Range<u16>) -> LeftOut {
        let damage = slots.map(|slot| {
            let row = RowId::Stored(RowAddress::new(1, slot));
            Damage::new(table, Some(row), Problem::OwnLine)
        });
        LeftOut::of(&damage.collect::<Vec<_>>())
    }

    #[test]
    fn what_both_lack_counts_once_in_a_union() {
        let lines = rows("lines", 0..3);
        assert_eq!(lines.with(lines).rows, 6);
        assert_eq!(lines.union(lines).rows, 3);
        // Half of them, and some of another table.
        assert_eq!(lines.union(rows("lines", 2..5)).rows, 5);
        assert_eq!(lines.union(rows("links", 0..2)).rows, 5);
        // One answer rests on both already.
        let both = lines.with(rows("links", 0..2));
        assert_eq!(both.union(lines), both);
        assert_eq!(both.union(rows("links", 0..4)).rows, 7);
        assert_eq!(lines.union(LeftOut::default()), lines);
    }

    #[test]
    fn rows_without_a_place_are_the_larger_number_of_a_union() {
        let lines = rows("lines", 0..3);
        let (two, four) = (LeftOut::of_rows(2), LeftOut::of_rows(4));
        assert_eq!(two.union(four).rows, 4);
        assert_eq!(lines.with(two).union(lines.with(four)).rows, 7);
        let (one, three) = (LeftOut::of_unread_tables(1), LeftOut::of_unread_tables(3));
        assert_eq!(one.union(three).unread_tables, 3);
    }

    #[test]
    fn a_union_of_many_rows_is_at_least_the_larger_number() {
        let (many, more) = (rows("lines", 0..40), rows("lines", 0..90));
        assert_eq!(many.union(many).rows, 40);
        assert_eq!(more.union(more.with(LeftOut::default())).rows, 90);
        let both = many.union(more).rows;
        assert!((90..=90 + 8).contains(&both), "{both}");
        let other = many.union(rows("links", 0..40)).rows;
        assert!((40..=80).contains(&other), "{other}");
    }
}

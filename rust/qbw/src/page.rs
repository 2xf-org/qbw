//! Parts of a long answer.

use std::ops::Range;

/// Which rows of a long answer to give: at most so many, from one of them on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Page {
    start: usize,
    size: usize,
}

impl Page {
    /// At most `size` rows, after the first `start`.
    #[must_use]
    pub const fn new(start: usize, size: usize) -> Self {
        Self { start, size }
    }

    /// Every row.
    #[must_use]
    pub const fn all() -> Self {
        Self::new(0, usize::MAX)
    }

    /// How many rows come before the page.
    #[must_use]
    pub const fn start(self) -> usize {
        self.start
    }

    /// How many rows the page has at most.
    #[must_use]
    pub const fn size(self) -> usize {
        self.size
    }

    /// The places of the page among `rows` rows.
    pub(crate) fn of(self, rows: usize) -> Range<usize> {
        let start = self.start.min(rows);
        start..start.saturating_add(self.size).min(rows)
    }
}

impl Default for Page {
    /// Every row.
    fn default() -> Self {
        Self::all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_page_ends_with_the_rows() {
        assert_eq!(Page::new(0, 2).of(5), 0..2);
        assert_eq!(Page::new(4, 2).of(5), 4..5);
        assert_eq!(Page::new(6, 2).of(5), 5..5);
        assert_eq!(Page::new(1, usize::MAX).of(5), 1..5);
        assert_eq!(Page::default().of(5), 0..5);
    }
}

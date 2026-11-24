//! Where what is left out is.

use std::hash::{DefaultHasher, Hash, Hasher};

use crate::damage::Place;

/// How many places there is room for.
const ROOM: usize = 8;

/// The bit of a print that says whether it is of a page.
const PAGE: u32 = 1;

/// Some places that could not be read, each as a number that no other place is
/// likely to have. Of more than there is room for, they are the ones with the
/// lowest numbers, so that two sets of the same places have the same ones.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Places {
    /// The numbers in order. Those past the last place are zero, which no place
    /// has.
    prints: [u32; ROOM],
}

/// How many rows and pages are at some places.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Counts {
    pub(super) rows: u32,
    pub(super) pages: u32,
}

impl Places {
    pub(super) const NONE: Self = Self { prints: [0; ROOM] };

    pub(super) fn of(place: Place) -> Self {
        let mut hasher = DefaultHasher::new();
        place.hash(&mut hasher);
        // The low half of the hash, where the last bit makes room for the kind.
        let print = u32::try_from(hasher.finish() >> 32).unwrap_or(u32::MAX);
        let print = print & !PAGE | if place.page.is_some() { PAGE } else { 0 };
        let mut places = Self::NONE;
        // A place has some number, be it one that another has, too.
        places.prints[0] = if print == 0 { 2 } else { print };
        places
    }

    /// The places of both, each once, as far as there is room.
    pub(super) const fn with(self, other: Self) -> Self {
        let mut merged = Merge::new(self, other);
        let mut prints = [0; ROOM];
        let mut room = 0;
        while room < ROOM
            && let Some(print) = merged.next()
        {
            prints[room] = print;
            room += 1;
        }
        Self { prints }
    }

    /// How many rows and pages are at the places of both, each once.
    pub(super) const fn together(self, other: Self) -> Counts {
        let mut merged = Merge::new(self, other);
        let mut counts = Counts { rows: 0, pages: 0 };
        while let Some(print) = merged.next() {
            if print & PAGE == 0 {
                counts.rows += 1;
            } else {
                counts.pages += 1;
            }
        }
        counts
    }

    pub(super) const fn rows(self) -> u32 {
        self.together(Self::NONE).rows
    }

    pub(super) const fn pages(self) -> u32 {
        self.together(Self::NONE).pages
    }
}

/// Goes through the prints of two sets in order, each once.
struct Merge {
    left: Places,
    right: Places,
    left_place: usize,
    right_place: usize,
}

impl Merge {
    const fn new(left: Places, right: Places) -> Self {
        Self {
            left,
            right,
            left_place: 0,
            right_place: 0,
        }
    }

    const fn next(&mut self) -> Option<u32> {
        let left = print_at(&self.left, self.left_place);
        let right = print_at(&self.right, self.right_place);
        // Zero is past the last of a set.
        let print = if left == 0 || (right != 0 && right < left) {
            right
        } else {
            left
        };
        if print == 0 {
            return None;
        }
        if left == print {
            self.left_place += 1;
        }
        if right == print {
            self.right_place += 1;
        }
        Some(print)
    }
}

const fn print_at(places: &Places, place: usize) -> u32 {
    if place < ROOM {
        places.prints[place]
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use sa17::{RowAddress, RowId};

    use super::*;
    use crate::damage::{Damage, Problem};

    fn row(slot: u16) -> Places {
        let row = RowId::Stored(RowAddress::new(1, slot));
        let damage = Damage::new("lines", Some(row), Problem::OwnLine);
        Places::of(damage.place())
    }

    fn rows(slots: impl IntoIterator<Item = u16>) -> Places {
        slots.into_iter().map(row).fold(Places::NONE, Places::with)
    }

    #[test]
    fn a_place_is_there_once() {
        let places = rows([1, 2, 1, 3, 2]);
        assert_eq!((places.rows(), places.pages()), (3, 0));
        let page = Damage::new(
            "lines",
            None,
            Problem::Page {
                page: 12,
                reason: String::new(),
            },
        );
        let page = Places::of(page.place());
        let both = places.together(page.with(page));
        assert_eq!(both, Counts { rows: 3, pages: 1 });
    }

    #[test]
    fn two_sets_have_what_is_in_either() {
        let (first, second) = (rows([1, 2, 3, 4]), rows([3, 4, 5, 6]));
        let both = first.with(second);
        assert_eq!((both.rows(), first.together(second).rows), (6, 6));
        assert_eq!(both.prints, rows([6, 5, 4, 3, 2, 1]).prints);
        // In order, with the room that is left at the end.
        assert!(both.prints[..6].is_sorted() && both.prints[6..] == [0, 0]);
        assert_eq!(both.with(first).prints, both.prints);
    }

    #[test]
    fn more_places_than_there_is_room_for_are_the_same_ones_each_time() {
        let forward = rows(1..=40);
        let backward = rows((1..=40).rev());
        assert_eq!(forward.prints, backward.prints);
        let room = u32::try_from(ROOM).unwrap();
        assert_eq!(forward.rows(), room);
        assert_eq!(forward.together(backward).rows, room);
    }
}

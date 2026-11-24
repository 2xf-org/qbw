//! The order of a list.
//!
//! Each entry of a list keeps the number of the one after it, and the last
//! one the number of the first. The first keeps it with a minus sign. One
//! chain goes through the customers, the vendors, the employees and the other
//! names.

use std::collections::HashMap;

/// Puts `entries` in the order of the list, from the number of each and the
/// one it keeps. `number` gives an entry's own.
///
/// An entry that the chain does not reach comes after the others, by its number.
pub(crate) fn sort<T>(
    entries: &mut [T],
    links: impl IntoIterator<Item = (i32, i32)>,
    number: impl Fn(&T) -> i32,
) {
    let places = places(links);
    entries.sort_by_key(|entry| {
        let number = number(entry);
        (places.get(&number).copied().unwrap_or(usize::MAX), number)
    });
}

/// The place of every number that the chain reaches, from each entry's number
/// and the one it keeps.
fn places(links: impl IntoIterator<Item = (i32, i32)>) -> HashMap<i32, usize> {
    let next: HashMap<i32, i32> = links.into_iter().collect();
    let firsts = next.iter().filter(|(_, next)| next.is_negative());
    let mut firsts: Vec<i32> = firsts.map(|(&number, _)| number).collect();
    firsts.sort_unstable();

    let mut places = HashMap::with_capacity(next.len());
    for first in firsts {
        let mut at = first;
        while let Some(&after) = next.get(&at)
            && !places.contains_key(&at)
        {
            places.insert(at, places.len());
            at = after.saturating_abs();
        }
    }
    places
}

#[cfg(test)]
mod tests {
    use super::*;

    fn in_order(links: &[(i32, i32)]) -> Vec<i32> {
        let mut numbers: Vec<i32> = links.iter().map(|&(number, _)| number).collect();
        sort(&mut numbers, links.iter().copied(), |&number| number);
        numbers
    }

    #[test]
    fn the_first_entry_has_a_minus_sign() {
        assert_eq!(in_order(&[(1, 2), (2, 3), (3, -1)]), [3, 1, 2]);
        assert_eq!(in_order(&[(5, -5)]), [5]);
        assert!(in_order(&[]).is_empty());
    }

    #[test]
    fn what_the_chain_does_not_reach_comes_last() {
        // 4 is reached by nothing, and 2 leads to an entry that is gone.
        assert_eq!(in_order(&[(4, 1), (3, -1), (1, 2), (2, 9)]), [3, 1, 2, 4]);
        // A ring has no first entry.
        assert_eq!(in_order(&[(2, 1), (1, 2)]), [1, 2]);
        assert_eq!(in_order(&[(1, i32::MIN), (2, -1)]), [1, 2]);
    }
}

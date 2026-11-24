//! The order that a list keeps in its rows, read a second time.

use std::collections::HashMap;

use qbw::Company;

use crate::agreement;

/// The number of every row of a table, deleted rows among them, with the one
/// that it keeps for the row after it.
pub(crate) fn links(company: &Company, table: &str, key: &str) -> HashMap<i32, i32> {
    let rows = agreement::rows(company, table, &[key, "display_index_next"]);
    let links = rows.iter();
    links
        .map(|row| (row.read(0).unwrap(), row.read(1).unwrap()))
        .collect()
}

/// The numbers in order: each keeps the one after it, the first one with a
/// minus sign, and the last the first.
pub(crate) fn chain(links: &HashMap<i32, i32>) -> Vec<i32> {
    let firsts = links.iter().filter(|(_, next)| **next < 0);
    let firsts: Vec<i32> = firsts.map(|(&number, _)| number).collect();
    let Some(&first) = firsts.first() else {
        return Vec::new();
    };
    assert_eq!(firsts.len(), 1, "a list has one chain");
    let mut chain = vec![first];
    let mut at = links[&first].abs();
    while at != first {
        assert!(chain.len() < links.len(), "a chain ends where it starts");
        chain.push(at);
        at = links[&at];
    }
    chain
}

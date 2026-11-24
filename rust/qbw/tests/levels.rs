//! Verifies that internal modules obey a strict acyclic dependency order.

// Every function in this file runs inside a test, where a failed unwrap is a
// failed test.
#![allow(clippy::unwrap_used)]

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

/// The modules. Each uses only the ones before it.
const LEVELS: &[&str] = &[
    "case",
    "number",
    "ids",
    "unix_time",
    "money_error",
    "money",
    "totals",
    "period",
    "as_of",
    "page",
    "amount",
    "currencies",
    "origin",
    "damage",
    "left_out",
    "error",
    "code",
    "codes",
    "posting_state",
    "rest",
    "party",
    "tables",
    "list",
    "side",
    "other_side",
    "chain",
    "address",
    "price",
    "settings",
    "accounts",
    "names",
    "items",
    "lists",
    "custom_fields",
    "ledger",
    "transactions",
    "query",
    "links",
    "open_items",
    "aging",
    "inventory",
    "orders",
    "reports",
    "registers",
    "overviews",
    "check",
    "company",
];

fn sources(folder: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, found);
        } else {
            found.push(path);
        }
    }
}

/// The module at the root of the crate that a file belongs to.
fn module(source: &Path, path: &Path) -> String {
    let path = path.strip_prefix(source).unwrap();
    let first = path.iter().next().unwrap();
    Path::new(first)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
}

/// The modules that a line of code names by their path from the crate's root.
fn used(line: &str) -> impl Iterator<Item = &str> {
    let paths = line.split("crate::").skip(1);
    paths.filter_map(|path| {
        path.split(|part: char| !part.is_alphanumeric() && part != '_')
            .next()
    })
}

#[test]
fn each_module_uses_only_the_ones_before_it() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let places = LEVELS.iter().enumerate();
    let places: HashMap<&str, usize> = places.map(|(place, name)| (*name, place)).collect();
    let source = root.join("src");
    let mut files = Vec::new();
    sources(&source, &mut files);
    files.retain(|path| *path != source.join("lib.rs"));
    let mut wrong = Vec::new();
    let mut uses = 0;
    for path in &files {
        let module = module(&source, path);
        let place = places.get(module.as_str());
        let text = fs::read_to_string(path).unwrap();
        let code = text
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"));
        for name in code.flat_map(used).filter(|&name| name != module) {
            uses += 1;
            let is_before = places.get(name).is_some_and(|name| Some(name) < place);
            if !is_before {
                let file = path.strip_prefix(&source).unwrap().display();
                wrong.push(format!("{file} uses {name}"));
            }
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
    assert!(uses > 500, "only {uses} uses were found");
}

#[test]
fn every_module_has_one_level() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let library = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    let declared = library.lines().filter_map(|line| {
        let line = line.strip_prefix("pub ").unwrap_or(line);
        line.strip_prefix("mod ")?.strip_suffix(';')
    });
    let declared: BTreeSet<&str> = declared.collect();
    let levels: BTreeSet<&str> = LEVELS.iter().copied().collect();
    assert_eq!(levels, declared);
    assert_eq!(LEVELS.len(), levels.len());
}

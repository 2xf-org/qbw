//! The company files that the tests read.

use std::env;
use std::fs;

use qbw::Company;

/// Every company file of the folder.
pub(crate) fn samples() -> impl Iterator<Item = Company> {
    let folder = env::var_os("QBW_SAMPLES").expect("QBW_SAMPLES names a folder");
    let entries = fs::read_dir(folder).unwrap();
    let paths = entries.map(|entry| entry.unwrap().path());
    let files = paths.filter(|path| {
        path.extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("qbw"))
    });
    let mut companies = files.map(|file| Company::open(file).unwrap()).peekable();
    assert!(
        companies.peek().is_some(),
        "QBW_SAMPLES holds no company file"
    );
    companies
}

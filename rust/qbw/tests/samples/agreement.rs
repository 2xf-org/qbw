//! How often what is worked out agrees with what the file keeps.

use qbw::Company;
use qbw::sa17::{Columns, Row};

/// How often figures agree.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Agreement {
    looked_at: usize,
    agreed: usize,
}

/// What the three books that the crate was checked against keep wrong: by how
/// many figures there are of a kind, how many do not agree with the postings.
const KEPT_WRONG: &[(usize, usize)] = &[
    // The accounts of two books, and the customers of one.
    (295, 3),
    (343, 2),
    (1125, 1),
    // Transactions on accounts receivable that the file keeps as open against
    // its own links.
    (34_432, 5),
    // A deposit in another currency that is a cent off.
    (62_378, 1),
    // Payments and an invoice of which the file keeps that something is left,
    // or nothing, against its own links.
    (17_362, 1),
    (16_503, 4),
    // Sales orders that are done with a line that is not.
    (9_295, 1),
    (2_801, 1),
    // A line of a purchase order.
    (23_434, 1),
];

impl Agreement {
    pub(crate) fn add(&mut self, agrees: bool) {
        self.looked_at += 1;
        self.agreed += usize::from(agrees);
    }

    /// Prints the figures and checks that no more of them disagree than the
    /// book keeps wrong.
    pub(crate) fn check(self, figures: &str) {
        let Self { looked_at, agreed } = self;
        println!("{figures}: {agreed} of {looked_at}");
        let known = KEPT_WRONG.iter().find(|(known, _)| *known == looked_at);
        let wrong = known.map_or(0, |(_, wrong)| *wrong);
        assert!(
            looked_at - agreed <= wrong,
            "{figures} agree less often than they did"
        );
    }
}

/// The rows of a table, with some of its columns.
pub(crate) fn rows(company: &Company, table: &str, columns: &[&str]) -> Vec<Row> {
    let table = company.source().table(table).unwrap();
    let columns = Columns::new(table.definition(), columns).unwrap();
    table.scan(&columns).collect::<Result<_, _>>().unwrap()
}

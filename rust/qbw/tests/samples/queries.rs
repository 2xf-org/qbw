//! Pages of transactions against the whole.

use qbw::query::{Direction, PaidState, TransactionOrder, TransactionPage, TransactionQuery};
use qbw::transactions::TransactionKind;
use qbw::{Company, Page};

use crate::books;

const SIZE: usize = 50;

/// The invoices, the newest first.
fn invoices() -> TransactionQuery {
    TransactionQuery::new()
        .kinds([TransactionKind::Invoice])
        .order(TransactionOrder::Date, Direction::Descending)
}

fn page(company: &Company, query: &TransactionQuery, page: Page) -> TransactionPage {
    let page = company.transactions(query, page).unwrap();
    assert!(page.left_out.is_nothing());
    page
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn pages_are_parts_of_the_whole() {
    for company in books::books() {
        let queries = [
            TransactionQuery::new(),
            invoices(),
            invoices().paid(PaidState::Unpaid),
            invoices().text("valve"),
        ];
        for query in queries {
            let all = page(company, &query, Page::all());
            assert_eq!(all.rows.len(), all.row_count);
            let counts = all.kinds.iter().map(|kind| kind.count);
            assert_eq!(counts.sum::<usize>(), all.row_count);
            for start in [0, all.row_count / 2, all.row_count.saturating_sub(SIZE)] {
                let some = page(company, &query, Page::new(start, SIZE));
                let end = (start + SIZE).min(all.row_count);
                assert_eq!(some.rows, all.rows[start..end]);
                assert_eq!(
                    (some.row_count, some.kinds),
                    (all.row_count, all.kinds.clone())
                );
            }
        }
    }
}

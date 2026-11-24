//! What transactions have to do with each other.

use crate::company::books::literal::{Literal, Values, row};
use crate::company::books::table::Table;

pub(super) fn tables() -> Vec<Table> {
    vec![links()]
}

/// Every link is there twice, once from each of its ends. Amounts are the
/// negative of what was paid, credited or taken off.
fn links() -> Table {
    Table::new(
        "abmc_transaction_link",
        &["
            link_id integer key
            transaction_id integer
            transaction_2_id integer null
            target_1_id integer null
            target_2_id integer null
            link_type smallint null
            is_t1_source_bool bit
            is_secondary_bool bit
            is_information_only_bool bit
            amount_amt numeric(20,5) null
            amount_qnty numeric(20,5) null
        "],
    )
    // Check 501 pays the bill.
    .rows(paid("-200.00", 6), ends(1, (300, 302), (200, 202)))
    // Payment 88 pays the lines of invoice 1041, and takes a discount.
    .rows(paid("-74.00", 4), ends(3, (500, 502), (400, 402)))
    .rows(paid("-20.00", 4), ends(5, (500, 502), (400, 403)))
    .rows(paid("-6.00", 4), ends(7, (500, 502), (400, 405)))
    .rows(paid("-2.00", 5), ends(9, (500, 503), (400, 402)))
    // The deposit brings payment 88 to the bank.
    .rows(paid("-100.00", 9), ends(11, (500, 501), (600, 602)))
    // The credit memo credits invoice 1041. The first part is there once more,
    // from the invoice, which does not count.
    .rows(paid("-50.00", 3), ends(13, (700, 702), (400, 402)))
    .rows(paid("-4.00", 3), ends(15, (700, 703), (400, 405)))
    .rows(
        row! { "amount_amt" => "-50.00", "link_type" => 3, "is_secondary_bool" => true },
        ends(17, (400, 402), (700, 702)),
    )
    // Wire W-9 pays invoice 1042.
    .rows(paid("-100.00", 4), ends(19, (900, 902), (800, 802)))
    // Two of the five valves ordered were received.
    .rows(
        row! { "amount_amt" => Literal::Null, "amount_qnty" => "2.00", "link_type" => 15 },
        ends(21, (1500, 1502), (1400, 1402)),
    )
    // A discount without money on invoice 1041.
    .rows(paid("-1.00", 5), ends(23, (2400, 2401), (400, 403)))
    // Neither counts: a line of invoice 1041 linked to itself, and the payment
    // of the bill once more, for information only.
    .rows(paid("-40.00", 13), ends(25, (400, 403), (400, 403)))
    .rows(
        row! { "amount_amt" => "-200.00", "link_type" => 6, "is_information_only_bool" => true },
        ends(27, (300, 302), (200, 201)),
    )
}

/// What both rows of a link of `kind` have.
fn paid(amount: &'static str, kind: i32) -> Values {
    row! { "amount_amt" => amount, "link_type" => kind }
}

/// The two rows of a link between two lines, each given with its transaction.
/// The first is the one that pays, credits or was made from the other.
fn ends(first_id: i32, source: (i32, i32), target: (i32, i32)) -> [Values; 2] {
    [
        row! {
            "link_id" => first_id, "transaction_id" => source.0, "target_1_id" => source.1,
            "transaction_2_id" => target.0, "target_2_id" => target.1, "is_t1_source_bool" => true,
        },
        row! {
            "link_id" => first_id + 1, "transaction_id" => target.0, "target_1_id" => target.1,
            "transaction_2_id" => source.0, "target_2_id" => source.1,
        },
    ]
}

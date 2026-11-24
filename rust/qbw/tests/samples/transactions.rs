//! Every transaction, against what the file keeps of it.

use std::collections::{BTreeMap, HashMap, HashSet};

use qbw::query::{PostingFilter, TransactionQuery};
use qbw::sa17::{Decimal, Row};
use qbw::transactions::{Details, Line, LineRole, PostingState, Transaction, TransactionKind};
use qbw::{Company, Money, OtherSide, Page};

use crate::agreement::Agreement;
use crate::{agreement, support};

/// Every transaction of a company, one after the other, the memorized ones
/// among them.
fn transactions(company: &Company) -> impl Iterator<Item = Transaction> {
    let query = TransactionQuery::new().posting(PostingFilter::All);
    let page = company.transactions(&query, Page::all()).unwrap();
    assert!(page.left_out.is_nothing());
    assert_eq!(page.rows.len(), page.row_count);
    let ids = page.rows.into_iter().map(|row| row.id);
    ids.map(|id| company.transaction(id).unwrap().unwrap())
}

/// The value at a position of a row, as a whole number.
fn number(row: &Row, position: usize) -> i64 {
    row.value(position)
        .and_then(|value| value.as_i64())
        .unwrap_or_default()
}

/// The value at a position of a row, as a number that may be missing.
fn decimal(row: &Row, position: usize) -> Option<Decimal> {
    row.read(position).unwrap()
}

fn flag(row: &Row, position: usize) -> bool {
    row.read::<Option<bool>>(position)
        .unwrap()
        .unwrap_or_default()
}

/// What the tables of lines have of a transaction.
#[derive(Default)]
struct Stored {
    lines: usize,
    kinds: HashSet<i64>,
}

/// What the tables of lines have of each transaction.
fn stored_lines(company: &Company) -> HashMap<i64, Stored> {
    let tables = company.source().tables().filter(|table| {
        let has = |column| table.definition().position(column).is_some();
        // The audit trail keeps lines that were changed or deleted.
        has("is_source_bool")
            && has("transaction_view_type")
            && table.name() != "abmc_audit_lineitem"
    });
    let tables = tables.map(|table| table.name());
    let mut stored: HashMap<i64, Stored> = HashMap::new();
    for table in tables {
        for row in agreement::rows(company, table, &["transaction_id", "transaction_view_type"]) {
            let stored = stored.entry(number(&row, 0)).or_default();
            stored.lines += 1;
            stored.kinds.insert(number(&row, 1));
        }
    }
    stored
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn every_transaction_reads_whole_and_is_of_the_kind_of_its_lines() {
    for company in support::samples() {
        let mut stored = stored_lines(&company);
        let mut kinds = Agreement::default();
        let mut whole = Agreement::default();
        let mut rows: usize = 0;
        for read in transactions(&company) {
            let stored = stored.remove(&read.id.number().into()).unwrap();
            kinds.add(stored.kinds == HashSet::from([read.kind.code()]));
            // The own line of a payment without money is among the lines.
            let own = read.own_line.as_ref().map(|own| own.id);
            let others = read.lines.iter().filter(|line| Some(line.id) != own);
            let lines = others.count() + usize::from(own.is_some());
            whole.add(read.damage.is_empty() && own.is_some() && lines == stored.lines);
            rows += lines;
        }
        kinds.check("transactions of the kind of their lines");
        whole.check("transactions that read whole");
        println!("{rows} lines");
        assert!(stored.is_empty(), "every line is of a transaction");
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn every_line_has_a_role_that_hides_no_amount() {
    for company in support::samples() {
        let mut roles: BTreeMap<String, usize> = BTreeMap::new();
        let mut without_amount = Agreement::default();
        for line in transactions(&company).flat_map(|read| read.lines) {
            if matches!(line.role, LineRole::Text | LineRole::Empty) {
                without_amount.add(line.amount.is_none_or(Money::is_zero));
            }
            *roles.entry(format!("{:?}", line.role)).or_default() += 1;
        }
        without_amount.check("lines of words or of nothing that have no amount");
        println!("{roles:?}");
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn the_lines_of_every_form_add_up_to_its_total() {
    for company in support::samples() {
        let mut totals = Agreement::default();
        for read in transactions(&company) {
            let Some(total) = read.total else {
                continue;
            };
            let adding = read.lines.iter().filter(|line| line.role.adds_to_total());
            let amounts = adding.filter_map(|line| line.amount);
            let sum = amounts.fold(Money::zero(read.currency), |sum, amount| {
                sum.checked_add(amount).unwrap()
            });
            totals.add(sum == total);
        }
        totals.check("forms whose lines add up to their total");
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn own_lines_are_on_the_side_of_their_kind() {
    for company in support::samples() {
        let mut sides = Agreement::default();
        let mut without_money: usize = 0;
        for read in transactions(&company) {
            let Some(total) = read.total else {
                continue;
            };
            sides.add(!total.is_negative());
            if read.payment().is_some_and(|payment| !payment.moved_money) {
                let own = read.own_line.and_then(|own| own.amount).unwrap();
                assert!(total.is_zero() && own.is_negative());
                assert_eq!(read.lines[0].role, LineRole::DiscountTaken);
                without_money += 1;
            }
        }
        sides.check("totals that are not below zero");
        println!("{without_money} payments without money");
    }
}

/// The columns of links that say what is paid or filled.
const LINK: &[&str] = &[
    "transaction_id",
    "transaction_2_id",
    "target_1_id",
    "link_type",
    "is_secondary_bool",
    "is_information_only_bool",
    "amount_amt",
    "amount_qnty",
];

/// The links that count, as they are stored.
fn links(company: &Company) -> impl Iterator<Item = Row> {
    let links = agreement::rows(company, "abmc_transaction_link", LINK);
    links.into_iter().filter(|link| {
        let is_to_itself = number(link, 0) == number(link, 1);
        !is_to_itself && !flag(link, 4) && !flag(link, 5)
    })
}

/// The sums of one column of the links of `types`, by another.
fn linked(company: &Company, types: &[i64], by: usize, column: usize) -> HashMap<i64, Decimal> {
    let mut sums: HashMap<i64, Decimal> = HashMap::new();
    for link in links(company).filter(|link| types.contains(&number(link, 3))) {
        let sum = sums.entry(number(&link, by)).or_default();
        let part = decimal(&link, column).unwrap_or_default();
        *sum = sum.checked_add(part).unwrap();
    }
    sums
}

/// The types of links that take something off a transaction of `kind`, when
/// one of them can be open.
fn types_of_links(kind: TransactionKind) -> Option<&'static [i64]> {
    Some(match kind {
        TransactionKind::Invoice
        | TransactionKind::StatementCharge
        | TransactionKind::CreditMemo => &[2, 3, 4, 5, 10, 22],
        TransactionKind::Bill | TransactionKind::VendorCredit | TransactionKind::ItemReceipt => {
            &[6, 7, 8, 16]
        }
        // A discount that a payment takes is not paid with it.
        TransactionKind::Payment => &[3, 4, 10, 22],
        TransactionKind::BillPaymentCheck | TransactionKind::BillPaymentCreditCard => &[6, 7, 8],
        _ => return None,
    })
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn what_is_left_is_what_links_leave() {
    for company in support::samples() {
        let mut linked: HashMap<i64, HashMap<i64, Decimal>> = HashMap::new();
        for link in links(&company) {
            let types = linked.entry(number(&link, 0)).or_default();
            let sum = types.entry(number(&link, 3)).or_default();
            let part = decimal(&link, 6).unwrap_or_default();
            *sum = sum.checked_add(part).unwrap();
        }
        let mut left: BTreeMap<String, Agreement> = BTreeMap::new();
        for read in transactions(&company) {
            let (Some(types), Some(open)) = (types_of_links(read.kind), read.open()) else {
                continue;
            };
            let links = linked.get(&read.id.number().into());
            let taken = types.iter().filter_map(|of| links?.get(of).copied());
            // What a link takes off is below zero.
            let links_leave = taken.fold(read.total.unwrap().amount(), |left, taken| {
                left.checked_add(taken).unwrap()
            });
            let of_kind = left.entry(format!("{:?}", read.kind)).or_default();
            of_kind.add(links_leave == open.amount());
        }
        for (kind, left) in left {
            left.check(&format!("open amounts that links leave, {kind}"));
        }
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn what_is_paid_is_what_the_header_says() {
    for company in support::samples() {
        let mut paid = Agreement::default();
        for table in ["abmc_invoice_header", "abmc_bill_header"] {
            for row in agreement::rows(&company, table, &["transaction_id", "is_paid_bool"]) {
                let id = i32::try_from(number(&row, 0)).unwrap();
                let read = company.transaction(qbw::TransactionId::new(id));
                let read = read.unwrap().unwrap();
                if read.posting == PostingState::Posted {
                    paid.add(read.is_paid() == Some(flag(&row, 1)));
                }
            }
        }
        paid.check("invoices and bills that are paid as their headers say");
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn orders_are_filled_by_their_links() {
    for company in support::samples() {
        let filled = linked(&company, &[15, 20], 2, 7);
        let mut lines = Agreement::default();
        let mut invoiced = Agreement::default();
        let mut received = Agreement::default();
        for read in transactions(&company) {
            if read.kind == TransactionKind::PurchaseOrder {
                let header = company.stored_row(read.origin).unwrap().unwrap();
                let flag = header.get("is_received_bool");
                let flag = flag.and_then(|flag| flag.as_bool());
                received.add(flag == Some(read.is_fully_received()));
            }
            for line in &read.lines {
                let Some(order) = line.order.filter(|order| order.filled.is_some()) else {
                    continue;
                };
                let linked = filled.get(&line.id.number().into());
                let linked = linked.copied().unwrap_or_default().abs();
                lines.add(order.filled.map(Decimal::abs) == Some(linked));
            }
            if read.kind == TransactionKind::SalesOrder && !read.is_closed() {
                let items = read.item_lines();
                let orders: Vec<_> = items.filter_map(|line| line.order).collect();
                let all_done = orders.iter().all(|order| order.is_done);
                invoiced.add(read.is_fully_invoiced() == all_done);
            }
        }
        lines.check("lines of orders that links fill");
        invoiced.check("sales orders that are invoiced when their lines are");
        received.check("purchase orders that are received as their headers say");
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn a_row_of_a_page_agrees_with_its_transaction() {
    for company in support::samples() {
        let query = TransactionQuery::new().posting(PostingFilter::All);
        let page = company.transactions(&query, Page::all()).unwrap();
        let mut rows = Agreement::default();
        for row in page.rows {
            let read = company.transaction(row.id).unwrap().unwrap();
            let own = read.own_line.as_ref().unwrap();
            // A transfer is between two accounts.
            let is_transfer = row.kind == TransactionKind::Transfer;
            let across = is_transfer.then_some(own.other_side);
            let transfer = match read.details {
                Details::Transfer(transfer) => Some(transfer),
                _ => None,
            };
            rows.add(
                (row.order, row.other_account.map(OtherSide::Account)) == (own.order, across)
                    && (row.transfer(), row.open()) == (transfer, read.open())
                    && (row.kind, row.date, row.name) == (read.kind, read.date, read.name)
                    && (row.number, row.memo) == (read.number, read.memo)
                    && (row.total, row.home_total) == (read.total, read.home_total)
                    && (row.account, row.amount) == (own.account, own.amount)
                    && (row.open, row.due_date) == (read.open_state, read.due_date)
                    && (row.posting, row.origin) == (read.posting, read.origin),
            );
        }
        rows.check("rows that agree with their transactions");
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn the_money_of_a_transfer_leaves_the_account_it_is_of() {
    for company in support::samples() {
        let query = TransactionQuery::new().kinds([TransactionKind::Transfer]);
        let page = company.transactions(&query, Page::all()).unwrap();
        let mut transfers = Agreement::default();
        for row in page.rows {
            let amount = row.amount.unwrap();
            let leaves = amount.is_negative() && row.transfer().unwrap().from == row.account;
            transfers.add(leaves);
        }
        transfers.check("transfers whose own line is the credit");
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn the_first_and_the_last_day_are_those_of_the_transactions() {
    for company in support::samples() {
        let days = company.transaction_days().unwrap().unwrap();
        let page = company.transactions(&TransactionQuery::new(), Page::all());
        let dates = page.unwrap().rows.into_iter().map(|row| row.date);
        let dates: Vec<_> = dates.collect();
        let found = (dates.iter().min().copied(), dates.iter().max().copied());
        assert_eq!((days.first(), days.last()), found);
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn the_columns_of_a_journal_entry_come_to_the_same() {
    for company in support::samples() {
        let home = company.preferences().unwrap().home_currency;
        let (mut home_entries, mut own_entries) = (Agreement::default(), Agreement::default());
        for read in transactions(&company) {
            let Details::Journal(journal) = read.details else {
                let mut lines = read.lines.iter();
                let has_none = lines.all(|line| line.debit().is_none() && line.credit().is_none());
                assert!(has_none, "only a journal entry has columns");
                continue;
            };
            let currency = if journal.is_shown_in_home_currency {
                home
            } else {
                read.currency
            };
            let sum = |side: fn(&Line) -> Option<Money>| {
                let mut amounts = read.form_lines().filter_map(side);
                amounts.try_fold(Money::zero(currency), Money::checked_add)
            };
            let agrees = sum(Line::debit).unwrap() == sum(Line::credit).unwrap();
            if journal.is_shown_in_home_currency {
                home_entries.add(agrees);
            } else {
                own_entries.add(agrees);
            }
        }
        home_entries.check("journal entries shown in the company's currency");
        own_entries.check("journal entries shown in their own currency");
    }
}

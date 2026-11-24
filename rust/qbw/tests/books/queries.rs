//! Finding transactions.

use std::cmp::Reverse;

use qbw::chrono::NaiveDate;
use qbw::query::{Direction, PaidState, PostingFilter, TransactionOrder, TransactionQuery};
use qbw::sa17::{Decimal, Value};
use qbw::transactions::{Details, OpenState, PostingState, TransactionKind};
use qbw::{AccountId, AsOf, NameId, OtherSide, Page, Period, Side, TransactionId};

use crate::changed::{Changed, date, whole};
use crate::{counted, forms, support};

/// What `query` finds, all of it.
fn found(query: &TransactionQuery) -> Vec<i32> {
    let company = support::books();
    let page = company.transactions(query, Page::all()).unwrap();
    assert_eq!(page.rows.len(), page.row_count);
    assert!(page.left_out.is_nothing());
    page.rows.iter().map(|row| row.id.number()).collect()
}

fn query() -> TransactionQuery {
    TransactionQuery::new()
}

fn day(text: &str) -> NaiveDate {
    text.parse().unwrap()
}

/// All that QuickBooks lists: the memorized check 1900 is not.
const NEWEST_FIRST: &[i32] = &[
    2500, 2400, 2300, 2200, 2100, 2000, 1800, 1700, 1600, 1500, 1400, 1300, 1200, 1100, 1000, 900,
    800, 700, 600, 500, 400, 300, 200, 100,
];

#[test]
fn without_a_question_everything_that_is_listed_is_found() {
    assert_eq!(found(&query()), NEWEST_FIRST);
    assert_eq!(TransactionQuery::default(), query());
}

#[test]
fn transactions_are_found_by_what_their_headers_say() {
    let invoices = query().kinds([TransactionKind::Invoice]);
    assert_eq!(found(&invoices), [1800, 800, 400]);
    let two = query().kinds([TransactionKind::Bill, TransactionKind::VendorCredit]);
    assert_eq!(found(&two), [1200, 200]);
    assert_eq!(found(&query().kinds([TransactionKind::Paycheck])), []);
    assert_eq!(found(&query().kinds([])), NEWEST_FIRST);

    let winter = query().dated(day("2024-02-01")..=day("2024-03-15"));
    assert_eq!(found(&winter), [1000, 900, 800, 700]);
    let start = query().dated(Period::until(day("2023-05-10")));
    assert_eq!(found(&start), [200, 100]);

    let ada = query().name(NameId::new(101));
    assert_eq!(found(&ada), [2400, 2000, 700, 500, 400]);
    let jobs = ada.clone().with_jobs();
    assert_eq!(found(&jobs), [2400, 2000, 1800, 1600, 700, 500, 400]);
    let vendor = query().name(NameId::new(202)).with_jobs();
    assert_eq!(found(&vendor), [2300]);
    assert_eq!(found(&ada.kinds([TransactionKind::Invoice])), [400]);

    assert_eq!(found(&query().number("cm-3")), [700]);
    assert_eq!(found(&query().number("CM")), []);
}

#[test]
fn transactions_are_found_by_their_own_lines() {
    let checking = query().account(AccountId::new(1));
    assert_eq!(found(&checking), [2300, 1700, 1300, 600, 300, 100]);
    let paid = query().paid(PaidState::Paid);
    assert_eq!(found(&paid), [2400, 900, 800, 700, 500, 300, 200]);
    let unpaid = query().paid(PaidState::Unpaid);
    assert_eq!(found(&unpaid), [1500, 1200, 400]);
    let owed = unpaid.account(AccountId::new(3));
    assert_eq!(found(&owed), [400]);
}

#[test]
fn transactions_are_found_by_a_text() {
    // In a memo, in a number, and in the full name of who they are with.
    assert_eq!(found(&query().text("PAPER")), [1100, 1000]);
    assert_eq!(found(&query().text("104")), [1800, 800, 400]);
    let name = query().text("ada st");
    assert_eq!(found(&name), [2400, 2000, 1800, 1600, 700, 500, 400]);
    assert_eq!(found(&query().text("stores:annex")), [1800, 1600]);
    assert_eq!(found(&query().text("no such thing")), []);
    assert_eq!(found(&query().text("")), NEWEST_FIRST);
    let both = query()
        .text("paper")
        .dated(Period::until(day("2024-03-15")));
    assert_eq!(found(&both), [1000]);
}

#[test]
fn transactions_are_found_by_whether_they_count() {
    let mut all = NEWEST_FIRST.to_vec();
    all.insert(6, 1900);
    assert_eq!(found(&query().posting(PostingFilter::All)), all);
    let only = |state| found(&query().posting(PostingFilter::Only(state)));
    assert_eq!(only(PostingState::Pending), [1800]);
    assert_eq!(only(PostingState::Memorized), [1900]);
    assert_eq!(only(PostingState::NonPosting), [2000, 1600, 1400]);
    assert_eq!(only(PostingState::Posted).len(), 20);
}

/// What is found in an order, from its first end and from its last.
fn both_ways(order: TransactionOrder) -> [Vec<i32>; 2] {
    let ordered = |direction| found(&query().order(order, direction));
    [
        ordered(Direction::Ascending),
        ordered(Direction::Descending),
    ]
}

#[test]
fn an_order_has_two_ends() {
    let mut oldest_first = NEWEST_FIRST.to_vec();
    oldest_first.reverse();
    let by_date = both_ways(TransactionOrder::Date);
    assert_eq!(by_date, [oldest_first.clone(), NEWEST_FIRST.to_vec()]);
    // They were entered in the order of their days.
    let entered = both_ways(TransactionOrder::Entered);
    assert_eq!(entered, [oldest_first, NEWEST_FIRST.to_vec()]);
}

#[test]
fn numbers_are_ordered_as_people_order_them() {
    let [ascending, descending] = both_ways(TransactionOrder::Number);
    assert_eq!(
        ascending,
        [
            100, 2500, 500, 300, 1700, 400, 800, 1800, 200, 2100, 2200, 700, 2000, 1400, 1500,
            1600, 1200, 900, 600, 1000, 1100, 1300, 2300, 2400,
        ]
    );
    // What has no number is last both ways, and then in the order of the days.
    assert_eq!(descending[..3], [900, 1200, 1600]);
    assert_eq!(descending[17..], [100, 2400, 2300, 1300, 1100, 1000, 600]);
}

#[test]
fn names_are_ordered_by_their_full_names() {
    let [ascending, descending] = both_ways(TransactionOrder::Name);
    assert_eq!(
        ascending,
        [
            400, 500, 700, 2000, 2400, 1600, 1800, 800, 900, 200, 300, 1000, 1100, 1200, 1400,
            1500, 1700, 2300, 100, 600, 1300, 2100, 2200, 2500,
        ]
    );
    assert_eq!(descending[..3], [2300, 1700, 1500]);
    assert_eq!(descending[18..], [2500, 2200, 2100, 1300, 600, 100]);
}

#[test]
fn totals_are_ordered_in_the_home_currency() {
    let [_, descending] = both_ways(TransactionOrder::Total);
    assert_eq!(
        descending,
        [
            400, 1600, 300, 200, 2000, 900, 800, 1400, 600, 500, 1300, 700, 1800, 2200, 1500, 1000,
            1200, 2300, 1100, 2400, 1700, 2500, 2100, 100,
        ]
    );
}

#[test]
fn due_dates_are_ordered_by_day() {
    let [ascending, _] = both_ways(TransactionOrder::DueDate);
    let due = [200, 400, 700, 800, 1400, 1500, 1800, 2000, 1600];
    assert_eq!(ascending[..9], due);
    assert_eq!(ascending[9..12], [100, 300, 500]);
}

#[test]
fn a_page_is_a_part_of_what_is_found() {
    let company = support::books();
    let invoices = query().kinds([TransactionKind::Invoice]);
    let ids = |page: Page| -> (Vec<i32>, usize) {
        let page = company.transactions(&invoices, page).unwrap();
        let ids = page.rows.iter().map(|row| row.id.number());
        (ids.collect(), page.row_count)
    };
    assert_eq!(ids(Page::new(0, 2)), (vec![1800, 800], 3));
    assert_eq!(ids(Page::new(2, 2)), (vec![400], 3));
    assert_eq!(ids(Page::new(3, 2)), (vec![], 3));
    assert_eq!(ids(Page::new(usize::MAX, usize::MAX)), (vec![], 3));
    assert_eq!(ids(Page::new(1, 0)), (vec![], 3));
    assert_eq!(ids(Page::new(0, 3)), (vec![1800, 800, 400], 3));
}

#[test]
fn a_row_has_what_a_list_shows() {
    let company = support::books();
    let query = query().posting(PostingFilter::All);
    let page = company.transactions(&query, Page::all()).unwrap();
    let row = |id| {
        let mut rows = page.rows.iter();
        rows.find(|row| row.id == TransactionId::new(id)).unwrap()
    };
    let invoice = row(400);
    assert_eq!(invoice.kind, TransactionKind::Invoice);
    assert_eq!(invoice.date, day("2023-11-15"));
    assert_eq!(invoice.number.as_deref(), Some("1041"));
    assert_eq!(invoice.name, Some(NameId::new(101)));
    assert_eq!(invoice.memo.as_deref(), Some("Spring order"));
    assert_eq!(invoice.account, Some(AccountId::new(3)));
    assert_eq!(forms::money(invoice.amount), "202.00 (1)");
    assert_eq!(forms::money(invoice.total), "202.00 (1)");
    assert_eq!(invoice.due_date, Some(day("2023-12-15")));
    assert!(matches!(invoice.open, OpenState::Open(open) if open.to_string() == "45.00"));
    assert_eq!(invoice.posting, PostingState::Posted);

    let foreign = row(800);
    assert_eq!(forms::money(foreign.total), "100.00 (2)");
    assert_eq!(forms::money(foreign.home_total), "110.00 (1)");
    let journal = row(100);
    assert_eq!(forms::money(journal.amount), "1000.00 (1)");
    assert_eq!((journal.total, journal.home_total), (None, None));
    let discount = row(2400);
    assert_eq!(forms::money(discount.total), "0.00 (1)");
    assert_eq!(forms::money(discount.amount), "-1.00 (1)");
    assert!(row(1700).is_voided() && !invoice.is_voided());
    assert_eq!(row(1900).posting, PostingState::Memorized);
}

#[test]
fn a_row_agrees_with_its_transaction() {
    let company = support::books();
    let query = query().posting(PostingFilter::All);
    let page = company.transactions(&query, Page::all()).unwrap();
    assert_eq!(page.row_count, 25);
    for row in page.rows {
        let read = company.transaction(row.id).unwrap().unwrap();
        let of = format!("transaction {}", row.id);
        assert_eq!((row.kind, row.date), (read.kind, read.date), "{of}");
        assert_eq!((&row.number, &row.memo), (&read.number, &read.memo), "{of}");
        assert_eq!(row.name, read.name, "{of}");
        assert_eq!(row.total, read.total, "{of}");
        assert_eq!(row.home_total, read.home_total, "{of}");
        assert_eq!(row.origin, read.origin, "{of}");
        assert_eq!(
            (row.open, row.due_date),
            (read.open_state, read.due_date),
            "{of}"
        );
        assert_eq!(
            (row.open(), row.currency),
            (read.open(), read.currency),
            "{of}"
        );
        assert_eq!(row.is_closed(), read.is_closed(), "{of}");
        assert_eq!(row.is_fully_received(), read.is_fully_received(), "{of}");
        assert_eq!(row.is_fully_invoiced(), read.is_fully_invoiced(), "{of}");
        let own = read.own_line.unwrap();
        assert_eq!((row.account, row.amount), (own.account, own.amount), "{of}");
        assert_eq!(row.order, own.order, "{of}");
    }
}

#[test]
fn a_row_says_how_far_its_order_is() {
    let own = |id| ("abmc_purchase_order_lineitem", "target_id", id);
    let state = |books: Changed, id| {
        let company = books.company();
        let row = company.transaction_summary(TransactionId::new(id));
        let row = row.unwrap().unwrap();
        [
            row.is_closed(),
            row.is_fully_received(),
            row.is_fully_invoiced(),
        ]
    };
    let yes = || Value::Boolean(true);
    assert_eq!(state(Changed::books(), 1400), [false; 3]);
    let received = Changed::books().set(own(1401), "is_received_bool", yes());
    assert_eq!(state(received, 1400), [false, true, false]);
    let closed = Changed::books()
        .set(own(1401), "is_manually_closed_bool", yes())
        .set(own(1401), "is_received_bool", yes());
    assert_eq!(state(closed, 1400), [true, true, false]);
    let sold = ("abmc_sales_order_lineitem", "target_id", 1601);
    let invoiced = Changed::books().set(sold, "is_invoiced_bool", yes());
    assert_eq!(state(invoiced, 1600), [false, false, true]);
    // What is no order is in none of these states.
    assert_eq!(state(Changed::books(), 400), [false; 3]);
}

#[test]
fn a_row_of_a_transfer_says_where_it_goes() {
    let company = support::books();
    let row = |id| {
        let row = company.transaction_summary(TransactionId::new(id));
        let row = row.unwrap().unwrap();
        (row.account, row.other_account)
    };
    let [checking, euros, receivable] = [1, 2, 3].map(AccountId::new);
    assert_eq!(row(1300), (Some(checking), Some(euros)));
    let read = forms::transaction(&company, 1300);
    let own = read.own_line.unwrap();
    assert_eq!(own.other_side, OtherSide::Account(euros));
    // The money leaves the account that the transfer is of.
    let transfer = |id| {
        let row = company.transaction_summary(TransactionId::new(id));
        row.unwrap().unwrap().transfer()
    };
    let moved = transfer(1300).unwrap();
    assert_eq!((moved.from, moved.to), (Some(checking), Some(euros)));
    assert_eq!(read.details, Details::Transfer(moved));
    assert_eq!((transfer(400), transfer(1400)), (None, None));
    // Nothing else is moved from one account to another.
    assert_eq!(row(400), (Some(receivable), None));
    let (_, order) = row(1400);
    assert_eq!(order, None);
}

#[test]
fn a_transaction_is_found_by_its_kind_and_number() {
    let company = support::books();
    let find = |kind, number| company.find(kind, number).unwrap();
    let invoice = find(TransactionKind::Invoice, "1041");
    assert_eq!(invoice, Some(TransactionId::new(400)));
    let credit = find(TransactionKind::CreditMemo, "cm-3");
    assert_eq!(credit, Some(TransactionId::new(700)));
    assert_eq!(find(TransactionKind::Bill, "1041"), None);
    assert_eq!(find(TransactionKind::Invoice, ""), None);
    let all = company.find_all(TransactionKind::Invoice, "1041").unwrap();
    assert_eq!(all, [TransactionId::new(400)]);
}

#[test]
fn only_the_memos_of_the_rows_that_are_shown_are_looked_up() {
    let company = counted::books();
    let fetched = || company.source().fetched();
    let invoices = TransactionQuery::new().kinds([TransactionKind::Invoice]);
    let page = company.transactions(&invoices, Page::new(0, 2)).unwrap();
    assert_eq!((page.rows.len(), page.row_count, fetched()), (2, 3, 2));
    let unpaid = invoices.paid(PaidState::Unpaid);
    let page = company.transactions(&unpaid, Page::new(0, 2)).unwrap();
    assert_eq!((page.rows.len(), fetched()), (1, 3));
}

#[test]
fn a_page_counts_every_kind_that_is_found() {
    let company = support::books();
    let query = TransactionQuery::new().name(NameId::new(102));
    let page = company.transactions(&query, Page::new(0, 1)).unwrap();
    let counts = page.kinds.iter().map(|kind| (kind.kind, kind.count));
    let expected = [(TransactionKind::Invoice, 1), (TransactionKind::Payment, 1)];
    assert_eq!(counts.collect::<Vec<_>>(), expected);
}

#[test]
fn transactions_are_found_by_an_account_that_any_line_posts_to() {
    // The asset account of the stock, which only the build is of.
    let stock = AccountId::new(6);
    assert_eq!(found(&query().account(stock)), [2200]);
    let posts = found(&query().posts_to(stock));
    assert_eq!(posts, [2200, 2100, 1500, 1200, 700, 400, 200]);
    let ledger = support::books();
    let ledger = ledger.ledger().unwrap().of_account(stock);
    let mut posted: Vec<i32> = ledger.map(|posting| posting.transaction.number()).collect();
    posted.sort_unstable_by_key(|&id| Reverse(id));
    posted.dedup();
    assert_eq!(posts, posted);
    assert_eq!(found(&query().posts_to(AccountId::new(99))), []);
    let invoices = query().posts_to(stock).kinds([TransactionKind::Invoice]);
    assert_eq!(found(&invoices), [400]);
    // The invoice takes 60 of it, and the build a part of what it adds.
    let amount = |text: &str| text.parse::<Decimal>().unwrap();
    let sixty = query().posts_to(stock).amount(amount("60")..=amount("60"));
    assert_eq!(found(&sixty), [400]);
    let more = query().posts_to(stock).amount(amount("50")..);
    assert_eq!(found(&more), [400, 200]);
}

#[test]
fn transactions_are_found_by_their_amount() {
    let amount = |text: &str| text.parse::<Decimal>().unwrap();
    let between = query().amount(amount("150")..=amount("202"));
    assert_eq!(found(&between), [1600, 400, 300, 200]);
    let exactly = query().amount(amount("202")..=amount("202"));
    assert_eq!(found(&exactly), [400]);
    let less = query().amount(amount("150")..amount("202"));
    assert_eq!(found(&less), [1600, 300, 200]);
    let at_least = query().amount(amount("1000")..);
    assert_eq!(found(&at_least), [100]);
    // A check of 200 is one of 200, whichever way its bank goes.
    let check = query().kinds([TransactionKind::BillPaymentCheck]);
    assert_eq!(found(&check.amount(amount("200")..)), [300]);
}

#[test]
fn the_summary_of_a_transaction_is_its_row() {
    let company = support::books();
    let query = query().posting(PostingFilter::All);
    let page = company.transactions(&query, Page::all()).unwrap();
    for row in page.rows {
        let summary = company.transaction_summary(row.id).unwrap();
        assert_eq!(summary, Some(row));
    }
    let nothing = company.transaction_summary(TransactionId::new(99_999));
    assert_eq!(nothing.unwrap(), None);
}

#[test]
fn a_row_is_overdue_when_its_transaction_is() {
    let company = support::books();
    let page = company.transactions(&query(), Page::all()).unwrap();
    let overdue = |date: &str| -> Vec<i32> {
        let date = day(date);
        let rows = page.rows.iter().filter(|row| row.is_overdue(date));
        rows.map(|row| {
            let read = company.transaction(row.id).unwrap().unwrap();
            assert!(read.is_overdue(date), "transaction {}", row.id);
            row.id.number()
        })
        .collect()
    };
    // Invoice 1041 was due on 2023-12-15, and the item receipt on 2024-05-10.
    assert_eq!(overdue("2023-12-15"), []);
    assert_eq!(overdue("2023-12-16"), [400]);
    assert_eq!(overdue("2024-06-30"), [1500, 400]);
    // It is the rule of the open items.
    let open = [Side::Receivable, Side::Payable].map(|side| {
        let open = company.open_items(side, AsOf::Latest).unwrap();
        let overdue = open.overdue(day("2024-06-30"));
        let overdue = overdue.map(|item| item.transaction.number());
        overdue.collect::<Vec<_>>()
    });
    assert_eq!(open.concat(), [400, 1500]);
}

#[test]
fn what_is_open_on_a_check_or_a_journal_entry_is_found() {
    // A journal entry that a customer owes 5 by.
    let line = ("abmc_general_journal_lineitem", "target_id", 2501);
    let company = Changed::books()
        .set(line, "account_id", whole(3))
        .set(line, "customer_id", whole(101))
        .company();
    let unpaid = query().paid(PaidState::Unpaid);
    let page = company.transactions(&unpaid, Page::all()).unwrap();
    let rows: Vec<i32> = page.rows.iter().map(|row| row.id.number()).collect();
    let open = [Side::Receivable, Side::Payable].map(|side| {
        let open = company.open_items(side, AsOf::Latest).unwrap();
        let items = open.iter().map(|item| item.transaction.number());
        items.collect::<Vec<_>>()
    });
    let mut open = open.concat();
    open.sort_unstable_by_key(|&id| Reverse(id));
    assert_eq!(rows, open);
    assert!(rows.contains(&2500));
    let read = company.transaction(TransactionId::new(2500)).unwrap();
    assert_eq!(read.unwrap().is_paid(), Some(false));
}

#[test]
fn what_is_paid_has_nothing_left_in_its_own_currency() {
    let company = support::books();
    let row = company.transaction_summary(TransactionId::new(800));
    let row = row.unwrap().unwrap();
    // A paid invoice in euros, an invoice, and a journal entry.
    assert_eq!(forms::money(row.open()), "0.00 (2)");
    let open = |id| {
        let row = company.transaction_summary(TransactionId::new(id));
        forms::money(row.unwrap().unwrap().open())
    };
    assert_eq!((open(400), open(100)), ("45.00 (1)".into(), "-".into()));
}

#[test]
fn the_first_and_the_last_day_are_known_without_the_ledger() {
    let company = counted::books();
    let days = company.transaction_days().unwrap().unwrap();
    assert_eq!(days, Period::between(day("2023-03-01"), day("2024-06-30")));
    // Only the headers were read: those of twenty forms and those of payroll.
    assert!(company.source().scanned() <= 23);
    // A memorized transaction is of no day, and an order is of its own.
    let memorized = ("abmc_check_header", "transaction_id", 1900);
    let order = ("abmc_sales_order_header", "transaction_id", 1600);
    let days = |books: Changed| books.company().transaction_days().unwrap().unwrap();
    let late = || date("2031-01-01");
    let books = Changed::books().set(memorized, "transaction_date", late());
    assert_eq!(days(books).last(), Some(day("2024-06-30")));
    let books = Changed::books().set(order, "transaction_date", late());
    assert_eq!(days(books).last(), Some(day("2031-01-01")));
    let nothing = support::with_preferences(&[]);
    assert_eq!(nothing.transaction_days().unwrap(), None);
}

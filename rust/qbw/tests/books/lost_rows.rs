//! Answers with rows whose header cannot be read: what they show, and how much they
//! say that they lack.

use qbw::check::{CheckKind, Subject};
use qbw::chrono::NaiveDate;
use qbw::query::TransactionQuery;
use qbw::{AccountId, AsOf, Company, ItemId, NameId, Page, Period, Side, TransactionId};

use crate::changed::{Changed, Key, number, text, whole};
use crate::small_owing;
use crate::small_owing::{BUILDER, Line, Link};

const RECEIVABLE: AccountId = AccountId::new(3);
const ADA: NameId = NameId::new(101);
const INVOICE: TransactionId = TransactionId::new(400);
const PAYMENT: TransactionId = TransactionId::new(500);
const INVOICE_HEADER: Key = ("abmc_invoice_header", "transaction_id", 400);

fn today() -> NaiveDate {
    "2024-06-30".parse().unwrap()
}

/// How much the register of the receivables lacks.
fn register_lacks(company: &Company<Changed>) -> u32 {
    let register = company.register(RECEIVABLE, Period::all(), Page::all());
    register.unwrap().unwrap().left_out.rows
}

#[test]
fn a_row_without_its_header_has_its_amounts_and_no_number() {
    let sound = Changed::books().company();
    let lost = Changed::books().unreadable(INVOICE_HEADER).company();
    let [sound, lost] = [&sound, &lost].map(|company| {
        let register = company.register(RECEIVABLE, Period::all(), Page::all());
        register.unwrap().unwrap()
    });
    assert_eq!(sound.rows.len(), lost.rows.len());
    for (sound, lost) in sound.rows.iter().zip(&lost.rows) {
        assert_eq!((sound.amount, sound.balance), (lost.amount, lost.balance));
        let is_of_invoice = lost.transaction == INVOICE;
        assert_eq!(
            lost.number.is_none(),
            is_of_invoice || sound.number.is_none()
        );
        assert_eq!((sound.name, sound.date), (lost.name, lost.date));
    }
    assert_eq!((sound.left_out.rows, lost.left_out.rows), (0, 1));
}

#[test]
fn a_lost_header_counts_once_whatever_shows_it() {
    let company = Changed::books().unreadable(INVOICE_HEADER).company();
    // Four links of the payment lead to the invoice.
    let line_links = company.line_links(PAYMENT).unwrap().unwrap();
    let to_invoice = line_links.links.iter().filter(|link| link.other == INVOICE);
    assert_eq!(to_invoice.count(), 4);
    let links = company.links(PAYMENT).unwrap().unwrap();
    assert!(links.iter().all(|link| link.other.id != INVOICE));
    assert_eq!(links.left_out.rows, 1);
    let postings = company.ledger().unwrap().of_transaction(INVOICE);
    let postings: Vec<_> = postings.collect();
    assert_eq!(postings.len(), 6);
    let texts = company.posting_texts(&postings).unwrap();
    assert!(texts.rows.iter().all(|text| text.number.is_none()));
    assert_eq!(texts.left_out.rows, 1);
    // What several answers lack is one row, too.
    let open = company.open_items(Side::Receivable, today()).unwrap();
    let found = company.transactions(&TransactionQuery::new(), Page::all());
    let both = open.left_out.union(found.unwrap().left_out);
    assert_eq!(both.union(links.left_out).rows, 1);
}

#[test]
fn only_the_rows_of_a_page_need_their_headers() {
    let company = Changed::books().unreadable(INVOICE_HEADER).company();
    let page = |page| {
        let register = company.register(RECEIVABLE, Period::all(), page);
        let register = register.unwrap().unwrap();
        let is_there = register.rows.iter().any(|row| row.transaction == INVOICE);
        (is_there, register.left_out.rows)
    };
    assert_eq!(page(Page::new(0, 1)), (true, 1));
    assert_eq!(page(Page::new(1, 9)), (false, 0));
}

// Nothing tells which of the two is whose.
#[test]
fn a_lost_header_may_be_any_that_cannot_be_read_of_its_form() {
    let two = || Changed::books().unreadable(INVOICE_HEADER);
    let of_one_form = two().unreadable(("abmc_invoice_header", "transaction_id", 800));
    assert_eq!(register_lacks(&of_one_form.company()), 2);
    let deposit = ("abmc_deposit_header", "transaction_id", 600);
    let of_two_forms = two().unreadable(deposit).company();
    assert_eq!(register_lacks(&of_two_forms), 1);
    let open = of_two_forms.open_items(Side::Receivable, AsOf::Latest);
    assert_eq!(open.unwrap().left_out.rows, 1);
    let postings = of_two_forms.ledger().unwrap().of_transaction(INVOICE);
    let texts = of_two_forms.posting_texts(&postings.collect::<Vec<_>>());
    assert_eq!(texts.unwrap().left_out.rows, 1);
    let valve = |rows| {
        let valve = of_two_forms.item_overview(ItemId::new(502), rows).unwrap();
        valve.unwrap().left_out.rows
    };
    // The latest of its rows is another transaction's.
    assert_eq!((valve(1), valve(20)), (0, 1));
    // Neither a link nor a movement says what kind of transaction it leads to.
    let credit_memo = TransactionId::new(700);
    let links = of_two_forms.links(credit_memo).unwrap().unwrap();
    assert_eq!(links.left_out.rows, 2);
    let movements = of_two_forms.movements(ItemId::new(502)).unwrap().unwrap();
    assert_eq!(movements.left_out.rows, 2);
}

#[test]
fn a_header_that_is_there_twice_is_not_lost() {
    let company = Changed::books().copied(INVOICE_HEADER, &[]).company();
    assert_eq!(register_lacks(&company), 0);
    // Another one that is lost is none of the two.
    let memo = ("abmc_invoice_header", "transaction_id", 800);
    let company = Changed::books()
        .copied(INVOICE_HEADER, &[])
        .unreadable(memo);
    let euros = company
        .company()
        .register(AccountId::new(4), Period::all(), Page::all());
    assert_eq!(euros.unwrap().unwrap().left_out.rows, 1);
}

#[test]
fn a_header_that_is_nowhere_counts_too() {
    let company = Changed::books().deleted(INVOICE_HEADER).company();
    assert_eq!(register_lacks(&company), 1);
    let links = company.links(PAYMENT).unwrap().unwrap();
    assert_eq!(links.left_out.rows, 1);
    assert_eq!(company.transaction(INVOICE).unwrap(), None);
}

#[test]
fn an_open_item_without_its_header_is_open_all_the_same() {
    let company = Changed::books().unreadable(INVOICE_HEADER).company();
    for as_of in [AsOf::Latest, AsOf::Date(today())] {
        let open = company.open_items(Side::Receivable, as_of).unwrap();
        let invoice = open.items.iter().find(|item| item.transaction == INVOICE);
        let invoice = invoice.unwrap();
        assert_eq!(
            (invoice.number.as_deref(), invoice.open.to_string()),
            (None, "45.00".into())
        );
        assert_eq!(invoice.due_date, "2023-12-15".parse().ok());
        assert_eq!(open.left_out.rows, 1);
        // One that is not open is not looked for.
        let payable = company.open_items(Side::Payable, as_of).unwrap();
        assert!(payable.left_out.is_nothing());
    }
    let bell = company.owed_by(NameId::new(102), today()).unwrap().unwrap();
    assert!(bell.left_out.is_nothing());
}

#[test]
fn an_amount_does_not_lack_a_number() {
    let company = Changed::books().unreadable(INVOICE_HEADER).company();
    let owed = company.names_owed(Side::Receivable, today()).unwrap();
    assert_eq!(owed.owed(ADA).unwrap().home_total.to_string(), "45.00");
    assert!(owed.left_out.is_nothing());
}

#[test]
fn a_lost_number_explains_no_difference() {
    // The links settle a dollar more of the invoice than the file keeps.
    let link = ("abmc_transaction_link", "link_id", 24);
    let books = || Changed::books().set(link, "amount_amt", number("-2.00"));
    let findings = |books: Changed| {
        let check = books.company().check().unwrap();
        let open = check.of(CheckKind::OpenReceivable);
        let open = open.map(|found| (found.subject.clone(), found.causes.clone()));
        (open.collect::<Vec<_>>(), check.left_out.rows)
    };
    let expected = vec![(Subject::Transaction(INVOICE), Vec::new())];
    assert_eq!(findings(books()), (expected.clone(), 0));
    let lost = books().unreadable(INVOICE_HEADER);
    assert_eq!(findings(lost), (expected, 1));
}

// The payment has lost everything but its link, so nothing says when it was.
#[test]
fn a_payment_that_nothing_dates_is_counted_in_what_is_open_on_a_day() {
    let lines =
        [Line::new(101, "2024-01-05", small_owing::RECEIVABLE, BUILDER, "70.00").open("70.00")];
    let links: &[Link] = &[(4, 301, 101, "-30.00")];
    let company = small_owing::company(&lines, links);
    let builder = NameId::new(BUILDER);
    let owed = company.names_owed(Side::Receivable, today()).unwrap();
    let total = owed.owed(builder).unwrap().home_total.to_string();
    assert_eq!((total.as_str(), owed.left_out.rows), ("70.00", 1));
    // The invoice has no header, either, which its number would be in.
    let open = company.open_items(Side::Receivable, today()).unwrap();
    assert_eq!(open.left_out.rows, 2);
    let latest = company.names_owed(Side::Receivable, AsOf::Latest).unwrap();
    assert!(latest.left_out.is_nothing());
    // On the last day there is, it counts.
    let last = company
        .open_items(Side::Receivable, NaiveDate::MAX)
        .unwrap();
    assert_eq!(last.home_total().to_string(), "40.00");
}

#[test]
fn the_date_of_what_posts_nothing_is_in_its_header() {
    // The pending invoice of April as what pays the one of November.
    let link: &[(&str, _)] = &[
        ("link_id", whole(901)),
        ("transaction_id", whole(400)),
        ("transaction_2_id", whole(1800)),
        ("target_1_id", whole(401)),
        ("target_2_id", whole(1802)),
        ("link_type", whole(4)),
        ("amount_amt", number("-5.00")),
    ];
    let books = || Changed::books().added("abmc_transaction_link", link);
    let open = |books: Changed, day: &str| {
        let day: NaiveDate = day.parse().unwrap();
        let open = books.company().owed_by(ADA, day).unwrap().unwrap();
        let invoice = open.items.iter().find(|item| item.transaction == INVOICE);
        (invoice.unwrap().open.to_string(), open.left_out.rows)
    };
    assert_eq!(open(books(), "2024-04-24"), ("46.00".into(), 0));
    assert_eq!(open(books(), "2024-04-25"), ("41.00".into(), 0));
    let lost = || books().unreadable(("abmc_invoice_header", "transaction_id", 1800));
    assert_eq!(open(lost(), "2024-04-25"), ("46.00".into(), 1));
    // The amounts and the number lack the same two rows.
    let both = lost().unreadable(INVOICE_HEADER);
    assert_eq!(open(both, "2024-04-25"), ("46.00".into(), 2));
}

#[test]
fn a_transaction_without_a_header_has_links_in_the_company_s_currency() {
    let header = ("abmc_invoice_header", "transaction_id", 800);
    let wire = TransactionId::new(800);
    let company = Changed::books().unreadable(header).company();
    let links = company.links(wire).unwrap().unwrap();
    let home = company.preferences().unwrap().home_currency;
    assert_eq!((links.currency, links.left_out.rows), (home, 1));
    let lines = company.line_links(wire).unwrap().unwrap();
    assert_eq!(lines.left_out.rows, 1);
    let sound = Changed::books().company().line_links(wire);
    assert!(sound.unwrap().unwrap().left_out.is_nothing());
}

#[test]
fn a_movement_without_a_header_is_one_of_no_kind() {
    let company = Changed::books().unreadable(INVOICE_HEADER).company();
    let movements = company.movements(ItemId::new(502)).unwrap().unwrap();
    let sale = movements.rows.iter().find(|row| row.transaction == INVOICE);
    let sale = sale.unwrap();
    assert_eq!((sale.kind, &sale.number, sale.name), (None, &None, None));
    assert_eq!(movements.left_out.rows, 1);
}

#[test]
fn a_line_counts_once_in_the_texts_of_what_it_posts() {
    // It posts to three accounts. Its amounts can be read.
    let line = ("abmc_invoice_lineitem", "target_id", 402);
    let company = Changed::books().set(line, "memo", whole(5)).company();
    let postings = company.ledger().unwrap().of_transaction(INVOICE);
    let postings: Vec<_> = postings.collect();
    let of_line = postings
        .iter()
        .filter(|posting| posting.line.number() == 402);
    assert_eq!(of_line.count(), 3);
    let texts = company.posting_texts(&postings).unwrap();
    assert_eq!(texts.left_out.rows, 1);
    let parts = company.register(AccountId::new(13), Period::all(), Page::all());
    let parts = parts.unwrap().unwrap();
    let sale = parts.rows.iter().find(|row| row.transaction == INVOICE);
    let sale = sale.unwrap();
    assert_eq!((sale.number.as_deref(), &sale.memo), (Some("1041"), &None));
    assert_eq!(parts.left_out.rows, 1);
}

#[test]
fn a_line_counts_once_in_an_overview_of_its_item() {
    let line = ("abmc_invoice_lineitem", "target_id", 402);
    // The stock needs the quantity, too.
    let damaged = [("quantity_qnty", text("five")), ("other1", whole(5))];
    for (column, value) in damaged {
        let company = Changed::books().set(line, column, value).company();
        let valve = company.item_overview(ItemId::new(502), 20).unwrap();
        assert_eq!(valve.unwrap().left_out.rows, 1, "{column}");
    }
}

#[test]
fn an_item_that_cannot_be_read_is_counted_with_the_others() {
    let item = ("abmc_item_user", "item_id", 506);
    let company = Changed::books().unreadable(item).company();
    // One in stock, and one that is not.
    for item in [502, 501] {
        let overview = company
            .item_overview(ItemId::new(item), 5)
            .unwrap()
            .unwrap();
        assert_eq!(overview.left_out.rows, 1, "item {item}");
    }
}

#[test]
fn a_form_says_that_the_terms_or_the_rep_are_lost() {
    let of = |key: Key| {
        let company = Changed::books().unreadable(key).company();
        let form = |id| {
            company
                .transaction(TransactionId::new(id))
                .unwrap()
                .unwrap()
        };
        let invoice = form(400);
        let texts = (
            invoice.terms_name.clone(),
            invoice.sales_rep_initials.clone(),
        );
        // The payment has neither.
        assert!(form(500).left_out().is_nothing());
        (texts, invoice.left_out().rows)
    };
    let (terms, rep) = (Some("2% 10 Net 30".to_owned()), Some("EM".to_owned()));
    assert_eq!(of(("abmc_terms", "terms_id", 1)), ((None, rep.clone()), 1));
    assert_eq!(
        of(("abmc_sales_rep", "sales_rep_id", 1)),
        ((terms.clone(), None), 1)
    );
    // Others are not missed.
    assert_eq!(of(("abmc_terms", "terms_id", 2)), ((terms, rep), 0));
}

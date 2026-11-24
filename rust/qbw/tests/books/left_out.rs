//! Every figure says what it lacks: what decides it, and could not be read.

use qbw::aging::{AgingBy, AgingOptions};
use qbw::chrono::NaiveDate;
use qbw::query::{PaidState, TransactionQuery};
use qbw::sa17::Value;
use qbw::transactions::TransactionKind;
use qbw::{
    AccountId, AsOf, Company, ItemId, LeftOut, NameId, Page, Period, Result, Side, TransactionId,
};

use crate::changed::{Changed, Key, number, whole};
use crate::small_owing::{BUILDER, Line};
use crate::{small_books, small_owing, support};

const CUSTOMER: NameId = NameId::new(101);
const VENDOR: NameId = NameId::new(201);
const RECEIVABLE: AccountId = AccountId::new(3);
const INVOICE: TransactionId = TransactionId::new(400);
/// A payment of the invoice.
const PAYMENT: TransactionId = TransactionId::new(500);
const RECEIVABLE_ACCOUNT: Key = ("abmc_account_user", "account_id", 3);

fn today() -> NaiveDate {
    "2024-06-30".parse().unwrap()
}

type Figure = fn(&Company<Changed>) -> Result<LeftOut>;

/// How many rows a figure lacks without an account, without the header of invoice
/// 1041, without a customer, and without a customer whom only the list of names
/// cannot read.
type Lacking = [u32; 4];

/// Every figure of the crate, with how much it lacks.
const FIGURES: &[(&str, Lacking, Figure)] = &[
    ("the accounts", [1, 0, 0, 0], |company| {
        Ok(company.accounts()?.left_out())
    }),
    ("the names", [0, 0, 1, 1], |company| {
        Ok(company.names()?.left_out())
    }),
    ("the ledger", [1, 0, 1, 0], |company| {
        Ok(company.ledger()?.left_out())
    }),
    ("a balance", [1, 0, 1, 0], |company| {
        let balance = company.ledger()?.balance(AccountId::new(2), AsOf::Latest);
        Ok(balance.unwrap().left_out)
    }),
    ("customer balances", [1, 0, 1, 0], |company| {
        let balances = company.ledger()?.customer_balances(AsOf::Latest);
        Ok(match balances.get(&CUSTOMER) {
            Some(balance) => balance.left_out,
            None => company.ledger()?.left_out(),
        })
    }),
    ("a vendor's balance", [1, 0, 1, 0], |company| {
        let balance = company.ledger()?.vendor_balance(VENDOR, AsOf::Latest);
        Ok(balance.unwrap().left_out)
    }),
    ("name balances", [1, 0, 1, 1], |company| {
        let balances = company.name_balances(Side::Receivable, AsOf::Latest)?;
        Ok(balances.left_out)
    }),
    ("a name ledger", [1, 0, 1, 0], |company| {
        let ledger = company.name_ledger(Side::Receivable, CUSTOMER, Period::all())?;
        Ok(ledger.unwrap().left_out)
    }),
    ("a register", [1, 0, 1, 0], |company| {
        let register = company.register(AccountId::new(2), Period::all(), Page::all())?;
        Ok(register.unwrap().left_out)
    }),
    ("the register with the invoice", [1, 1, 1, 0], |company| {
        let register = company.register(RECEIVABLE, Period::all(), Page::all())?;
        Ok(register.unwrap().left_out)
    }),
    ("the texts of the invoice", [1, 1, 1, 0], |company| {
        let postings = company.ledger()?.of_transaction(INVOICE);
        let texts = company.posting_texts(&postings.collect::<Vec<_>>())?;
        Ok(texts.left_out)
    }),
    ("the texts of its payment", [1, 0, 1, 0], |company| {
        let postings = company.ledger()?.of_transaction(PAYMENT);
        let texts = company.posting_texts(&postings.collect::<Vec<_>>())?;
        Ok(texts.left_out)
    }),
    ("the open items now", [1, 1, 1, 0], |company| {
        Ok(company.open_items(Side::Receivable, AsOf::Latest)?.left_out)
    }),
    ("the open items of a day", [1, 1, 1, 0], |company| {
        Ok(company.open_items(Side::Receivable, today())?.left_out)
    }),
    ("what is open with vendors", [1, 0, 1, 0], |company| {
        Ok(company.open_items(Side::Payable, AsOf::Latest)?.left_out)
    }),
    ("what a customer owes", [1, 1, 1, 1], |company| {
        Ok(company.owed_by(CUSTOMER, today())?.unwrap().left_out)
    }),
    ("what a vendor is owed", [1, 0, 1, 0], |company| {
        Ok(company.owed_to(VENDOR, today())?.unwrap().left_out)
    }),
    ("what every name owes", [1, 0, 1, 1], |company| {
        Ok(company.names_owed(Side::Receivable, today())?.left_out)
    }),
    ("an aging of names", [1, 0, 1, 1], |company| {
        let options = AgingOptions::new(today());
        Ok(company.aging(Side::Receivable, &options)?.left_out)
    }),
    ("an aging of items", [1, 1, 1, 0], |company| {
        let options = AgingOptions::new(today()).by(AgingBy::Items);
        Ok(company.aging(Side::Receivable, &options)?.left_out)
    }),
    ("unpaid transactions", [1, 1, 0, 0], |company| {
        let unpaid = TransactionQuery::new().paid(PaidState::Unpaid);
        Ok(company.transactions(&unpaid, Page::all())?.left_out)
    }),
    ("what posts to an account", [1, 1, 1, 0], |company| {
        let posts = TransactionQuery::new().posts_to(AccountId::new(2));
        Ok(company.transactions(&posts, Page::all())?.left_out)
    }),
    ("what has a text", [1, 1, 1, 1], |company| {
        let text = TransactionQuery::new().text("spring");
        Ok(company.transactions(&text, Page::all())?.left_out)
    }),
    ("deposits", [1, 0, 0, 0], |company| {
        let deposits = TransactionQuery::new().kinds([TransactionKind::Deposit]);
        Ok(company.transactions(&deposits, Page::all())?.left_out)
    }),
    ("a transaction", [1, 0, 0, 0], |company| {
        Ok(company.transaction(PAYMENT)?.unwrap().left_out())
    }),
    ("a row of a list", [1, 0, 0, 0], |company| {
        Ok(company.transaction_summary(PAYMENT)?.unwrap().left_out)
    }),
    ("the links of the payment", [0, 1, 0, 0], |company| {
        Ok(company.links(PAYMENT)?.unwrap().left_out)
    }),
    ("the links of the invoice", [0, 1, 0, 0], |company| {
        Ok(company.links(INVOICE)?.unwrap().left_out)
    }),
    ("what paid the invoice", [0, 1, 0, 0], |company| {
        Ok(company.paid_by(INVOICE)?.unwrap().left_out)
    }),
    ("what the payment paid", [0, 1, 0, 0], |company| {
        Ok(company.applied_to(PAYMENT)?.unwrap().left_out)
    }),
    (
        "the links of the payment's lines",
        [0, 0, 0, 0],
        |company| Ok(company.line_links(PAYMENT)?.unwrap().left_out),
    ),
    (
        "the links of the invoice's lines",
        [0, 1, 0, 0],
        |company| Ok(company.line_links(INVOICE)?.unwrap().left_out),
    ),
    ("the links of an order", [0, 0, 0, 0], |company| {
        let order = TransactionId::new(1400);
        Ok(company.links(order)?.unwrap().left_out)
    }),
    ("the stock", [0, 0, 0, 0], |company| {
        Ok(company.stock()?.left_out)
    }),
    ("a valuation", [0, 0, 0, 0], |company| {
        Ok(company.valuation(today())?.left_out)
    }),
    ("the movements of an item", [0, 1, 0, 0], |company| {
        Ok(company.movements(ItemId::new(502))?.unwrap().left_out)
    }),
    ("the movements of another item", [0, 0, 0, 0], |company| {
        Ok(company.movements(ItemId::new(507))?.unwrap().left_out)
    }),
    ("open orders", [0, 0, 0, 0], |company| {
        Ok(company.open_orders(Side::Receivable)?.left_out)
    }),
    ("the trial balance", [1, 0, 1, 0], |company| {
        Ok(company.trial_balance(today())?.left_out)
    }),
    ("profit and loss", [1, 0, 1, 0], |company| {
        Ok(company.profit_and_loss(Period::all())?.left_out)
    }),
    ("the balance sheet", [1, 0, 1, 0], |company| {
        Ok(company.balance_sheet(today())?.left_out)
    }),
    ("the general ledger", [1, 0, 1, 0], |company| {
        Ok(company.general_ledger(Period::all())?.left_out)
    }),
    ("the journal", [1, 0, 1, 0], |company| {
        Ok(company.journal(Period::all())?.left_out)
    }),
    ("the account listing", [1, 0, 1, 0], |company| {
        Ok(company.account_listing(today())?.left_out)
    }),
    ("the bank at a glance", [1, 0, 1, 0], |company| {
        Ok(company.overview(today(), 5)?.left_out)
    }),
    ("what is owed at a glance", [1, 1, 1, 0], |company| {
        Ok(company.overview(today(), 5)?.receivable.left_out)
    }),
    ("what is owing at a glance", [1, 0, 1, 0], |company| {
        Ok(company.overview(today(), 5)?.payable.left_out)
    }),
    ("the latest at a glance", [1, 1, 0, 0], |company| {
        Ok(company.overview(today(), 5)?.transactions.left_out)
    }),
    ("a customer at a glance", [1, 0, 1, 1], |company| {
        let overview = company.name_overview(CUSTOMER, today(), 5)?;
        Ok(overview.unwrap().left_out)
    }),
    ("what it owes at a glance", [1, 1, 1, 1], |company| {
        let overview = company.name_overview(CUSTOMER, today(), 5)?;
        Ok(overview.unwrap().owed.unwrap().left_out)
    }),
    ("its latest at a glance", [1, 1, 1, 1], |company| {
        let overview = company.name_overview(CUSTOMER, today(), 5)?;
        Ok(overview.unwrap().transactions.left_out)
    }),
    ("an account at a glance", [1, 1, 1, 0], |company| {
        let overview = company.account_overview(AccountId::new(12), today(), 9)?;
        Ok(overview.unwrap().register.left_out)
    }),
    ("an item at a glance", [1, 1, 1, 0], |company| {
        let overview = company.item_overview(ItemId::new(502), 20)?;
        Ok(overview.unwrap().left_out)
    }),
    ("the check", [1, 1, 1, 1], |company| {
        Ok(company.check()?.left_out)
    }),
];

#[test]
fn every_figure_says_which_rows_it_lacks() {
    let header = ("abmc_invoice_header", "transaction_id", 400);
    let customer = ("abmc_customer_user", "customer_id", 102);
    let damaged = [
        Changed::books().unreadable(RECEIVABLE_ACCOUNT),
        Changed::books().unreadable(header),
        Changed::books().unreadable(customer),
        // A number is no name.
        Changed::books().set(customer, "name", whole(7)),
    ];
    let whole = Changed::books().company();
    let damaged = damaged.map(Changed::company);
    for (what, lacking, left_out) in FIGURES {
        assert_eq!(left_out(&whole).unwrap(), LeftOut::default(), "{what}");
        let left_out = damaged.each_ref().map(|damaged| left_out(damaged).unwrap());
        assert_eq!(left_out.map(|left_out| left_out.rows), *lacking, "{what}");
    }
}

#[test]
fn a_lost_account_takes_a_customer_s_balance_and_says_so() {
    let damaged = Changed::books().unreadable(RECEIVABLE_ACCOUNT).company();
    let ledger = damaged.ledger().unwrap();
    assert_eq!(ledger.customer_balance(CUSTOMER, AsOf::Latest), None);
    // The ledger's own rows are whole.
    assert_eq!((ledger.left_out().rows, ledger.damage().len()), (1, 0));
    let overview = damaged.name_overview(CUSTOMER, today(), 5).unwrap();
    let overview = overview.unwrap();
    assert_eq!(overview.balance.unwrap().home_amount.to_string(), "0.00");
    assert_eq!(overview.left_out.rows, 1);
}

#[test]
fn a_lost_line_may_post_to_the_account_that_is_asked_for() {
    // Neither of them is the own line.
    let line = ("abmc_general_journal_lineitem", "target_id", 102);
    let books = Changed::books().unreadable(line).company();
    let left_out = |query: &TransactionQuery| {
        let page = books.transactions(query, Page::new(0, 0));
        page.unwrap().left_out.rows
    };
    let posts = TransactionQuery::new().posts_to(AccountId::new(10));
    assert_eq!(left_out(&posts), 1);
    assert_eq!(left_out(&TransactionQuery::new()), 0);
}

#[test]
fn a_row_of_a_list_says_that_its_own_line_is_lost() {
    let own = ("abmc_invoice_lineitem", "target_id", 401);
    let other = ("abmc_invoice_lineitem", "target_id", 402);
    let of = |books: Changed| {
        let company = books.company();
        let row = company.transaction_summary(TransactionId::new(400));
        let row = row.unwrap().unwrap();
        (row.total.map(|total| total.to_string()), row.left_out.rows)
    };
    assert_eq!(of(Changed::books().unreadable(own)), (None, 1));
    // Another line may be lost.
    let whole = (Some("202.00".to_owned()), 0);
    assert_eq!(of(Changed::books().unreadable(other)), whole);
}

#[test]
fn a_lost_line_of_an_order_counts_once() {
    // It is read twice, for what every list shows and for what an order does.
    let own = ("abmc_purchase_order_lineitem", "target_id", 1401);
    let books = Changed::books().unreadable(own).company();
    let order = books.transaction_summary(TransactionId::new(1400));
    assert_eq!(order.unwrap().unwrap().left_out.rows, 1);
    let orders = TransactionQuery::new().kinds([TransactionKind::PurchaseOrder]);
    let page = books.transactions(&orders, Page::all()).unwrap();
    assert_eq!(page.left_out.rows, 1);
}

#[test]
fn a_row_of_an_item_whose_header_is_lost_is_shown_without_it() {
    let header = ("abmc_invoice_header", "transaction_id", 400);
    let books = Changed::books().unreadable(header).company();
    let valve = books.item_overview(ItemId::new(502), 20).unwrap().unwrap();
    let mut rows = valve.rows.iter();
    let of_invoice = rows.find(|row| row.posting.transaction == TransactionId::new(400));
    let of_invoice = of_invoice.unwrap();
    assert_eq!((&of_invoice.number, of_invoice.quantity), (&None, None));
    assert_eq!(valve.left_out.rows, 1);
}

#[test]
fn a_lost_line_of_an_account_s_special_use_is_said() {
    // What sales tax is payable to is what that row says.
    let damaged = Changed::books().unreadable(("abmc_account_internal", "account_id", 9));
    let damaged = damaged.company();
    let balances = damaged.ledger().unwrap().left_out();
    assert_eq!(balances.rows, 1);
}

#[test]
fn the_links_of_a_day_s_open_items_are_counted() {
    let books = Changed::books().unreadable(("abmc_transaction_link", "link_id", 3));
    let books = books.company();
    let open = |as_of| books.open_items(Side::Receivable, as_of).unwrap().left_out;
    assert_eq!(open(AsOf::Date(today())).rows, 1);
    // What is open now does not go by the links.
    assert_eq!(open(AsOf::Latest), LeftOut::default());
    let names = books.names_owed(Side::Receivable, today()).unwrap();
    assert_eq!(names.left_out.rows, 1);
    let paid = books.paid_by(TransactionId::new(400)).unwrap().unwrap();
    assert_eq!(paid.left_out.rows, 1);
    let lines = books.line_links(TransactionId::new(400)).unwrap().unwrap();
    assert_eq!(lines.left_out.rows, 1);
}

#[test]
fn a_link_of_a_code_without_a_name_may_settle_something_and_is_counted() {
    let link = ("abmc_transaction_link", "link_id", 3);
    let books = Changed::books().set(link, "link_type", whole(33)).company();
    let open = books.open_items(Side::Receivable, today()).unwrap();
    assert_eq!(open.left_out.rows, 1);
    let code = books.line_links(TransactionId::new(500)).unwrap().unwrap();
    let unknown = code.links.iter().filter(|link| link.code.code() == 33);
    let unknown = unknown.count();
    assert_eq!(unknown, 1);
    let invoice = code.links.iter().find(|link| link.code.code() == 33);
    let invoice = invoice.unwrap().other;
    let paid = books.paid_by(invoice).unwrap().unwrap();
    let applied = books.applied_to(TransactionId::new(500)).unwrap().unwrap();
    assert_eq!(paid.left_out.rows + applied.left_out.rows, 1);
    let check = books.check().unwrap();
    assert_eq!(check.left_out.rows, 1);
}

#[test]
fn a_row_with_a_key_that_another_has_is_damage_and_counts_once() {
    let account = ("abmc_account_user", "account_id", 1);
    let line = ("abmc_general_journal_lineitem", "target_id", 101);
    let header = ("abmc_general_journal_header", "transaction_id", 100);
    let link = ("abmc_transaction_link", "link_id", 3);
    let whole = Changed::books().company();
    for (what, books) in [
        ("an account", Changed::books().copied(account, &[])),
        ("a line", Changed::books().copied(line, &[])),
    ] {
        let books = books.company();
        let sheet = books.trial_balance(today()).unwrap();
        let expected = whole.trial_balance(today()).unwrap();
        assert_eq!(sheet.debits, expected.debits, "{what}");
        assert_eq!(sheet.rows, expected.rows, "{what}");
        assert_eq!(sheet.left_out.rows, 1, "{what}");
    }
    let books = Changed::books().copied(header, &[]).company();
    let all = books.transactions(&TransactionQuery::new(), Page::all());
    let all = all.unwrap();
    assert_eq!((all.row_count, all.left_out.rows), (24, 1));
    let books = Changed::books().copied(link, &[]).company();
    let paid = books.paid_by(TransactionId::new(400)).unwrap().unwrap();
    assert_eq!(paid.total.to_string(), "157.00");
    assert_eq!(paid.left_out.rows, 1);
}

#[test]
fn a_repeated_row_of_the_stock_s_history_is_damage() {
    let history = Changed::books();
    let row = ("abmc_item_history", "item_id", 502);
    let books = history.copied(row, &[]).company();
    let stock = books.stock().unwrap();
    assert_eq!(stock.value.to_string(), "200.00");
    assert_eq!(stock.left_out.rows, 1);
}

#[test]
fn a_table_that_every_file_has_is_missed() {
    let without = |table| Changed::books().without(table).company();
    let lines = without("abmc_general_journal_lineitem");
    assert!(lines.ledger().is_err());
    let error = lines.profit_and_loss(Period::all()).unwrap_err();
    let says = "the company file has no table abmc_general_journal_lineitem";
    assert_eq!(error.to_string(), says);
    let links = without("abmc_transaction_link");
    let error = links.open_items(Side::Receivable, today()).unwrap_err();
    let says = "the company file has no table abmc_transaction_link";
    assert_eq!(error.to_string(), says);
    assert!(links.paid_by(TransactionId::new(400)).is_err());
    let headers = without("abmc_bill_header");
    assert!(headers.transaction(TransactionId::new(400)).is_err());
    // A company without payroll has no tables for it.
    let books = support::books();
    assert!(books.source().table("es_paycheck_lineitem").is_err());
    assert!(books.ledger().is_ok());
}

#[test]
fn what_cannot_be_open_is_not_counted_as_open() {
    // A customer's line, of which something is left, on an account that is not
    // one for what customers owe.
    let line = Line::new(101, "2024-01-05", small_owing::SALES, BUILDER, "70.00");
    let receivable = small_owing::RECEIVABLE;
    let other = Line::new(201, "2024-01-06", receivable, BUILDER, "30.00");
    let company = small_owing::company(&[line, other], &[]);
    let open = company.open_items(Side::Receivable, AsOf::Latest).unwrap();
    assert_eq!(open.home_total().to_string(), "30.00");
}

#[test]
fn an_item_that_is_too_large_is_counted() {
    // An invoice in euros that is paid in part, at no rate that there is.
    let line = ("abmc_invoice_lineitem", "target_id", 801);
    let link = ("abmc_transaction_link", "link_id", 20);
    let part = || Changed::books().set(link, "amount_amt", number("-40.00"));
    let of_bell = |books: Changed| {
        let open = books.company().open_items(Side::Receivable, today());
        let open = open.unwrap();
        let of_bell = open.of(NameId::new(102)).map(|item| item.open.to_string());
        (of_bell.collect::<Vec<_>>(), open.left_out.rows)
    };
    assert_eq!(of_bell(part()), (vec!["60.00".to_owned()], 0));
    let at_no_rate = part().set(line, "exchange_rate", Value::Float(1e30));
    assert_eq!(of_bell(at_no_rate), (Vec::new(), 1));
}

#[test]
fn a_company_without_postings_lacks_nothing() {
    let company = small_books::company(&[small_books::BANK], &[]);
    let ledger = company.ledger().unwrap();
    assert!(ledger.left_out().is_nothing());
    let balance = ledger.balance(AccountId::new(1), AsOf::Latest).unwrap();
    assert!(balance.amount.is_zero() && balance.left_out.is_nothing());
    assert_eq!(ledger.balance(AccountId::new(2), AsOf::Latest), None);
}

#[test]
fn a_lost_line_counts_once_in_an_answer_that_two_parts_make() {
    // The line of the installation, which the ledger and the stock both read.
    let lines = support::damaged_books();
    let valve = lines.item_overview(ItemId::new(502), 20).unwrap().unwrap();
    assert_eq!(valve.left_out.rows, 1);
    // What two answers lack of the same books.
    let [owed, owing] = [Side::Receivable, Side::Payable].map(|side| {
        let open = lines.open_items(side, AsOf::Latest);
        open.unwrap().left_out
    });
    assert_eq!((owed.with(owing).rows, owed.union(owing).rows), (2, 1));
    let accounts = Changed::books().unreadable(RECEIVABLE_ACCOUNT).company();
    let chart = accounts.accounts().unwrap().left_out();
    let books = accounts.ledger().unwrap().left_out();
    assert_eq!(chart.union(books).rows, 1);
    let page = accounts.transactions(&TransactionQuery::new(), Page::all());
    assert_eq!(page.unwrap().left_out.union(books).rows, 1);
    assert_eq!(accounts.check().unwrap().left_out.union(chart).rows, 1);
    // What each lacks by itself is added.
    let links = Changed::books()
        .unreadable(RECEIVABLE_ACCOUNT)
        .unreadable(("abmc_transaction_link", "link_id", 3))
        .company();
    let chart = links.accounts().unwrap().left_out();
    let paid = links.paid_by(TransactionId::new(400)).unwrap().unwrap();
    assert_eq!(chart.union(paid.left_out).rows, 2);
}

#[test]
fn a_customer_that_cannot_be_read_counts_once_at_a_glance() {
    let customer = ("abmc_customer_user", "customer_id", 102);
    let books = Changed::books().unreadable(customer).company();
    let overview = books.name_overview(CUSTOMER, today(), 5).unwrap().unwrap();
    assert_eq!(overview.left_out.rows, 1);
    let balances = books.name_balances(Side::Receivable, AsOf::Latest).unwrap();
    assert_eq!(balances.left_out.rows, 1);
}

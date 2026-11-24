//! The postings and the balances they add up to.

use qbw::chrono::NaiveDate;
use qbw::ledger::{Posting, PostingSource};
use qbw::transactions::{Cleared, TransactionKind};
use qbw::{AccountId, AsOf, CurrencyId, ItemId, LineId, NameId, OtherSide, Problem, TransactionId};

use crate::support;

fn day(text: &str) -> NaiveDate {
    text.parse().unwrap()
}

/// The balances of accounts in the company's currency, at the end of a day.
fn balances(date: &str, accounts: &[i32]) -> Vec<String> {
    let company = support::books();
    let ledger = company.ledger().unwrap();
    let accounts = accounts.iter().map(|&account| AccountId::new(account));
    let balances = accounts.map(|account| ledger.balance(account, day(date)).unwrap().home_amount);
    balances.map(|balance| balance.to_string()).collect()
}

/// The accounts and amounts of a transaction's postings.
fn posted(transaction: i32) -> Vec<(i32, String)> {
    let company = support::books();
    let ledger = company.ledger().unwrap();
    let postings = ledger.of_transaction(TransactionId::new(transaction));
    postings
        .map(|posting| (posting.account.number(), posting.home_amount.to_string()))
        .collect()
}

#[test]
fn every_line_that_posts_is_in_the_ledger_once() {
    let company = support::books();
    let ledger = company.ledger().unwrap();
    assert_eq!(ledger.postings().len(), 51);
    assert!(ledger.left_out().is_nothing() && ledger.damage().is_empty());
    let sorted = ledger
        .postings()
        .is_sorted_by_key(|posting| (posting.account, posting.date, posting.line, posting.source));
    assert!(sorted);
    let of_accounts = company.accounts().unwrap().iter();
    let postings: usize = of_accounts
        .map(|account| ledger.of_account(account.id).len())
        .sum();
    assert_eq!(postings, 51);
}

#[test]
fn a_balance_is_everything_posted_up_to_a_day() {
    let accounts = [1, 2, 3, 6, 9, 12, 13, 14];
    let first_year = [
        "800.00", "0.00", "202.00", "140.00", "-12.00", "-40.00", "-150.00", "60.00",
    ];
    assert_eq!(balances("2023-12-31", &accounts), first_year);
    let in_all = [
        "837.00", "180.00", "45.00", "200.00", "0.00", "-150.00", "-100.00", "40.00",
    ];
    assert_eq!(balances("2024-06-30", &accounts), in_all);
    assert_eq!(balances("2023-02-28", &accounts), ["0.00"; 8]);
}

#[test]
fn an_account_in_another_currency_has_two_balances() {
    let company = support::books();
    let ledger = company.ledger().unwrap();
    let euro_bank = ledger.balance(AccountId::new(2), AsOf::Latest).unwrap();
    assert_eq!(
        (euro_bank.amount.to_string(), euro_bank.amount.currency()),
        ("150.00".to_owned(), CurrencyId::new(2))
    );
    assert_eq!(
        (
            euro_bank.home_amount.to_string(),
            euro_bank.home_amount.currency()
        ),
        ("180.00".to_owned(), CurrencyId::new(1))
    );
    let receivable = ledger.balance(AccountId::new(4), AsOf::Latest).unwrap();
    assert!(receivable.amount.is_zero() && receivable.home_amount.is_zero());
    // Nothing is posted to an account that the company does not have.
    assert_eq!(ledger.balance(AccountId::new(99), AsOf::Latest), None);
}

#[test]
fn a_transaction_posts_its_lines_in_the_order_of_its_form() {
    let invoice = [
        (3, "202.00"),
        (13, "-150.00"),
        (6, "-60.00"),
        (14, "60.00"),
        (12, "-40.00"),
        (9, "-12.00"),
    ];
    let invoice = invoice.map(|(account, amount)| (account, amount.to_owned()));
    assert_eq!(posted(400), invoice);
}

#[test]
fn a_posting_says_what_it_is() {
    let company = support::books();
    let ledger = company.ledger().unwrap();
    let invoice: Vec<Posting> = ledger.of_transaction(TransactionId::new(400)).collect();
    let [own, goods, stock, cost, service, tax] = invoice.as_slice() else {
        panic!("the invoice posts six amounts: {invoice:?}");
    };
    assert_eq!(
        (own.source, own.other_side),
        (PostingSource::OwnLine, OtherSide::Split)
    );
    assert_eq!(
        (own.kind, own.date),
        (TransactionKind::Invoice, day("2023-11-15"))
    );
    assert_eq!(own.name, Some(NameId::new(101)));
    assert_eq!(
        (goods.source, goods.line),
        (PostingSource::Line, LineId::new(402))
    );
    assert_eq!(goods.item, Some(ItemId::new(502)));

    let receivable = OtherSide::Account(AccountId::new(3));
    for (posting, source) in [
        (stock, PostingSource::StockValue),
        (cost, PostingSource::CostOfGoodsSold),
    ] {
        assert_eq!((posting.source, posting.line), (source, LineId::new(402)));
        assert_eq!(
            (posting.other_side, posting.item),
            (receivable, Some(ItemId::new(502)))
        );
        assert_eq!(posting.origin.table, "abmc_invoice_inventory_lineitem");
    }
    assert_eq!(
        (service.other_side, service.customer),
        (receivable, Some(NameId::new(101)))
    );
    // The tax is owed to its agency, for a sale to the customer.
    assert_eq!(
        (tax.name, tax.vendor),
        (Some(NameId::new(202)), Some(NameId::new(202)))
    );
    assert_eq!(tax.customer, Some(NameId::new(101)));
    assert_eq!(own.cleared, Cleared::NotCleared);
}

#[test]
fn what_does_not_post_is_not_in_the_ledger() {
    // A purchase order, a sales order, an estimate, a pending invoice and a
    // memorized check.
    for transaction in [1400, 1600, 1800, 1900, 2000] {
        assert_eq!(posted(transaction), [], "transaction {transaction}");
    }
    // A voided check posts nothing, on two accounts.
    let voided = [(1, "0.00".to_owned()), (15, "0.00".to_owned())];
    assert_eq!(posted(1700), voided);
}

#[test]
fn every_transaction_and_the_ledger_come_to_zero() {
    let company = support::books();
    let ledger = company.ledger().unwrap();
    let zero = |postings: &[Posting]| {
        let amounts = postings.iter().map(|posting| posting.home_amount);
        let sum = amounts.reduce(|sum, amount| sum.checked_add(amount).unwrap());
        sum.is_none_or(|sum| sum.is_zero())
    };
    let all: Vec<Posting> = ledger.postings().collect();
    assert!(zero(&all));
    for transaction in (1..=25).map(|number| TransactionId::new(number * 100)) {
        let postings: Vec<Posting> = ledger.of_transaction(transaction).collect();
        assert!(zero(&postings), "transaction {transaction}");
    }
}

#[test]
fn a_period_has_both_of_its_ends() {
    let company = support::books();
    let ledger = company.ledger().unwrap();
    let checking = AccountId::new(1);
    let lines = |from: &str, to: &str| -> Vec<i32> {
        let postings = ledger.during(checking, day(from)..=day(to));
        postings.map(|posting| posting.line.number()).collect()
    };
    assert_eq!(lines("2024-01-25", "2024-04-20"), [601, 1301, 1701]);
    assert_eq!(lines("2024-04-20", "2024-01-25"), []);
    let last = ledger.of_account(checking).next_back().unwrap();
    assert_eq!(last.line, LineId::new(2301));
}

#[test]
fn a_customer_owes_what_is_on_accounts_receivable() {
    let company = support::books();
    let ledger = company.ledger().unwrap();
    let ada = NameId::new(101);
    let owes = |date: &str| {
        ledger
            .customer_balance(ada, day(date))
            .unwrap()
            .home_amount
            .to_string()
    };
    assert_eq!(owes("2023-12-31"), "202.00");
    assert_eq!(owes("2024-01-31"), "100.00");
    let now = ledger.customer_balance(ada, AsOf::Latest).unwrap();
    assert_eq!(
        (now.home_amount.to_string(), now.amount),
        ("45.00".to_owned(), now.home_amount)
    );

    let in_february = ledger
        .customer_balance(NameId::new(102), day("2024-02-29"))
        .unwrap();
    let (own, home) = (in_february.amount, in_february.home_amount);
    assert_eq!(
        (own.to_string(), own.currency()),
        ("100.00".to_owned(), CurrencyId::new(2))
    );
    assert_eq!(home.to_string(), "110.00");
    // The job has only a pending invoice and an order.
    let job = ledger.customer_balance(NameId::new(103), AsOf::Latest);
    assert_eq!(job, None);

    let all = ledger.customer_balances(AsOf::Latest);
    assert_eq!(
        all.keys().map(|name| name.number()).collect::<Vec<_>>(),
        [101, 102]
    );
    assert_eq!(all.get(&ada), Some(&now));
}

#[test]
fn a_vendor_is_owed_bills_and_sales_tax() {
    let company = support::books();
    let ledger = company.ledger().unwrap();
    let owed = |vendor, date: &str| {
        let balance = ledger
            .vendor_balance(NameId::new(vendor), day(date))
            .unwrap();
        balance.home_amount.to_string()
    };
    assert_eq!(owed(201, "2023-05-31"), "-200.00");
    assert_eq!(owed(201, "2024-06-30"), "-20.00");
    assert_eq!(owed(202, "2023-12-31"), "-12.00");
    assert_eq!(owed(202, "2024-02-01"), "-8.00");
    assert_eq!(owed(202, "2024-06-30"), "0.00");
}

#[test]
fn a_balance_says_what_is_left_out() {
    let company = support::damaged_books();
    let ledger = company.ledger().unwrap();
    // The line for the installation, of 40.
    assert_eq!((ledger.postings().len(), ledger.left_out().rows), (50, 1));
    let [damage] = ledger.damage() else {
        panic!("one line is damaged: {:?}", ledger.damage());
    };
    assert_eq!(damage.table, "abmc_invoice_lineitem");
    assert!(matches!(damage.problem, Problem::Row { .. }));
    let checking = ledger.balance(AccountId::new(1), AsOf::Latest).unwrap();
    assert_eq!(
        (checking.home_amount.to_string(), checking.left_out.rows),
        ("837.00".to_owned(), 1)
    );
    assert_eq!(
        ledger
            .balance(AccountId::new(12), AsOf::Latest)
            .unwrap()
            .home_amount
            .to_string(),
        "-110.00"
    );
    // The chain of the bill is broken, and the bill posts all the same.
    let bill: Vec<i32> = ledger
        .of_transaction(TransactionId::new(200))
        .map(|posting| posting.line.number())
        .collect();
    assert_eq!(bill, [201, 202]);
}

#[test]
fn a_posting_leads_to_its_row() {
    let company = support::books();
    for posting in company.ledger().unwrap().postings() {
        let row = company.stored_row(posting.origin).unwrap().unwrap();
        let line = row.get("target_id").unwrap().as_i64();
        assert_eq!(line, Some(posting.line.number().into()));
    }
}

#[test]
fn a_posting_is_a_debit_or_a_credit() {
    let company = support::books();
    let ledger = company.ledger().unwrap();
    // The payment of euros: 120.00 to the bank, 110.00 off what is owed, and the
    // difference.
    let sides: Vec<[Option<String>; 2]> = ledger
        .of_transaction(TransactionId::new(900))
        .map(|posting| [posting.debit(), posting.credit()])
        .map(|sides| sides.map(|side| side.map(|side| side.to_string())))
        .collect();
    let side = |text: &str| Some(text.to_owned());
    let expected = [
        [side("120.00"), None],
        [None, side("110.00")],
        [None, side("10.00")],
    ];
    assert_eq!(sides, expected);
}

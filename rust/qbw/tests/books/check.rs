//! What is worked out against what is kept.

use qbw::check::{Cause, Check, CheckKind, Figure, Finding, Subject};
use qbw::sa17::Value;
use qbw::{AccountId, CurrencyId, ItemId, Money, NameId, TransactionId};

use crate::changed::{Cell, Changed, date, number};
use crate::{changed, support};

/// How often figures of each kind agree.
fn counts(check: &Check) -> Vec<(CheckKind, usize, usize)> {
    let compared = check.compared.iter();
    let counts = compared.map(|compared| (compared.what, compared.agreed, compared.looked_at));
    counts.collect()
}

fn written(figure: Option<Figure>) -> String {
    figure.map_or_else(|| "none".to_owned(), |figure| figure.to_string())
}

/// What a finding says: `-25.00 instead of -30.00`.
fn says(found: &Finding) -> String {
    format!(
        "{} instead of {}",
        written(found.computed),
        written(found.expected)
    )
}

/// The findings of the sample company with one cell changed, without the one that
/// it has anyway.
fn findings(cell: Cell, value: Value) -> Vec<Finding> {
    let mut check = changed::books(vec![(cell, value)]).check().unwrap();
    check
        .findings
        .retain(|found| found.subject != Subject::Account(AccountId::new(8)));
    check.findings
}

/// The one finding of the sample company with one cell changed.
fn finding(cell: Cell, value: Value) -> Finding {
    let mut findings = findings(cell, value);
    assert_eq!(findings.len(), 1, "{findings:?}");
    findings.remove(0)
}

#[test]
fn the_sample_company_agrees_but_for_a_balance() {
    let check = support::books().check().unwrap();
    let expected = [
        (CheckKind::TransactionBalance, 20, 20),
        (CheckKind::LedgerBalance, 1, 1),
        (CheckKind::HeaderAndLines, 25, 25),
        (CheckKind::LineChain, 25, 25),
        (CheckKind::FormTotal, 22, 22),
        (CheckKind::AccountBalance, 21, 22),
        (CheckKind::CustomerBalance, 3, 3),
        (CheckKind::VendorBalance, 2, 2),
        (CheckKind::OpenForm, 10, 10),
        (CheckKind::OpenReceivable, 6, 6),
        (CheckKind::OpenPayable, 4, 4),
        (CheckKind::ReceivableAccount, 2, 2),
        (CheckKind::PayableAccount, 1, 1),
        (CheckKind::ItemQuantity, 2, 2),
        (CheckKind::ItemCount, 2, 2),
        (CheckKind::ItemValue, 2, 2),
        (CheckKind::ItemPostings, 2, 2),
        (CheckKind::OnOrder, 2, 2),
        (CheckKind::OnSalesOrders, 2, 2),
        (CheckKind::ToBeBuilt, 2, 2),
        (CheckKind::RequiredForBuilds, 2, 2),
        (CheckKind::InventoryAsset, 1, 1),
        (CheckKind::YearlyFigure, 1, 1),
    ];
    assert_eq!(counts(&check), expected);
    let found: Vec<&Finding> = check.findings.iter().collect();
    let [found] = found[..] else {
        panic!("one balance is out of date: {found:?}");
    };
    assert_eq!(found.what, CheckKind::AccountBalance);
    assert_eq!(found.subject, Subject::Account(AccountId::new(8)));
    assert_eq!(says(found), "-25.00 instead of -30.00");
    // It may be out of date, and it may be wrong.
    assert_eq!(found.causes, [Cause::OutOfDate]);
    assert!(!found.is_explained() && check.unexplained().count() == 1);
    assert!(check.left_out.is_nothing() && check.not_compared.is_empty());
}

#[test]
fn a_balance_that_is_kept_wrong_is_out_of_date() {
    let account = finding(
        (
            "abmc_account_internal",
            "account_id",
            1,
            "ending_balance_amt",
        ),
        number("900"),
    );
    assert_eq!(
        (account.what, account.subject.clone()),
        (
            CheckKind::AccountBalance,
            Subject::Account(AccountId::new(1))
        )
    );
    assert_eq!(says(&account), "837.00 instead of 900.00");
    assert_eq!(account.causes, [Cause::OutOfDate]);
    let in_home = finding(
        (
            "abmc_account_internal",
            "account_id",
            2,
            "ending_balance_home_amt",
        ),
        number("181"),
    );
    assert_eq!(says(&in_home), "180.00 instead of 181.00");
    let customer = finding(
        (
            "abmc_customer_internal",
            "customer_id",
            101,
            "end_balance_amt",
        ),
        number("44"),
    );
    assert_eq!(
        (customer.what, customer.subject.clone()),
        (CheckKind::CustomerBalance, Subject::Name(NameId::new(101)))
    );
    assert_eq!(says(&customer), "45.00 instead of 44.00");
    let vendor = finding(
        ("abmc_vendor_internal", "vendor_id", 201, "end_balance_amt"),
        number("-21"),
    );
    assert_eq!(
        (vendor.what, says(&vendor)),
        (
            CheckKind::VendorBalance,
            "-20.00 instead of -21.00".to_owned()
        )
    );
}

/// An amount in the currency that the sample company keeps its books in.
fn home(text: &str) -> Money {
    Money::new(text.parse().unwrap(), CurrencyId::new(1))
}

fn kinds(findings: &[Finding]) -> Vec<CheckKind> {
    findings.iter().map(|found| found.what).collect()
}

#[test]
fn a_transaction_that_does_not_come_to_zero_explains_nothing() {
    // A line of invoice 1041.
    let line = ("abmc_invoice_lineitem", "target_id", 402, "amount_amt");
    let found = findings(line, number("-140"));
    let expected = [
        CheckKind::TransactionBalance,
        CheckKind::LedgerBalance,
        CheckKind::FormTotal,
        CheckKind::AccountBalance,
        CheckKind::YearlyFigure,
    ];
    assert_eq!(kinds(&found), expected);
    assert_eq!(
        found[0].subject,
        Subject::Transaction(TransactionId::new(400))
    );
    assert_eq!(says(&found[0]), "10.00 instead of 0.00");
    assert_eq!(found[1].subject, Subject::Books);
    assert_eq!(says(&found[2]), "192.00 instead of 202.00");
    assert!(
        found
            .iter()
            .all(|found| found.causes.is_empty() && !found.is_explained())
    );
}

#[test]
fn a_broken_chain_has_no_figures() {
    let next = ("abmc_invoice_lineitem", "target_id", 403, "next_target_id");
    let found = finding(next, Value::Integer(0));
    assert_eq!(
        (found.what, found.subject.clone()),
        (
            CheckKind::LineChain,
            Subject::Transaction(TransactionId::new(400))
        )
    );
    assert_eq!((found.computed, found.expected), (None, None));
    assert!(!found.is_explained());
}

#[test]
fn a_cent_in_another_currency_is_rounding() {
    // The line of invoice 1042, which is in euros.
    let line = (
        "abmc_invoice_lineitem",
        "target_id",
        802,
        "foreign_amount_amt",
    );
    let found = finding(line, number("-99.99"));
    assert_eq!(
        (found.what, says(&found)),
        (CheckKind::FormTotal, "99.99 instead of 100.00".to_owned())
    );
    assert_eq!(found.causes, [Cause::Rounding]);
    let more = finding(line, number("-99.90"));
    assert_eq!(more.causes, []);
}

#[test]
fn what_is_kept_open_against_the_links_is_found_and_explains_the_account() {
    let open = (
        "abmc_invoice_lineitem",
        "target_id",
        401,
        "open_balance_amt",
    );
    let found = findings(open, number("50"));
    assert_eq!(
        kinds(&found),
        [
            CheckKind::OpenForm,
            CheckKind::OpenReceivable,
            CheckKind::ReceivableAccount
        ]
    );
    let [form, receivable, account] = &found[..] else {
        unreachable!("there are three");
    };
    for found in [form, receivable] {
        assert_eq!(found.subject, Subject::Transaction(TransactionId::new(400)));
        assert!(!found.is_explained());
    }
    assert_eq!(says(form), "50.00 instead of 45.00");
    assert_eq!(says(receivable), "45.00 instead of 50.00");
    assert_eq!(account.subject, Subject::Account(AccountId::new(3)));
    assert_eq!(says(account), "50.00 instead of 45.00");
    assert_eq!(account.causes, [Cause::KeptAsOpen(home("-5"))]);
    assert!(account.is_explained());
}

#[test]
fn what_is_kept_of_an_item_is_compared() {
    let kept = |column, text| finding(("abmc_item_internal", "item_id", 502, column), number(text));
    let value = kept("value_on_hand_amt", "161");
    assert_eq!(
        (value.what, value.subject.clone()),
        (CheckKind::ItemValue, Subject::Item(ItemId::new(502)))
    );
    assert_eq!(
        (says(&value), value.causes.len()),
        ("160.00 instead of 161.00".to_owned(), 0)
    );
    let quantity = kept("on_hand_qnty", "9");
    assert_eq!(
        (quantity.what, says(&quantity)),
        (CheckKind::ItemQuantity, "8 instead of 9".to_owned())
    );
    let on_order = kept("on_order_qnty", "4");
    assert_eq!(
        (on_order.what, says(&on_order)),
        (CheckKind::OnOrder, "3 instead of 4".to_owned())
    );
    assert_eq!(on_order.causes, [Cause::OutOfDate]);
    let sold = kept("sales_order_qnty", "1.5");
    assert_eq!(
        (sold.what, sold.causes.clone()),
        (CheckKind::OnSalesOrders, vec![Cause::OutOfDate])
    );
    let built = kept("to_be_built_by_pending_build_txns_qnty", "1");
    assert_eq!(
        (built.what, says(&built)),
        (CheckKind::ToBeBuilt, "0 instead of 1".to_owned())
    );
    let needed = kept("required_by_pending_build_txns_qnty", "1");
    assert_eq!(needed.what, CheckKind::RequiredForBuilds);
}

#[test]
fn what_is_posted_without_an_item_explains_the_asset_account() {
    // A line of invoice 1041 for a service, which now posts to the stock.
    let account = ("abmc_invoice_lineitem", "target_id", 403, "account_id");
    let found = findings(account, Value::Integer(6));
    let asset = found
        .iter()
        .find(|found| found.what == CheckKind::InventoryAsset);
    let asset = asset.unwrap();
    assert_eq!(asset.subject, Subject::Account(AccountId::new(6)));
    assert_eq!(says(asset), "200.00 instead of 160.00");
    assert_eq!(asset.causes, [Cause::OtherPostings(home("-40"))]);
    assert!(asset.is_explained());
}

#[test]
fn a_build_that_posted_another_cost_may_explain_the_assembly_and_its_account() {
    // The own line of build B-1.
    let amount = (
        "abmc_item_assembly_lineitem",
        "target_id",
        2201,
        "amount_amt",
    );
    let found = findings(amount, number("50"));
    let of = |what| found.iter().find(|found| found.what == what).unwrap();
    let assembly = of(CheckKind::ItemPostings);
    assert_eq!(assembly.subject, Subject::Item(ItemId::new(507)));
    assert_eq!(says(assembly), "50.00 instead of 40.00");
    assert_eq!(assembly.causes, [Cause::BuildCosts]);
    let asset = of(CheckKind::InventoryAsset);
    assert_eq!(says(asset), "200.00 instead of 210.00");
    assert_eq!(asset.causes, []);
    // How much the cost differs by is what the history says, so a build that is
    // wrong would look like it.
    assert!(!assembly.is_explained() && !asset.is_explained());
}

#[test]
fn the_lines_of_an_item_are_counted() {
    // The line of the item receipt that brings two valves.
    let item = ("abmc_bill_lineitem", "target_id", 1502, "item_id");
    let found = findings(item, Value::Null);
    let counted = found
        .iter()
        .find(|found| found.what == CheckKind::ItemCount);
    assert_eq!(says(counted.unwrap()), "6 instead of 8");
    let postings = found
        .iter()
        .find(|found| found.what == CheckKind::ItemPostings);
    assert_eq!(says(postings.unwrap()), "120.00 instead of 160.00");
    assert!(found.iter().all(|found| found.causes.is_empty()));
}

#[test]
fn a_figure_of_a_year_is_looked_for_in_all_of_them() {
    let fact = |column| ("abmc_cf_facts", "fact_id", 1, column);
    let found = finding(fact("fact_value"), Value::Text("1.00".to_owned()));
    let key = "QBF_NetIncomePnLAccrualInLastFiscalYear";
    assert_eq!(
        (found.what, found.subject.clone()),
        (CheckKind::YearlyFigure, Subject::Fact(key.to_owned()))
    );
    assert_eq!(
        (says(&found), found.causes.len()),
        ("none instead of 1.00".to_owned(), 0)
    );
    let current = Value::Text("QBF_RevenuePnLAccrualInCurrentFiscalYear".to_owned());
    let found = finding(fact("fact_key"), current);
    assert_eq!(found.causes, [Cause::LaterChanges]);
}

#[test]
fn what_cannot_be_read_is_the_cause_and_no_error() {
    let check = support::damaged_books().check().unwrap();
    assert!(!check.left_out.is_nothing());
    let balances = check.of(CheckKind::AccountBalance);
    let causes: Vec<&Cause> = balances.flat_map(|found| &found.causes).collect();
    assert!(!causes.is_empty() && causes.iter().all(|cause| **cause == Cause::Unread));
}

/// What a check of changed books finds, without the balance that is out of date
/// anyway: what was compared, the two figures, and whether it is explained.
fn found(books: Changed) -> Vec<(CheckKind, String, bool)> {
    let check = books.company().check().unwrap();
    let findings = check.findings.iter();
    let findings = findings.filter(|found| found.subject != Subject::Account(AccountId::new(8)));
    let findings = findings.map(|found| (found.what, says(found), found.is_explained()));
    findings.collect()
}

#[test]
fn a_transaction_that_is_gone_is_not_explained() {
    // The journal entry that opened the books, so that the rest comes to zero.
    let lines = "abmc_general_journal_lineitem";
    let books = Changed::books()
        .deleted(("abmc_general_journal_header", "transaction_id", 100))
        .deleted((lines, "target_id", 101))
        .deleted((lines, "target_id", 102));
    let checking = (
        CheckKind::AccountBalance,
        "-163.00 instead of 837.00".to_owned(),
        false,
    );
    let equity = (
        CheckKind::AccountBalance,
        "0.00 instead of -1000.00".to_owned(),
        false,
    );
    assert_eq!(found(books), [checking, equity]);
}

#[test]
fn a_transaction_that_is_too_large_is_not_explained() {
    let line = |id| ("abmc_general_journal_lineitem", "target_id", id);
    let books = Changed::books()
        .set(line(101), "amount_amt", number("1000000"))
        .set(line(102), "amount_amt", number("-1000000"));
    let found = found(books);
    assert_eq!(found.len(), 2);
    assert!(found.iter().all(|(.., is_explained)| !is_explained));
}

#[test]
fn a_value_that_is_kept_wrong_twice_is_not_explained() {
    // The history of the assembly and the item itself say a million.
    let books = Changed::books()
        .set(
            ("abmc_item_history", "target_id", 2201),
            "value_on_hand",
            number("1000000"),
        )
        .set(
            ("abmc_item_internal", "item_id", 507),
            "value_on_hand_amt",
            number("1000000"),
        );
    let check = books.company().check().unwrap();
    let of = |what| check.of(what).next().unwrap();
    let value = of(CheckKind::ItemPostings);
    assert_eq!(says(value), "40.00 instead of 1000000.00");
    assert_eq!(value.causes, [Cause::BuildCosts]);
    assert!(!value.is_explained() && !of(CheckKind::InventoryAsset).is_explained());
}

#[test]
fn a_header_and_its_lines_say_the_same() {
    let header = ("abmc_invoice_header", "transaction_id", 400);
    let day = Changed::books().set(header, "transaction_date", date("2030-01-01"));
    let pending = Changed::books().set(header, "is_no_post_bool", Value::Boolean(true));
    for (what, books) in [("the day", day), ("whether it posts", pending)] {
        let company = books.company();
        let check = company.check().unwrap();
        let found: Vec<&Finding> = check.of(CheckKind::HeaderAndLines).collect();
        let [found] = found[..] else {
            panic!("{what}: {found:?}");
        };
        let invoice = Subject::Transaction(TransactionId::new(400));
        assert_eq!((&found.subject, found.is_explained()), (&invoice, false));
    }
}

#[test]
fn a_file_without_a_history_says_what_it_did_not_compare() {
    let company = Changed::books().without("abmc_item_history").company();
    let check = company.check().unwrap();
    let not_compared = check.not_compared.iter();
    assert!(
        not_compared
            .clone()
            .all(|missing| missing.table == "abmc_item_history")
    );
    let kinds: Vec<CheckKind> = not_compared.map(|missing| missing.what).collect();
    assert!(kinds.contains(&CheckKind::ItemValue) && kinds.contains(&CheckKind::OnOrder));
    assert!(check.get(CheckKind::ItemValue).is_none());
    let accounts = check.get(CheckKind::AccountBalance).unwrap();
    assert_eq!((accounts.agreed, accounts.looked_at), (21, 22));
    for (table, what) in [
        ("abmc_transaction_link", CheckKind::OpenReceivable),
        ("abmc_other_internal", CheckKind::CustomerBalance),
    ] {
        let check = Changed::books().without(table).company().check().unwrap();
        let mut not_compared = check.not_compared.iter();
        assert!(not_compared.any(|missing| missing.what == what), "{table}");
        assert!(check.get(CheckKind::AccountBalance).is_some());
    }
}

#[test]
fn a_finding_can_be_read_as_its_account_is_kept() {
    let books = Changed::books()
        .set(
            ("abmc_vendor_internal", "vendor_id", 201),
            "end_balance_amt",
            number("-90"),
        )
        .set(
            ("abmc_account_internal", "account_id", 1),
            "ending_balance_amt",
            number("900"),
        )
        .company();
    let (check, accounts) = (books.check().unwrap(), books.accounts().unwrap());
    let kept = |subject: Subject| {
        let mut found = check.findings.iter();
        let found = found.find(|found| found.subject == subject).unwrap();
        (says(found), says(&found.as_kept(accounts)))
    };
    let says = |books: &str, kept: &str| (books.to_owned(), kept.to_owned());
    // A card, a vendor and a bank.
    let card = kept(Subject::Account(AccountId::new(8)));
    assert_eq!(
        card,
        says("-25.00 instead of -30.00", "25.00 instead of 30.00")
    );
    let vendor = kept(Subject::Name(NameId::new(201)));
    assert_eq!(
        vendor,
        says("-20.00 instead of -90.00", "20.00 instead of 90.00")
    );
    let bank = kept(Subject::Account(AccountId::new(1)));
    assert_eq!(
        bank,
        says("837.00 instead of 900.00", "837.00 instead of 900.00")
    );
}

#[test]
fn what_is_owed_is_above_zero_in_a_finding_as_it_is() {
    // The bill is kept as open.
    let line = ("abmc_bill_lineitem", "target_id", 201);
    let books = Changed::books().set(line, "is_paid_bool", Value::Boolean(false));
    let books = books.set(line, "open_balance_amt", number("-50")).company();
    let (check, accounts) = (books.check().unwrap(), books.accounts().unwrap());
    let payable = check.of(CheckKind::PayableAccount).next().unwrap();
    assert_eq!(says(payable), "70.00 instead of 20.00");
    assert_eq!(payable.as_kept(accounts), *payable);
    let open = check.of(CheckKind::OpenPayable).next().unwrap();
    assert_eq!(open.as_kept(accounts), *open);
    // The amount of a cause turns with the figures.
    let found = Changed::books()
        .set(
            ("abmc_invoice_lineitem", "target_id", 403),
            "account_id",
            Value::Integer(6),
        )
        .set(
            ("abmc_account_user", "account_id", 6),
            "type",
            Value::Integer(8),
        )
        .company();
    let (check, accounts) = (found.check().unwrap(), found.accounts().unwrap());
    let asset = check.of(CheckKind::InventoryAsset).next().unwrap();
    let kept = asset.as_kept(accounts);
    assert_eq!(
        (says(asset), says(&kept)),
        (
            "200.00 instead of 160.00".into(),
            "-200.00 instead of -160.00".into()
        )
    );
    assert_eq!(kept.causes, [Cause::OtherPostings(home("40"))]);
    assert!(asset.is_explained() && kept.is_explained());
}

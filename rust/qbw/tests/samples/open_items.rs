//! Open items against what the file keeps and against the accounts.

use std::collections::{BTreeMap, HashMap, HashSet};

use qbw::chrono::{Months, NaiveDate};
use qbw::open_items::OpenItems;
use qbw::{AccountId, AsOf, Company, Money, NameId, Period, Side, TransactionId};

use crate::agreement::Agreement;
use crate::support;

const SIDES: [Side; 2] = [Side::Receivable, Side::Payable];

/// Five dates over ten years.
const DATES: [&str; 5] = [
    "2014-12-31",
    "2018-06-30",
    "2021-12-31",
    "2023-06-30",
    "2025-12-31",
];

/// What is open of each transaction, with each name and on each side of the books.
fn by_transaction(open: &OpenItems) -> HashMap<TransactionId, Vec<(Option<NameId>, Money)>> {
    let mut items: HashMap<_, Vec<_>> = HashMap::new();
    for item in open {
        let of_transaction = items.entry(item.transaction).or_default();
        of_transaction.push((item.name, item.open));
    }
    items
}

/// The accounts of a side.
fn accounts(company: &Company, side: Side) -> Vec<AccountId> {
    let accounts = company.accounts().unwrap().iter();
    let accounts = accounts.filter(|account| account.account_type == side.account_type());
    accounts.map(|account| account.id).collect()
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn what_is_kept_as_open_is_the_amount_less_the_links() {
    for company in support::samples() {
        for side in SIDES {
            let kept = company.open_items(side, AsOf::Latest).unwrap();
            let linked = company
                .open_items(side, AsOf::Date(NaiveDate::MAX))
                .unwrap();
            let (kept, linked) = (by_transaction(&kept), by_transaction(&linked));
            let ledger = company.ledger().unwrap();
            let posted = accounts(&company, side)
                .into_iter()
                .flat_map(|id| ledger.of_account(id));
            let transactions: HashSet<TransactionId> =
                posted.map(|posting| posting.transaction).collect();
            let mut agreement = Agreement::default();
            for transaction in transactions {
                agreement.add(kept.get(&transaction) == linked.get(&transaction));
            }
            agreement.check(&format!("{side:?}: kept as open what the links leave"));
        }
    }
}

/// What the items of each account come to in its currency.
fn by_account(open: &OpenItems) -> BTreeMap<AccountId, Money> {
    let mut sums: BTreeMap<AccountId, Money> = BTreeMap::new();
    for item in open {
        let sum = sums
            .entry(item.account)
            .or_insert(Money::zero(item.open.currency()));
        *sum = sum.checked_add(item.open).unwrap();
    }
    sums
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn open_items_come_to_the_accounts_at_five_dates() {
    for company in support::samples() {
        let ledger = company.ledger().unwrap();
        let mut agreement = Agreement::default();
        for date in DATES.map(|date| date.parse().unwrap()) {
            for side in SIDES {
                let open = company.open_items(side, AsOf::Date(date)).unwrap();
                let sums = by_account(&open);
                for account in accounts(&company, side) {
                    if ledger.during(account, Period::until(date)).len() == 0 {
                        continue;
                    }
                    let balance = ledger.balance(account, date).unwrap().amount;
                    let balance = if side == Side::Payable {
                        -balance
                    } else {
                        balance
                    };
                    let sum = sums.get(&account).copied();
                    agreement.add(sum.unwrap_or(Money::zero(balance.currency())) == balance);
                }
            }
        }
        agreement.check("balances of accounts that the open items come to");
    }
}

/// The part of a home balance that may be missing from the open items, for
/// invoices that were paid bit by bit and for what contradicts itself.
const HOME_TOLERANCE: f64 = 0.002;

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn open_items_and_adjustments_come_to_the_home_balances() {
    for company in support::samples() {
        let ledger = company.ledger().unwrap();
        for side in SIDES {
            let open = company.open_items(side, AsOf::Latest).unwrap();
            let balances = accounts(&company, side)
                .into_iter()
                .map(|id| ledger.balance(id, AsOf::Latest).unwrap().home_amount);
            let balance = balances.reduce(add).unwrap();
            let balance = if side == Side::Payable {
                -balance
            } else {
                balance
            };
            let items = add(open.home_total(), open.adjustments);
            let missing = balance.checked_sub(items).unwrap();
            let all = balance.amount().to_f64().abs();
            let share = missing.amount().to_f64().abs() / all;
            println!("{side:?}: {share:.6} of the home balance is in no open item");
            assert!(share < HOME_TOLERANCE);
        }
    }
}

fn add(left: Money, right: Money) -> Money {
    left.checked_add(right).unwrap()
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn a_name_owes_among_all_what_it_owes_alone() {
    for company in support::samples() {
        for side in SIDES {
            let mut names = Agreement::default();
            let all = company.names_owed(side, AsOf::Latest).unwrap();
            for row in &all.rows {
                let alone = match side {
                    Side::Receivable => company.owed_by(row.name, AsOf::Latest),
                    _ => company.owed_to(row.name, AsOf::Latest),
                };
                names.add(alone.unwrap().is_some_and(|alone| {
                    (&row.total, &row.overdue) == (&alone.total, &alone.overdue)
                        && row.home_total == alone.home_total
                        && row.home_overdue == alone.home_overdue
                        && row.adjustments == alone.adjustments
                }));
            }
            names.check(&format!("{side:?}: names that owe what they owe alone"));
        }
    }
}

/// The first days of the quarters of twenty years.
fn quarters() -> impl Iterator<Item = NaiveDate> {
    let first = NaiveDate::from_ymd_opt(2006, 1, 1).unwrap();
    (0..80).map(move |quarter| first + Months::new(3 * quarter))
}

/// How many names in other currencies there are on the days of a book, and how
/// many of them come to their home balance to the cent. The others had something
/// that was paid bit by bit at rounded amounts.
const TIES: &[(usize, usize)] = &[(8_405, 3_907), (4_698, 1_602), (501, 337), (164, 155)];

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn names_in_other_currencies_come_to_their_home_balances_as_often_as_they_did() {
    for company in support::samples() {
        for side in SIDES {
            let (mut names, mut ties) = (0, 0);
            for day in quarters() {
                let balances = company.name_balances(side, day).unwrap();
                let balances = balances.rows.iter().map(|row| (row.name, row.total));
                let balances: HashMap<NameId, Money> = balances.collect();
                for row in &company.names_owed(side, day).unwrap().rows {
                    let home = row.home_total.currency();
                    let is_foreign = row.total.iter().any(|total| total.currency() != home);
                    if let Some(&balance) = balances.get(&row.name).filter(|_| is_foreign) {
                        names += 1;
                        ties += usize::from(balance == add(row.home_total, row.adjustments));
                    }
                }
            }
            println!("{side:?}: {ties} of {names} names in other currencies tie");
            let known = TIES.iter().find(|(known, _)| *known == names);
            assert!(known.is_none_or(|(_, known)| ties >= *known));
        }
    }
}

//! The ledger against what QuickBooks keeps itself, and against a second
//! reading of the postings.

use std::collections::{BTreeMap, HashMap, HashSet};

use qbw::chrono::NaiveDate;
use qbw::ledger::{Balance, Ledger};
use qbw::sa17::{Columns, Decimal, Row, Table};
use qbw::{AccountId, AsOf, Company, NameId, Period};

use crate::agreement::Agreement;
use crate::{agreement, support};

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn every_posting_can_be_read() {
    for company in support::samples() {
        let ledger = company.ledger().unwrap();
        println!("{} postings", ledger.postings().len());
        assert!(ledger.left_out().is_nothing());
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn every_transaction_and_the_whole_ledger_come_to_zero() {
    for company in support::samples() {
        let mut transactions = Agreement::default();
        for entry in company.journal(Period::all()).unwrap().entries() {
            transactions.add(entry.debits == entry.credits);
        }
        transactions.check("transactions that come to zero");
        let ledger = company.ledger().unwrap();
        let sum = ledger.postings().fold(Decimal::default(), |sum, posting| {
            sum.checked_add(posting.home_amount.amount()).unwrap()
        });
        assert!(sum.is_zero());
    }
}

/// The type of accounts that are not part of the books.
const NON_POSTING: i32 = 15;

/// Whether a balance is the one that a row keeps in its second and third column.
fn is_kept(row: &Row, balance: Option<Balance>) -> bool {
    let kept = |column| {
        row.read::<Option<Decimal>>(column)
            .unwrap()
            .unwrap_or_default()
    };
    let kept = Amounts {
        own: kept(1),
        home: kept(2),
    };
    balance.map_or_else(Amounts::default, amounts_of) == kept
}

fn amounts_of(balance: Balance) -> Amounts {
    Amounts {
        own: balance.amount.amount(),
        home: balance.home_amount.amount(),
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn lifetime_balances_are_the_stored_ones() {
    for company in support::samples() {
        let ledger = company.ledger().unwrap();
        let outside = agreement::rows(&company, "abmc_account_user", &["account_id", "type"]);
        let outside = outside
            .iter()
            .filter(|row| row.read::<i32>(1).unwrap() == NON_POSTING);
        let outside: HashSet<i32> = outside.map(|row| row.read(0).unwrap()).collect();
        let columns = [
            "account_id",
            "ending_balance_amt",
            "ending_balance_home_amt",
        ];
        let mut accounts = Agreement::default();
        for row in agreement::rows(&company, "abmc_account_internal", &columns) {
            let account: i32 = row.read(0).unwrap();
            if !outside.contains(&account) {
                // One that was deleted is in no list and has no balance.
                let balance = ledger.balance(AccountId::new(account), AsOf::Latest);
                accounts.add(is_kept(&row, balance));
            }
        }
        accounts.check("accounts with the stored balance");
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn customers_owe_and_vendors_are_owed_what_is_stored() {
    for company in support::samples() {
        let ledger = company.ledger().unwrap();
        let customers = ledger.customer_balances(AsOf::Latest);
        check_names(&company, "customer", &customers);
        let vendors = ledger.vendor_balances(AsOf::Latest);
        check_names(&company, "vendor", &vendors);
    }
}

fn check_names(company: &Company, kind: &str, balances: &BTreeMap<NameId, Balance>) {
    let key = format!("{kind}_id");
    let columns = [key.as_str(), "end_balance_amt", "end_balance_home_amt"];
    let mut names = Agreement::default();
    for row in agreement::rows(company, &format!("abmc_{kind}_internal"), &columns) {
        let name = NameId::new(row.read(0).unwrap());
        names.add(is_kept(&row, balances.get(&name).copied()));
    }
    names.check(&format!("{kind}s with the stored balance"));
}

/// Amounts in the currency of an account and in the company's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Amounts {
    own: Decimal,
    home: Decimal,
}

impl Amounts {
    fn add(&mut self, other: Self) {
        self.own = self.own.checked_add(other.own).unwrap();
        self.home = self.home.checked_add(other.home).unwrap();
    }
}

/// What is posted to an account on a day.
type Days = BTreeMap<(i32, NaiveDate), Amounts>;

/// A second reading of the postings, from the tables themselves: what every
/// line and what it posts besides adds to its account on its day.
struct SecondReading {
    days: Days,
    /// The days of the lines that post.
    posting_lines: HashMap<i64, NaiveDate>,
    foreign_accounts: HashSet<i32>,
}

impl SecondReading {
    fn read(company: &Company) -> Days {
        let home = company.preferences().unwrap().home_currency;
        let accounts = company.accounts().unwrap().iter();
        let foreign = accounts.filter(|account| account.currency != home);
        let mut days = Self {
            days: Days::new(),
            posting_lines: HashMap::new(),
            foreign_accounts: foreign.map(|account| account.id.number()).collect(),
        };
        let lines = |table: &Table<'_>| {
            let definition = table.definition();
            definition.stores_rows()
                && definition.position("next_target_id").is_some()
                && table.name() != "abmc_audit_lineitem"
        };
        let tables: Vec<_> = company.source().tables().collect();
        for table in tables.iter().filter(|table| lines(table)) {
            days.read_lines(*table);
        }
        for table in tables {
            if table.name().ends_with("_inventory_lineitem") {
                days.read_stock(table);
            }
        }
        days.days
    }

    fn read_lines(&mut self, table: Table<'_>) {
        let columns = [
            "target_id",
            "account_id",
            "transaction_date",
            "amount_amt",
            "foreign_amount_amt",
        ];
        let flags = ["is_no_post_bool", "is_memorized_transaction_bool"];
        let columns = Columns::with_optional(table.definition(), columns, flags).unwrap();
        for row in table.scan(&columns) {
            let row = row.unwrap();
            let is_set = |column| row.read::<Option<bool>>(column).unwrap() == Some(true);
            if is_set(5) || is_set(6) {
                continue;
            }
            let date = row.read(2).unwrap();
            self.posting_lines.insert(row.read(0).unwrap(), date);
            self.add(&row, date);
        }
    }

    fn read_stock(&mut self, table: Table<'_>) {
        let columns = [
            "target_id",
            "account_id",
            "target_type",
            "amount_amt",
            "foreign_amount_amt",
        ];
        let columns = Columns::new(table.definition(), columns).unwrap();
        for row in table.scan(&columns) {
            let row = row.unwrap();
            if let Some(&date) = self.posting_lines.get(&row.read::<i64>(0).unwrap()) {
                self.add(&row, date);
            }
        }
    }

    /// Adds the amounts of a row with an account in its second column and the
    /// amounts in its fourth and fifth.
    fn add(&mut self, row: &Row, date: NaiveDate) {
        let account: Option<i32> = row.read(1).unwrap();
        let Some(account) = account.filter(|&account| account != 0) else {
            return;
        };
        let amount = |column| {
            row.read::<Option<Decimal>>(column)
                .unwrap()
                .unwrap_or_default()
        };
        let own = if self.foreign_accounts.contains(&account) {
            amount(4)
        } else {
            amount(3)
        };
        let amounts = Amounts {
            own,
            home: amount(3),
        };
        self.days.entry((account, date)).or_default().add(amounts);
    }
}

/// The same from the ledger.
fn days(ledger: &Ledger) -> Days {
    let mut days = Days::new();
    for posting in ledger.postings() {
        let amounts = Amounts {
            own: posting.amount.amount(),
            home: posting.home_amount.amount(),
        };
        let day = (posting.account.number(), posting.date);
        days.entry(day).or_default().add(amounts);
    }
    days
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn day_end_balances_are_those_of_a_second_reading() {
    for company in support::samples() {
        let ledger = company.ledger().unwrap();
        let (found, expected) = (days(ledger), SecondReading::read(&company));
        assert_eq!(found.len(), expected.len());
        let mut day_ends = Agreement::default();
        let mut running = Amounts::default();
        let mut account = 0;
        for (&(day_account, date), amounts) in &expected {
            if day_account != account {
                (account, running) = (day_account, Amounts::default());
            }
            running.add(*amounts);
            let balance = ledger.balance(AccountId::new(account), date).unwrap();
            day_ends.add(running == amounts_of(balance));
        }
        day_ends.check("balances of an account at the end of a day");
    }
}

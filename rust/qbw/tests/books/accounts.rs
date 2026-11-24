//! The chart of accounts.

use qbw::accounts::{AccountType, SpecialAccount};
use qbw::{AccountId, CurrencyId, Problem};

use crate::support;

#[test]
fn accounts_are_in_quickbooks_order_with_their_full_names() {
    let company = support::books();
    let accounts = company.accounts().unwrap();
    let names: Vec<&str> = accounts
        .iter()
        .map(|account| accounts.full_name(account.id).unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "Checking",
            "Euro bank",
            "Old Savings",
            "Accounts Receivable",
            "Accounts Receivable - EUR",
            "Undeposited Funds",
            "Inventory Asset",
            "Accounts Payable",
            "Company Card",
            "Sales Tax Payable",
            "Opening Balance Equity",
            "Retained Earnings",
            "Sales",
            "Sales:Parts",
            "Cost of Goods Sold",
            "Office",
            "Office:Supplies",
            "Discounts Given",
            "Exchange Gain or Loss",
            "Purchase Orders",
            "Sales Orders",
            "Estimates",
        ]
    );
    assert!(accounts.left_out().is_nothing());
}

#[test]
fn an_inactive_account_is_listed_and_a_deleted_one_is_not() {
    let company = support::books();
    let accounts = company.accounts().unwrap();
    assert!(!accounts.named("Old Savings").unwrap().is_active);
    assert!(
        accounts
            .active()
            .all(|account| account.name != "Old Savings")
    );
    assert!(accounts.get(AccountId::new(22)).is_none());
}

#[test]
fn an_account_has_what_the_window_shows() {
    let company = support::books();
    let accounts = company.accounts().unwrap();
    let checking = accounts.named("checking").unwrap();
    assert_eq!(checking.id, AccountId::new(1));
    assert_eq!(checking.account_type, AccountType::Bank);
    assert_eq!(checking.number.as_deref(), Some("1000"));
    assert_eq!(checking.description.as_deref(), Some("Everyday account"));
    assert_eq!((checking.parent, checking.special), (None, None));
    let created = checking.created.unwrap();
    assert_eq!(created.to_string(), "2023-01-01 09:30:00 UTC");

    let parts = accounts.named("Sales:Parts").unwrap();
    assert_eq!(parts.parent, Some(AccountId::new(12)));
    assert_eq!((parts.number.as_deref(), parts.tax_line), (None, None));
    assert_eq!(
        accounts.get(AccountId::new(12)).unwrap().tax_line,
        Some(1261)
    );
}

#[test]
fn special_accounts_are_known_by_what_they_are_for() {
    let company = support::books();
    let accounts = company.accounts().unwrap();
    let special = |special| -> Vec<&str> {
        let accounts = accounts
            .iter()
            .filter(|account| account.special == Some(special));
        accounts.map(|account| account.name.as_str()).collect()
    };
    let receivable = ["Accounts Receivable", "Accounts Receivable - EUR"];
    assert_eq!(special(SpecialAccount::AccountsReceivable), receivable);
    // The deleted account that kept the same purpose is not listed.
    assert_eq!(
        special(SpecialAccount::RetainedEarnings),
        ["Retained Earnings"]
    );
    let exchange = special(SpecialAccount::ExchangeGainOrLoss);
    assert_eq!(exchange, ["Exchange Gain or Loss"]);
}

#[test]
fn stored_balances_are_in_the_account_s_currency_and_in_the_company_s() {
    let company = support::books();
    let home = company.preferences().unwrap().home_currency;
    let accounts = company.accounts().unwrap();
    let euro_bank = accounts.named("Euro bank").unwrap();
    assert_eq!(euro_bank.currency, CurrencyId::new(2));
    let stored = euro_bank.stored.unwrap();
    assert_eq!(
        (stored.amount.to_string(), stored.amount.currency()),
        ("150.00".to_owned(), CurrencyId::new(2))
    );
    let in_home = stored.home_amount.unwrap();
    assert_eq!(
        (in_home.to_string(), in_home.currency()),
        ("180.00".to_owned(), home)
    );
    // QuickBooks keeps no home balance of this one.
    let retained = accounts.named("Retained Earnings").unwrap();
    assert!(retained.stored.unwrap().home_amount.is_none());
}

#[test]
fn a_reconciliation_that_is_not_finished_is_kept() {
    let company = support::books();
    let accounts = company.accounts().unwrap();
    let reconciling = accounts.named("Checking").unwrap().reconciling.unwrap();
    assert_eq!(reconciling.statement_balance.to_string(), "845.00");
    assert_eq!(reconciling.statement_date, "2024-05-31".parse().ok());
    assert!(accounts.named("Euro bank").unwrap().reconciling.is_none());
}

#[test]
fn damage_to_transactions_leaves_the_accounts_whole() {
    let company = support::damaged_books();
    let accounts = company.accounts().unwrap();
    assert_eq!(
        (accounts.len(), accounts.left_out().is_nothing()),
        (22, true)
    );
}

#[test]
fn an_account_that_cannot_be_read_is_left_out_and_counted() {
    let company = support::with_a_damaged_account();
    let accounts = company.accounts().unwrap();
    assert_eq!((accounts.len(), accounts.left_out().rows), (1, 1));
    let [damage] = accounts.damage() else {
        panic!("one account is damaged: {:?}", accounts.damage());
    };
    assert_eq!(damage.table, "abmc_account_user");
    assert!(matches!(
        damage.problem,
        Problem::Column { column: "type", .. }
    ));
}

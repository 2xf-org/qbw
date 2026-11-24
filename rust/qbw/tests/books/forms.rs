//! Transactions, written down as the tests compare them.

use qbw::transactions::{Line, Transaction};
use qbw::{Company, Money, Price, TransactionId};

/// The transaction of `company` with `id`.
pub(crate) fn transaction(company: &Company, id: i32) -> Transaction {
    let found = company.transaction(TransactionId::new(id)).unwrap();
    found.unwrap_or_else(|| panic!("there is a transaction {id}"))
}

/// An amount with its currency: `100.00 (2)`.
pub(crate) fn money(amount: Option<Money>) -> String {
    written(amount, |amount| format!("{amount} ({})", amount.currency()))
}

/// The lines, each as its role, its account, its item, its quantity, its rate
/// and its amount: `Item 13 502 3 50.00 150.00`. A dash is for nothing.
pub(crate) fn lines(transaction: &Transaction) -> Vec<String> {
    transaction.lines.iter().map(line).collect()
}

fn line(line: &Line) -> String {
    let quantity = line.quantity;
    let parts = [
        format!("{:?}", line.role),
        written(line.account, |account| account.to_string()),
        written(line.item, |item| item.to_string()),
        written(quantity, |quantity| quantity.to_string()),
        written(line.rate, rate),
        written(line.amount, |amount| amount.to_string()),
    ];
    parts.join(" ")
}

pub(crate) fn rate(rate: Price) -> String {
    match rate {
        Price::Each(price) => price.to_string(),
        Price::Percent(percent) => format!("{}%", percent),
        _ => panic!("a price is of one or in percent"),
    }
}

/// The lines that post, each as its account and its amounts in the
/// transaction's currency and in the company's: `12 -100.00 -110.00`.
pub(crate) fn posted(transaction: &Transaction) -> Vec<String> {
    let posted = transaction.lines.iter().filter_map(Line::posted);
    let written = posted.map(|posted| {
        let (amount, home) = (posted.amount, posted.home_amount);
        format!("{} {amount} {home}", posted.account)
    });
    written.collect()
}

fn written<T>(value: Option<T>, write: impl FnOnce(T) -> String) -> String {
    value.map_or_else(|| "-".to_owned(), write)
}

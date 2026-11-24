//! The links between transactions.

use crate::amount::Amount;
use crate::codes::LinkCode;
use crate::ids::{CurrencyId, LineId, TransactionId};
#[cfg(test)]
use crate::tables::Form;
#[cfg(test)]
use crate::tables::tests::Declared;
use crate::tables::{Table, stored_row};

pub(crate) const LINKS: Table<LinkRow> = Table::new("abmc_transaction_link");

stored_row! {
    /// A link between a line of a transaction and a line of another, as the
    /// first of the two has it. The other one has a row of its own.
    pub(crate) struct LinkRow {
        required {
            id: i32 = "link_id",
            transaction: TransactionId = "transaction_id",
            other: Option<TransactionId> = "transaction_2_id",
            line: Option<LineId> = "target_1_id",
            other_line: Option<LineId> = "target_2_id",
            code: LinkCode = "link_type",
            /// Whether the transaction is the one that pays, credits or was
            /// made from the other.
            is_source: bool = "is_t1_source_bool",
            /// Below zero for what is paid or credited, in the currency of
            /// both transactions.
            amount: Option<Amount> = "amount_amt",
        }
        optional {
            /// Whether another row says the same.
            is_secondary: bool = "is_secondary_bool",
            is_information_only: bool = "is_information_only_bool",
            quantity: Option<Amount> = "amount_qnty",
        }
    }
}

stored_row! {
    /// The currency of a transaction, as its header has it.
    pub(crate) struct HeaderCurrencyRow {
        required {}
        optional {
            /// Nothing in a company of one currency.
            currency: Option<CurrencyId> = "currency_id",
        }
    }
}

#[cfg(test)]
pub(super) fn declared() -> Vec<Box<dyn Declared>> {
    let headers = Form::all().map(|form| form.headers::<HeaderCurrencyRow>());
    let mut declared: Vec<Box<dyn Declared>> = vec![Box::new(LINKS)];
    declared.extend(headers.map(|table| Box::new(table) as Box<dyn Declared>));
    declared
}

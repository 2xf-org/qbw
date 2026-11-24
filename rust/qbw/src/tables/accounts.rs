//! The chart of accounts.

use chrono::NaiveDate;
use sa17::Decimal;

use crate::codes::{AccountType, SpecialAccount};
use crate::ids::{AccountId, CurrencyId};
#[cfg(test)]
use crate::tables::tests::Declared;
use crate::tables::{Table, stored_row};
use crate::unix_time::UnixTime;

pub(crate) const ACCOUNTS: Table<AccountRow> = Table::new("abmc_account_user");

pub(crate) const ACCOUNT_TOTALS: Table<AccountTotalsRow> = Table::new("abmc_account_internal");

stored_row! {
    /// An account as people keep it.
    pub(crate) struct AccountRow {
        required {
            id: AccountId = "account_id",
            name: Option<String> = "name",
            parent: Option<AccountId> = "parent_id",
            account_type: AccountType = "type",
            is_hidden: bool = "is_hidden_bool",
            /// A deleted account keeps its row, without a name.
            is_deleted: bool = "is_deleted_bool",
        }
        optional {
            number: Option<String> = "account_num",
            description: Option<String> = "description",
            /// Nothing in a company of one currency.
            currency: Option<CurrencyId> = "currency_id",
            tax_line: Option<i32> = "tax_line_id_num",
            created: Option<UnixTime> = "timestamp_tms",
            modified: Option<UnixTime> = "modtimestamp_tms",
        }
    }
}

stored_row! {
    /// What QuickBooks keeps about an account.
    pub(crate) struct AccountTotalsRow {
        required {
            id: AccountId = "account_id",
            special: Option<SpecialAccount> = "special_type",
            /// The id of the next account in the list, with a minus sign on
            /// the first one. The last has the id of the first.
            next: i32 = "display_index_next",
        }
        optional {
            balance: Option<Decimal> = "ending_balance_amt",
            home_balance: Option<Decimal> = "ending_balance_home_amt",
            statement_date: Option<NaiveDate> = "reconcile_statement_date",
            /// Nothing unless a reconciliation is under way.
            statement_balance: Option<Decimal> = "reconcile_statement_amt",
        }
    }
}

#[cfg(test)]
pub(super) fn declared() -> Vec<Box<dyn Declared>> {
    vec![Box::new(ACCOUNTS), Box::new(ACCOUNT_TOTALS)]
}

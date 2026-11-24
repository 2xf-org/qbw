//! The choices that the file stores as numbers.
//!
//! Each is an enum with the numbers' meanings, and the modules that use one
//! re-export it.

mod account_type;
mod channel_kind;
mod cleared;
mod contact_kind;
mod inventory_side;
mod item_kind;
mod job_status;
mod last_action;
mod link_code;
mod name_column;
mod payment_method_kind;
mod price_level_kind;
mod special_account;
mod special_item;
mod template_form;
mod terms_kind;
mod transaction_kind;

pub use account_type::AccountType;
pub use channel_kind::ChannelKind;
pub use cleared::Cleared;
pub use contact_kind::ContactKind;
pub(crate) use inventory_side::InventorySide;
pub use item_kind::ItemKind;
pub use job_status::JobStatus;
pub use last_action::LastAction;
pub use link_code::LinkCode;
pub(crate) use name_column::NameColumn;
pub use payment_method_kind::PaymentMethodKind;
pub use price_level_kind::PriceLevelKind;
pub use special_account::SpecialAccount;
pub use special_item::SpecialItem;
pub use template_form::TemplateForm;
pub use terms_kind::TermsKind;
pub use transaction_kind::TransactionKind;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::fmt::{Debug, Display};
    use std::hash::Hash;
    use std::str::FromStr;

    use super::*;
    use crate::code::UnknownWords;
    use crate::number::FromNumber;

    #[test]
    fn a_code_is_never_lost() {
        assert_eq!(AccountType::from_code(9), AccountType::Equity);
        assert_eq!(AccountType::from_code(99), AccountType::Other(99));
        for code in [0, 9, 15, 16, -5, i64::MAX] {
            assert_eq!(AccountType::from_code(code).code(), code);
        }
        assert_eq!(AccountType::from_number(0), Some(AccountType::Bank));
    }

    /// Checks the list and the words of an enum, of which `from_code` makes a
    /// choice.
    fn check<T>(all: &[T], from_code: fn(i64) -> T)
    where
        T: Copy + Debug + Display + Eq + Hash + FromStr<Err = UnknownWords>,
    {
        assert_eq!(all.iter().collect::<HashSet<_>>().len(), all.len());
        for number in -2..200 {
            let choice = from_code(number);
            let is_named = format!("{choice:?}") != format!("Other({number})");
            assert_eq!(all.contains(&choice), is_named, "{choice:?}");
            let words = choice.to_string();
            let parsed = [words.parse(), words.to_uppercase().parse()];
            assert_eq!(parsed, [Ok(choice), Ok(choice)], "{words}");
        }
        for words in ["", "Receipt", "99", "Transaction", "Bill 9"] {
            let error = words.parse::<T>().unwrap_err();
            assert_eq!(error.words, words);
        }
    }

    #[test]
    fn a_choice_is_what_it_prints() {
        check(TransactionKind::ALL, TransactionKind::from_code);
        check(AccountType::ALL, AccountType::from_code);
        check(ItemKind::ALL, ItemKind::from_code);
        check(Cleared::ALL, Cleared::from_code);
        let lengths = [
            TransactionKind::ALL.len(),
            AccountType::ALL.len(),
            ItemKind::ALL.len(),
            Cleared::ALL.len(),
        ];
        assert_eq!(lengths, [35, 16, 12, 4]);
    }

    #[test]
    fn what_stands_between_the_words_does_not_matter() {
        for words in ["Bill Pmt -Check", "bill-pmt-check", " BILL  PMT CHECK "] {
            assert_eq!(words.parse(), Ok(TransactionKind::BillPaymentCheck));
        }
        assert_eq!(
            "ytd adjust".parse(),
            Ok(TransactionKind::YearToDateAdjustment)
        );
        assert_eq!("non inventory part".parse(), Ok(ItemKind::NonInventoryPart));
        assert_eq!(
            "cost-of-goods-sold".parse(),
            Ok(AccountType::CostOfGoodsSold)
        );
        // Not one word of another choice's.
        assert!("BillPmtCheck".parse::<TransactionKind>().is_err());
        assert!("Bill Pmt".parse::<TransactionKind>().is_err());
    }

    #[test]
    fn a_choice_without_a_name_is_its_number() {
        assert_eq!("transaction 99 ".parse(), Ok(TransactionKind::Other(99)));
        assert_eq!(" Account Type  -5".parse(), Ok(AccountType::Other(-5)));
        let others = ["transaction 99x", "transaction", "account type 99"];
        for words in others {
            assert!(words.parse::<TransactionKind>().is_err(), "{words}");
        }
    }

    #[test]
    fn words_that_no_choice_has_are_said() {
        let error = "Receipt\n".parse::<TransactionKind>().unwrap_err();
        assert_eq!(error.what, "kind of transaction");
        let says = r#"there is no kind of transaction called "Receipt\n""#;
        assert_eq!(error.to_string(), says);
        let whats = [
            "?".parse::<AccountType>().unwrap_err().what,
            "?".parse::<ItemKind>().unwrap_err().what,
            "?".parse::<Cleared>().unwrap_err().what,
        ];
        let expected = [
            "type of account",
            "kind of item",
            "state of a line on a statement",
        ];
        assert_eq!(whats, expected);
    }
}

//! What a column's value reads as.

use chrono::NaiveDate;
use sa17::{Decimal, FromValue, Value, ValueProblem};

use crate::amount::Amount;
use crate::ids::{AddressId, ContactId, OwnerKey};
use crate::number::FromNumber;

/// A type that a declared column reads as.
///
/// An `Option` is `None` for `NULL`, which is also what a column reads as that
/// the file lacks. The file writes 0 for an id, a code or a moment that it
/// does not have, and an empty text for a text, so an `Option` of those is
/// `None` then, too.
pub(crate) trait Field: Sized {
    /// Reads `value`.
    ///
    /// # Errors
    ///
    /// Returns how the value differs from what the type takes.
    fn read(value: &Value) -> Result<Self, ValueProblem>;
}

impl<T: FromNumber> Field for T {
    fn read(value: &Value) -> Result<Self, ValueProblem> {
        let number = i64::from_value(value)?;
        T::from_number(number).ok_or(ValueProblem::Range { expected: "an id" })
    }
}

impl<T: FromNumber> Field for Option<T> {
    fn read(value: &Value) -> Result<Self, ValueProblem> {
        match Option::<i64>::from_value(value)? {
            None | Some(0) => Ok(None),
            Some(_) => T::read(value).map(Some),
        }
    }
}

impl Field for bool {
    /// A flag that the table lacks is not set.
    fn read(value: &Value) -> Result<Self, ValueProblem> {
        Option::<bool>::from_value(value).map(Option::unwrap_or_default)
    }
}

impl Field for Option<String> {
    fn read(value: &Value) -> Result<Self, ValueProblem> {
        let text = Option::<&str>::from_value(value)?;
        Ok(text.filter(|text| !text.is_empty()).map(str::to_owned))
    }
}

impl Field for Option<Amount> {
    fn read(value: &Value) -> Result<Self, ValueProblem> {
        let Some(number) = Option::<Decimal>::from_value(value)? else {
            return Ok(None);
        };
        let amount = Amount::new(number);
        let expected = "an amount of five digits after the point";
        amount.map(Some).ok_or(ValueProblem::Range { expected })
    }
}

/// A type that the database reads as it is.
macro_rules! plain_fields {
    ($($type:ty),+) => {$(
        impl Field for $type {
            fn read(value: &Value) -> Result<Self, ValueProblem> {
                <$type>::from_value(value)
            }
        }
    )+};
}

plain_fields!(
    i32,
    Option<i32>,
    String,
    f32,
    Option<f32>,
    NaiveDate,
    Option<NaiveDate>
);

/// Without zeros at the end of its fraction, so that a quantity prints as a person
/// writes it.
impl Field for Option<Decimal> {
    fn read(value: &Value) -> Result<Self, ValueProblem> {
        let number = <Self as FromValue>::from_value(value)?;
        Ok(number.map(Decimal::normalize))
    }
}

impl Field for Decimal {
    fn read(value: &Value) -> Result<Self, ValueProblem> {
        Self::from_value(value).map(Self::normalize)
    }
}

/// The keys that the file stores as text.
macro_rules! text_fields {
    ($($type:ty),+) => {$(
        impl Field for Option<$type> {
            fn read(value: &Value) -> Result<Self, ValueProblem> {
                let Some(text) = <Option<String> as Field>::read(value)? else {
                    return Ok(None);
                };
                let key = <$type>::try_new(&text);
                key.map(Some).ok_or(ValueProblem::Range { expected: "a key" })
            }
        }

        impl Field for $type {
            fn read(value: &Value) -> Result<Self, ValueProblem> {
                let key = <Option<$type> as Field>::read(value)?;
                key.ok_or(ValueProblem::Null { expected: "a key" })
            }
        }
    )+};
}

text_fields!(AddressId, ContactId, OwnerKey);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codes::AccountType;
    use crate::ids::AccountId;

    fn text(text: &str) -> Value {
        Value::Text(text.to_owned())
    }

    #[test]
    fn nothing_is_null_or_zero_for_a_key_and_a_code() {
        for nothing in [Value::Null, Value::Integer(0)] {
            assert_eq!(Option::<AccountId>::read(&nothing), Ok(None));
        }
        let id = Value::Integer(7);
        assert_eq!(Option::<AccountId>::read(&id), Ok(Some(AccountId::new(7))));
        // A code that a row always has may be 0.
        assert_eq!(AccountType::read(&Value::Integer(0)), Ok(AccountType::Bank));
        assert_eq!(
            AccountId::read(&Value::Null),
            Err(ValueProblem::Null { expected: "i64" })
        );
        assert_eq!(
            AccountId::read(&Value::Integer(i128::from(u32::MAX))),
            Err(ValueProblem::Range { expected: "an id" })
        );
    }

    #[test]
    fn nothing_is_null_or_empty_for_a_text() {
        for nothing in [Value::Null, text("")] {
            assert_eq!(<Option<String> as Field>::read(&nothing), Ok(None));
            assert_eq!(Option::<OwnerKey>::read(&nothing), Ok(None));
        }
        let key = OwnerKey::try_new("0000000000000001");
        assert_eq!(Option::<OwnerKey>::read(&text("0000000000000001")), Ok(key));
        assert!(Option::<OwnerKey>::read(&text("000000000000000001")).is_err());
        // A key that a row always has is not empty.
        assert!(OwnerKey::read(&text("")).is_err());
    }

    #[test]
    fn a_number_keeps_its_zero_and_a_flag_may_be_missing() {
        assert_eq!(Option::<i32>::read(&Value::Integer(0)), Ok(Some(0)));
        assert_eq!(bool::read(&Value::Null), Ok(false));
        assert_eq!(bool::read(&Value::Boolean(true)), Ok(true));
        assert!(bool::read(&Value::Integer(1)).is_err());
    }
}

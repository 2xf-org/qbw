//! Who a transaction or a line is with.

use crate::codes::NameColumn;
use crate::ids::NameId;

/// The four columns that headers and lines have for a name.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FourNames {
    pub(crate) customer: Option<NameId>,
    pub(crate) vendor: Option<NameId>,
    pub(crate) employee: Option<NameId>,
    pub(crate) other_name: Option<NameId>,
}

/// A name, and which list it is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Party {
    pub(crate) list: NameColumn,
    pub(crate) name: NameId,
}

impl FourNames {
    /// The name of a transaction. Its header has one of the four at most, which
    /// a line without a name of its own repeats.
    pub(crate) fn of_transaction(self) -> Option<Party> {
        let lists = [
            NameColumn::Customer,
            NameColumn::Vendor,
            NameColumn::Employee,
            NameColumn::OtherName,
        ];
        lists.into_iter().find_map(|list| self.party(list))
    }

    /// The name of a line that is its own, when `column` says which of the four
    /// is. The others repeat the transaction's.
    pub(crate) fn of_line(self, column: Option<NameColumn>) -> Option<Party> {
        self.party(column?)
    }

    /// The name that a line posts with: its own, or else the transaction's.
    pub(crate) fn of_posting(self, column: Option<NameColumn>) -> Option<Party> {
        match column {
            Some(NameColumn::Transaction) | None => self.of_transaction(),
            Some(list) => self.party(list),
        }
    }

    fn party(self, list: NameColumn) -> Option<Party> {
        let name = match list {
            NameColumn::Customer => self.customer,
            NameColumn::Vendor => self.vendor,
            NameColumn::Employee => self.employee,
            NameColumn::OtherName => self.other_name,
            NameColumn::Transaction => None,
        };
        name.map(|name| Party { list, name })
    }
}

impl Party {
    /// The name, when it is in `list`.
    pub(crate) fn in_list(self, list: NameColumn) -> Option<NameId> {
        (self.list == list).then_some(self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_has_a_name_of_its_own_or_repeats_the_transaction_s() {
        let names = FourNames {
            customer: Some(NameId::new(1)),
            vendor: Some(NameId::new(2)),
            ..FourNames::default()
        };
        let name = |party: Option<Party>| party.map(|party| party.name.number());
        assert_eq!(name(names.of_line(Some(NameColumn::Vendor))), Some(2));
        assert_eq!(name(names.of_line(Some(NameColumn::Transaction))), None);
        assert_eq!(name(names.of_line(None)), None);
        assert_eq!(name(names.of_transaction()), Some(1));
        assert_eq!(name(names.of_posting(None)), Some(1));
        assert_eq!(name(names.of_posting(Some(NameColumn::Vendor))), Some(2));
        assert_eq!(name(names.of_posting(Some(NameColumn::Employee))), None);
        assert_eq!(
            names.of_transaction().unwrap().in_list(NameColumn::Vendor),
            None
        );
    }
}

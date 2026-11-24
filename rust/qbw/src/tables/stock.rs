//! What there is of items in stock, and what lines say about items.

use chrono::NaiveDate;
use sa17::Source;

use crate::amount::Amount;
use crate::codes::TransactionKind;
use crate::error::Result;
use crate::ids::{ItemId, LineId, SiteId, TransactionId};
use crate::posting_state::PostingFlags;
#[cfg(test)]
use crate::tables::tests::Declared;
use crate::tables::{Form, OpenTable, Table, stored_row, table};

pub(crate) const HISTORY: Table<HistoryRow> = Table::new("abmc_item_history");

/// The column that a line has its item in.
const ITEM: &str = "item_id";

/// The lines in `source` of `forms` that can have an item, form by form.
///
/// # Errors
///
/// Returns an error when the source lacks a table that every file has, or a
/// column of one.
pub(crate) fn item_lines(
    source: &dyn Source,
    forms: impl Iterator<Item = Form>,
) -> Result<Vec<OpenTable<'_, ItemLineRow>>> {
    let mut tables = Vec::new();
    for form in forms {
        let definition = table::definition(source, form.lines::<ItemLineRow>().name())?;
        if definition.is_none_or(|definition| definition.position(ITEM).is_some()) {
            tables.extend(form.lines_in(source)?);
        }
    }
    Ok(tables)
}

stored_row! {
    /// What there is of an item after one of its lines.
    ///
    /// Lines of orders and of what is pending have a row, too, which repeats
    /// the figures of the row before it.
    pub(crate) struct HistoryRow {
        required {
            item: ItemId = "item_id",
            /// The place among the rows of the item, from 0 on.
            sequence: i32 = "sequence_number",
            date: NaiveDate = "transaction_date",
            /// Nothing on the first row of an item, which is its start.
            transaction: Option<TransactionId> = "transaction_id",
            line: Option<LineId> = "target_id",
            on_hand: Option<Amount> = "quantity_on_hand",
            value: Option<Amount> = "value_on_hand",
            average_cost: Option<Amount> = "average_cost",
        }
        optional {}
    }
}

stored_row! {
    /// What a line says about its item.
    pub(crate) struct ItemLineRow {
        required {
            id: LineId = "target_id",
            transaction: TransactionId = "transaction_id",
            kind: TransactionKind = "transaction_view_type",
            is_own: bool = "is_source_bool",
        }
        optional {
            /// Nothing on a line of no item.
            item: Option<ItemId> = "item_id",
            /// How much of it goes into the stock, with a minus sign where it
            /// goes out. Nothing when nobody said how much.
            quantity: Option<Amount> = "quantity_qnty",
            is_non_posting: bool = "is_no_post_bool",
            is_memorized: bool = "is_memorized_transaction_bool",
            /// What is left of an order's quantity. Nothing when it is the
            /// whole quantity.
            open_quantity: Option<Amount> = "open_qnty",
            is_received: bool = "is_received_bool",
            is_invoiced: bool = "is_invoiced_bool",
            site: Option<SiteId> = "site_id",
            bin: Option<SiteId> = "rsb_id",
        }
    }
}

impl ItemLineRow {
    /// What the line says of whether its transaction counts.
    pub(crate) fn flags(&self) -> PostingFlags {
        PostingFlags {
            kind: self.kind,
            is_memorized: self.is_memorized,
            is_non_posting: self.is_non_posting,
        }
    }
}

#[cfg(test)]
pub(super) fn declared() -> Vec<Box<dyn Declared>> {
    let mut declared: Vec<Box<dyn Declared>> = vec![Box::new(HISTORY)];
    for form in Form::all() {
        declared.push(Box::new(form.lines::<ItemLineRow>()));
    }
    declared
}

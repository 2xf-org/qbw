//! How many of its item a line is of.

use crate::amount::Amount;
use crate::tables::stock::ItemLineRow;
use crate::transactions::{FormSide, OrderLine};

/// How many of its item a line puts into the stock, below zero for how many it
/// takes out. A line that does not say how many is of one.
pub(super) fn moved(line: &ItemLineRow) -> Amount {
    line.quantity.unwrap_or_else(|| {
        if FormSide::of(line.kind).takes_out(line.is_own) {
            -Amount::ONE
        } else {
            Amount::ONE
        }
    })
}

/// How many of its item a line of an order asks for.
pub(super) fn ordered(line: &ItemLineRow) -> Amount {
    order_line(line).ordered()
}

/// How many of them are still to come or to go, unless the line is done or of no
/// order.
pub(super) fn open(line: &ItemLineRow) -> Option<Amount> {
    let line = order_line(line);
    let is_done = line.is_done()?;
    (!is_done).then(|| line.open())
}

fn order_line(line: &ItemLineRow) -> OrderLine {
    OrderLine {
        kind: line.kind,
        quantity: line.quantity,
        open_quantity: line.open_quantity,
        is_received: line.is_received,
        is_invoiced: line.is_invoiced,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codes::TransactionKind;
    use crate::ids::{LineId, TransactionId};

    fn amount(text: &str) -> Amount {
        Amount::new(text.parse().unwrap()).unwrap()
    }

    fn line(kind: TransactionKind, quantity: Option<&str>) -> ItemLineRow {
        ItemLineRow {
            id: LineId::new(2),
            transaction: TransactionId::new(1),
            kind,
            is_own: false,
            item: None,
            quantity: quantity.map(amount),
            is_non_posting: false,
            is_memorized: false,
            open_quantity: None,
            is_received: false,
            is_invoiced: false,
            site: None,
            bin: None,
        }
    }

    #[test]
    fn a_line_without_a_quantity_is_of_one_in_its_direction() {
        let of = |kind| moved(&line(kind, None));
        assert_eq!(of(TransactionKind::Invoice), amount("-1"));
        assert_eq!(of(TransactionKind::CreditMemo), amount("1"));
        assert_eq!(of(TransactionKind::Bill), amount("1"));
        assert_eq!(of(TransactionKind::VendorCredit), amount("-1"));
        assert_eq!(of(TransactionKind::CreditCardCredit), amount("-1"));
        let said = line(TransactionKind::Invoice, Some("2.5"));
        assert_eq!(moved(&said), amount("2.5"));
    }

    #[test]
    fn an_order_shows_what_it_asks_for() {
        let sales = line(TransactionKind::SalesOrder, Some("-4"));
        assert_eq!(ordered(&sales), amount("4"));
        let purchase = line(TransactionKind::PurchaseOrder, Some("5"));
        assert_eq!(ordered(&purchase), amount("5"));
        let one = line(TransactionKind::SalesOrder, None);
        assert_eq!(
            (ordered(&one), open(&one)),
            (amount("1"), Some(amount("1")))
        );
    }

    #[test]
    fn what_is_open_is_the_whole_quantity_unless_it_says_less() {
        let mut sales = line(TransactionKind::SalesOrder, Some("-4"));
        assert_eq!(open(&sales), Some(amount("4")));
        sales.open_quantity = Some(amount("-1.5"));
        assert_eq!(open(&sales), Some(amount("1.5")));
        sales.is_received = true;
        assert_eq!(open(&sales), Some(amount("1.5")));
        sales.is_invoiced = true;
        assert_eq!(open(&sales), None);
        let mut purchase = line(TransactionKind::PurchaseOrder, Some("5"));
        purchase.is_invoiced = true;
        assert_eq!(open(&purchase), Some(amount("5")));
        purchase.is_received = true;
        assert_eq!(open(&purchase), None);
        assert_eq!(open(&line(TransactionKind::Invoice, Some("-1"))), None);
    }
}

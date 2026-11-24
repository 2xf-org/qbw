//! Which tables hold which transactions.

use sa17::Source;

use crate::codes::TransactionKind;

/// The tables of one form of transaction: a row for each transaction, its
/// lines, and what lines of items in stock post besides.
pub(super) struct TransactionTables {
    /// What decides the kind of a transaction.
    pub(super) kind: KindSource,
    pub(super) headers: &'static str,
    pub(super) lines: &'static str,
    pub(super) stock_lines: Option<&'static str>,
    /// Whether they are of payroll, which a file may lack.
    pub(super) is_of_payroll: bool,
}

/// What decides the kind of a transaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum KindSource {
    /// The table: it holds one kind. A header may have a code all the same,
    /// and it is not to be read.
    Table(TransactionKind),
    /// The header's code, which is of one of these kinds in every file that
    /// was looked at.
    Header(&'static [TransactionKind]),
}

impl TransactionTables {
    const fn of(kind: TransactionKind, headers: &'static str, lines: &'static str) -> Self {
        Self {
            kind: KindSource::Table(kind),
            headers,
            lines,
            stock_lines: None,
            is_of_payroll: false,
        }
    }

    const fn of_several(
        kinds: &'static [TransactionKind],
        headers: &'static str,
        lines: &'static str,
    ) -> Self {
        Self {
            kind: KindSource::Header(kinds),
            headers,
            lines,
            stock_lines: None,
            is_of_payroll: false,
        }
    }

    const fn with_stock(self, stock_lines: &'static str) -> Self {
        Self {
            stock_lines: Some(stock_lines),
            ..self
        }
    }

    const fn of_payroll(self) -> Self {
        Self {
            is_of_payroll: true,
            ..self
        }
    }

    /// Whether transactions of `kind` are known to be in the tables.
    pub(super) fn is_of(&self, kind: TransactionKind) -> bool {
        match self.kind {
            KindSource::Table(own) => own == kind,
            KindSource::Header(kinds) => kinds.contains(&kind),
        }
    }

    /// Whether the table of lines called `name` is one of these.
    fn holds(&self, name: &str) -> bool {
        self.lines == name || self.stock_lines == Some(name)
    }
}

/// The tables of every form of transaction.
///
/// The last five were empty in every file that this crate was checked against.
pub(super) const ALL: &[TransactionTables] = &[
    TransactionTables::of(
        TransactionKind::BillPaymentCreditCard,
        "abmc_bill_ccard_header",
        "abmc_bill_ccard_lineitem",
    ),
    TransactionTables::of(
        TransactionKind::BillPaymentCheck,
        "abmc_bill_check_header",
        "abmc_bill_check_lineitem",
    ),
    TransactionTables::of_several(
        &[
            TransactionKind::Bill,
            TransactionKind::VendorCredit,
            TransactionKind::ItemReceipt,
        ],
        "abmc_bill_header",
        "abmc_bill_lineitem",
    )
    .with_stock("abmc_bill_inventory_lineitem"),
    TransactionTables::of_several(
        &[TransactionKind::Check, TransactionKind::SalesTaxPayment],
        "abmc_check_header",
        "abmc_check_lineitem",
    )
    .with_stock("abmc_check_inventory_lineitem"),
    TransactionTables::of_several(
        &[
            TransactionKind::CreditCardCharge,
            TransactionKind::CreditCardCredit,
        ],
        "abmc_credit_cardcharge_header",
        "abmc_credit_cardcharge_lineitem",
    )
    .with_stock("abmc_credit_cardcharge_inventory_lineitem"),
    TransactionTables::of(
        TransactionKind::CreditMemo,
        "abmc_credit_memo_header",
        "abmc_credit_memo_lineitem",
    )
    .with_stock("abmc_credit_memo_inventory_lineitem"),
    TransactionTables::of(
        TransactionKind::Deposit,
        "abmc_deposit_header",
        "abmc_deposit_lineitem",
    ),
    TransactionTables::of(
        TransactionKind::Estimate,
        "abmc_estimate_header",
        "abmc_estimate_lineitem",
    ),
    TransactionTables::of(
        TransactionKind::JournalEntry,
        "abmc_general_journal_header",
        "abmc_general_journal_lineitem",
    )
    .with_stock("abmc_general_journal_inventory_lineitem"),
    TransactionTables::of(
        TransactionKind::InventoryAdjustment,
        "abmc_inventory_adjust_header",
        "abmc_inventory_adjust_lineitem",
    )
    .with_stock("abmc_inventory_adjust_inventory_lineitem"),
    TransactionTables::of(
        TransactionKind::InventoryTransfer,
        "abmc_inventory_transfer_header",
        "abmc_inventory_transfer_lineitem",
    ),
    TransactionTables::of(
        TransactionKind::Invoice,
        "abmc_invoice_header",
        "abmc_invoice_lineitem",
    )
    .with_stock("abmc_invoice_inventory_lineitem"),
    TransactionTables::of(
        TransactionKind::AssemblyBuild,
        "abmc_item_assembly_header",
        "abmc_item_assembly_lineitem",
    )
    .with_stock("abmc_item_assembly_inventory_lineitem"),
    TransactionTables::of(
        TransactionKind::PurchaseOrder,
        "abmc_purchase_order_header",
        "abmc_purchase_order_lineitem",
    ),
    TransactionTables::of(
        TransactionKind::Payment,
        "abmc_received_payment_header",
        "abmc_received_payment_lineitem",
    ),
    TransactionTables::of(
        TransactionKind::SalesOrder,
        "abmc_sales_order_header",
        "abmc_sales_order_lineitem",
    ),
    TransactionTables::of(
        TransactionKind::StatementCharge,
        "abmc_statement_charge_header",
        "abmc_statement_charge_lineitem",
    )
    .with_stock("abmc_statement_charge_inventory_lineitem"),
    TransactionTables::of(
        TransactionKind::Transfer,
        "abmc_transfer_header",
        "abmc_transfer_lineitem",
    ),
    TransactionTables::of(
        TransactionKind::CustomerRefund,
        "abmc_customer_refund_header",
        "abmc_customer_refund_lineitem",
    ),
    // The table of its stock spells the name in another way.
    TransactionTables::of(
        TransactionKind::SalesReceipt,
        "abmc_sales_receipt_header",
        "abmc_sales_receipt_lineitem",
    )
    .with_stock("abmc_sale_receipt_inventory_lineitem"),
    TransactionTables::of_several(
        &[
            TransactionKind::YearToDateAdjustment,
            TransactionKind::LiabilityAdjustment,
        ],
        "es_liability_adjust_header",
        "es_liability_adjust_lineitem",
    )
    .of_payroll(),
    TransactionTables::of_several(
        &[TransactionKind::PayrollLiabilityCheck],
        "es_liability_payment_header",
        "es_liability_payment_lineitem",
    )
    .of_payroll(),
    TransactionTables::of_several(
        &[TransactionKind::Paycheck],
        "es_paycheck_header",
        "es_paycheck_lineitem",
    )
    .of_payroll(),
];

/// How many tables of lines `source` has that post something this version does
/// not read: ones with rows, with an account and an amount, that are neither
/// above nor the copies of lines that the audit trail keeps.
pub(crate) fn unread(source: &dyn Source) -> u32 {
    let unread = source.definitions().iter().filter(|definition| {
        let name = definition.name();
        let is_read = ALL.iter().any(|tables| tables.holds(name));
        name.ends_with("_lineitem")
            && !name.starts_with("abmc_audit_")
            && !is_read
            && definition.row_count() > 0
            && definition.position("account_id").is_some()
            && definition.position("amount_amt").is_some()
    });
    unread.map(|_| 1).sum()
}

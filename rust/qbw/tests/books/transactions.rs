//! A transaction of every kind that the books have, as its form shows it.

use qbw::transactions::{Cleared, Details, LastAction, OpenState, PostingState, TransactionKind};
use qbw::{
    AccountId, AddressId, ClassId, ContactId, CurrencyId, ItemId, LineId, MessageId, NameId,
    OtherSide, OwnerKey, PaymentMethodId, SalesRepId, SalesTaxCodeId, ShipMethodId, TemplateId,
    TermsId, TransactionId, UnixTime, UserId,
};

use crate::{forms, support};

/// A transaction as its form shows it: its kind, its total, the amount of its
/// own side with its account, and its lines.
struct Shown {
    id: i32,
    kind: TransactionKind,
    total: &'static str,
    own: (i32, &'static str),
    lines: &'static [&'static str],
}

const ALL: &[Shown] = &[
    Shown {
        id: 100,
        kind: TransactionKind::JournalEntry,
        total: "-",
        own: (1, "1000.00 (1)"),
        lines: &["Account 1 - - - 1000.00", "Account 10 - - - -1000.00"],
    },
    Shown {
        id: 200,
        kind: TransactionKind::Bill,
        total: "200.00 (1)",
        own: (7, "-200.00 (1)"),
        lines: &["Item 6 502 10 20.00 200.00"],
    },
    Shown {
        id: 300,
        kind: TransactionKind::BillPaymentCheck,
        total: "200.00 (1)",
        own: (1, "-200.00 (1)"),
        lines: &["Account 7 - - - 200.00"],
    },
    Shown {
        id: 400,
        kind: TransactionKind::Invoice,
        total: "202.00 (1)",
        own: (3, "202.00 (1)"),
        lines: &[
            "Item 13 502 3 50.00 150.00",
            "Item 12 501 1 40.00 40.00",
            "Text - - - - -",
            "SalesTax 9 503 - 8% 12.00",
        ],
    },
    Shown {
        id: 500,
        kind: TransactionKind::Payment,
        total: "100.00 (1)",
        own: (5, "100.00 (1)"),
        lines: &[
            "Account 3 - - - 100.00",
            "DiscountTaken 3 - - - 2.00",
            "DiscountAccount 18 - - - -2.00",
        ],
    },
    Shown {
        id: 600,
        kind: TransactionKind::Deposit,
        total: "100.00 (1)",
        own: (1, "100.00 (1)"),
        lines: &["Account 5 - - - 100.00"],
    },
    Shown {
        id: 700,
        kind: TransactionKind::CreditMemo,
        total: "54.00 (1)",
        own: (3, "-54.00 (1)"),
        lines: &["Item 13 502 1 50.00 50.00", "SalesTax 9 503 - 8% 4.00"],
    },
    Shown {
        id: 800,
        kind: TransactionKind::Invoice,
        total: "100.00 (2)",
        own: (4, "100.00 (2)"),
        lines: &["Item 12 501 1 100.00 100.00"],
    },
    Shown {
        id: 900,
        kind: TransactionKind::Payment,
        total: "100.00 (2)",
        own: (2, "100.00 (2)"),
        lines: &["Account 4 - - - 100.00", "ExchangeDifference 17 - - - -"],
    },
    Shown {
        id: 1000,
        kind: TransactionKind::CreditCardCharge,
        total: "30.00 (1)",
        own: (8, "-30.00 (1)"),
        lines: &["Account 16 - - - 30.00"],
    },
    Shown {
        id: 1100,
        kind: TransactionKind::CreditCardCredit,
        total: "5.00 (1)",
        own: (8, "5.00 (1)"),
        lines: &["Account 16 - - - 5.00"],
    },
    Shown {
        id: 1200,
        kind: TransactionKind::VendorCredit,
        total: "20.00 (1)",
        own: (7, "20.00 (1)"),
        lines: &["Item 6 502 1 20.00 20.00"],
    },
    Shown {
        id: 1300,
        kind: TransactionKind::Transfer,
        total: "55.00 (1)",
        own: (1, "-55.00 (1)"),
        lines: &["Account 2 - - - 55.00"],
    },
    Shown {
        id: 1400,
        kind: TransactionKind::PurchaseOrder,
        total: "100.00 (1)",
        own: (19, "-100.00 (1)"),
        lines: &["Item 6 502 5 20.00 100.00"],
    },
    Shown {
        id: 1500,
        kind: TransactionKind::ItemReceipt,
        total: "40.00 (1)",
        own: (7, "-40.00 (1)"),
        lines: &["Item 6 502 2 20.00 40.00"],
    },
    Shown {
        id: 1600,
        kind: TransactionKind::SalesOrder,
        total: "200.00 (1)",
        own: (20, "200.00 (1)"),
        lines: &["Item 13 502 4 50.00 200.00"],
    },
    Shown {
        id: 1700,
        kind: TransactionKind::Check,
        total: "0.00 (1)",
        own: (1, "0.00 (1)"),
        lines: &["Account 15 - - - 0.00"],
    },
    Shown {
        id: 1800,
        kind: TransactionKind::Invoice,
        total: "50.00 (1)",
        own: (3, "50.00 (1)"),
        lines: &["Item 13 502 1 50.00 50.00"],
    },
    Shown {
        id: 1900,
        kind: TransactionKind::Check,
        total: "15.00 (1)",
        own: (1, "-15.00 (1)"),
        lines: &["Account 15 - - - 15.00"],
    },
    Shown {
        id: 2000,
        kind: TransactionKind::Estimate,
        total: "129.00 (1)",
        own: (21, "129.00 (1)"),
        lines: &[
            "GroupStart - 505 1 - -",
            "Item 13 502 2 50.00 100.00",
            "Item 12 501 1 40.00 40.00",
            "GroupEnd - 505 - - 140.00",
            "Subtotal - 504 - - 140.00",
            "Item 18 506 - -11.00 -11.00",
        ],
    },
    Shown {
        id: 2100,
        kind: TransactionKind::InventoryAdjustment,
        total: "-",
        own: (15, "-20.00 (1)"),
        lines: &["Item 6 502 1 - 20.00"],
    },
    Shown {
        id: 2200,
        kind: TransactionKind::AssemblyBuild,
        total: "40.00 (1)",
        own: (6, "40.00 (1)"),
        lines: &["Item 6 502 2 - 40.00"],
    },
    Shown {
        id: 2300,
        kind: TransactionKind::SalesTaxPayment,
        total: "8.00 (1)",
        own: (1, "-8.00 (1)"),
        lines: &["Item 9 503 - - 8.00"],
    },
    // A discount is all of it, so its own side is one of its lines.
    Shown {
        id: 2400,
        kind: TransactionKind::Payment,
        total: "0.00 (1)",
        own: (3, "-1.00 (1)"),
        lines: &[
            "DiscountTaken 3 - - - 1.00",
            "Account 3 - - - 0.00",
            "DiscountAccount 18 - - - -1.00",
        ],
    },
    Shown {
        id: 2500,
        kind: TransactionKind::JournalEntry,
        total: "-",
        own: (2, "5.00 (1)"),
        lines: &["Account 2 - - - 5.00", "Account 17 - - - -5.00"],
    },
];

#[test]
fn every_form_shows_its_total_and_its_lines() {
    let company = support::books();
    for shown in ALL {
        let read = forms::transaction(&company, shown.id);
        let of = format!("transaction {}", shown.id);
        assert_eq!(read.kind, shown.kind, "{of}");
        assert_eq!(forms::money(read.total), shown.total, "{of}");
        let own = read.own_line.as_ref().unwrap();
        let (account, amount) = shown.own;
        assert_eq!(own.account, Some(AccountId::new(account)), "{of}");
        assert_eq!(forms::money(own.amount), amount, "{of}");
        assert_eq!(forms::lines(&read), shown.lines, "{of}");
        assert!(read.left_out().is_nothing(), "{of}");
    }
}

#[test]
fn the_lines_of_a_form_add_up_to_its_total() {
    let company = support::books();
    let mut with_total = 0;
    for shown in ALL {
        let read = forms::transaction(&company, shown.id);
        let Some(total) = read.total else {
            continue;
        };
        let adding = read.lines.iter().filter(|line| line.role.adds_to_total());
        let amounts = adding.filter_map(|line| line.amount);
        let sum = amounts.reduce(|sum, amount| sum.checked_add(amount).unwrap());
        assert_eq!(sum, Some(total), "transaction {}", shown.id);
        with_total += 1;
    }
    assert_eq!(with_total, 22);
}

#[test]
fn an_invoice_has_what_its_form_shows() {
    let company = support::books();
    let invoice = forms::transaction(&company, 400);
    assert_eq!(invoice.id, TransactionId::new(400));
    assert_eq!(invoice.date.to_string(), "2023-11-15");
    assert_eq!(invoice.number.as_deref(), Some("1041"));
    assert_eq!(invoice.name, Some(NameId::new(101)));
    assert_eq!(invoice.customer(), Some(NameId::new(101)));
    assert_eq!(invoice.vendor(), None);
    assert_eq!(invoice.memo.as_deref(), Some("Spring order"));
    assert_eq!(invoice.currency, CurrencyId::new(1));
    assert_eq!(forms::money(invoice.home_total), "202.00 (1)");
    assert_eq!(invoice.posting, PostingState::Posted);
    assert_eq!(invoice.last_action, LastAction::Entered);
    assert_eq!(invoice.billing, ContactId::try_new("CONTACT000000100"));
    assert_eq!(invoice.created, Some(UnixTime::new(1_700_056_800)));
    assert_eq!(invoice.modified, Some(UnixTime::new(1_700_060_400)));
    assert_eq!(invoice.edited_by, Some(UserId::new(1)));
    assert_eq!(invoice.sequence, Some(4));
    assert_eq!(invoice.owner_key, OwnerKey::try_new("TXN0000000000400"));
    let row = company.stored_row(invoice.origin).unwrap().unwrap();
    assert_eq!(row.get("trans_num").unwrap().as_i64(), Some(4));

    let sale = invoice.sale().unwrap();
    assert_eq!(sale.terms, Some(TermsId::new(1)));
    assert_eq!(invoice.due_date.unwrap().to_string(), "2023-12-15");
    assert_eq!(sale.ship_to, AddressId::try_new("SHIPTO0000000400"));
    assert_eq!(sale.ship_date.unwrap().to_string(), "2023-11-15");
    assert_eq!(sale.ship_via, Some(ShipMethodId::new(1)));
    assert_eq!(sale.sales_rep, Some(SalesRepId::new(1)));
    assert_eq!(sale.po_number.as_deref(), Some("PO-77"));
    assert_eq!((sale.fob.as_deref(), sale.other.as_deref()), (None, None));
    assert_eq!(sale.message, Some(MessageId::new(1)));
    assert_eq!(sale.tax_code, Some(SalesTaxCodeId::new(1)));
    assert_eq!(sale.template, Some(TemplateId::new(1)));
    assert_eq!(sale.sales_order_number.as_deref(), Some("S-4"));
    assert!(sale.email_later && sale.is_active);
    assert!(!sale.print_later && !sale.is_finance_charge);
    assert_eq!(invoice.terms(), sale.terms);
}

#[test]
fn a_line_has_what_its_form_shows() {
    let company = support::books();
    let invoice = forms::transaction(&company, 400);
    let [valves, _, text, tax] = invoice.lines.as_slice() else {
        panic!("invoice 1041 has four lines");
    };
    assert_eq!(valves.id, LineId::new(402));
    assert_eq!(valves.description.as_deref(), Some("Brass valve"));
    assert_eq!(valves.tax_code, Some(SalesTaxCodeId::new(1)));
    assert_eq!(valves.service_date.unwrap().to_string(), "2023-11-14");
    assert_eq!(valves.part_number.as_deref(), Some("CV-20"));
    assert_eq!(valves.other1.as_deref(), Some("Shelf 4"));
    assert_eq!((valves.other2.as_deref(), valves.markup), (None, None));
    assert_eq!(valves.owner_key, OwnerKey::try_new("LINE000000000402"));
    assert_eq!(valves.reimbursed_code, Some(0));
    assert_eq!((valves.class, valves.order), (None::<ClassId>, None));
    // It repeats the invoice's customer, and the tax is owed to an agency.
    assert_eq!((valves.name, tax.name), (None, Some(NameId::new(202))));
    assert_eq!(text.description.as_deref(), Some("Delivered to dock 1"));
    assert_eq!((text.home_amount, text.posted()), (None, None));
    let row = company.stored_row(valves.origin).unwrap().unwrap();
    assert_eq!(row.get("target_id").unwrap().as_i64(), Some(402));

    let deposit = forms::transaction(&company, 600);
    let [check] = deposit.lines.as_slice() else {
        panic!("the deposit has one line");
    };
    assert_eq!(check.name, Some(NameId::new(101)));
    assert_eq!(check.number.as_deref(), Some("88"));
    assert_eq!(check.payment_method, Some(PaymentMethodId::new(1)));
}

#[test]
fn an_invoice_knows_its_tax() {
    let company = support::books();
    let invoice = forms::transaction(&company, 400);
    assert_eq!(forms::money(invoice.subtotal()), "190.00 (1)");
    assert_eq!(forms::money(invoice.tax_total()), "12.00 (1)");
    assert_eq!(invoice.tax_item(), Some(ItemId::new(503)));
    assert_eq!(invoice.tax_percent().unwrap().to_string(), "8");
    let untaxed = forms::transaction(&company, 800);
    assert_eq!(forms::money(untaxed.subtotal()), "100.00 (2)");
    assert_eq!(forms::money(untaxed.tax_total()), "0.00 (2)");
    assert_eq!((untaxed.tax_item(), untaxed.tax_percent()), (None, None));
    // A discount is part of the subtotal, and a bill has no such thing.
    let estimate = forms::transaction(&company, 2000);
    assert_eq!(forms::money(estimate.subtotal()), "129.00 (1)");
    let bill = forms::transaction(&company, 200);
    assert_eq!((bill.subtotal(), bill.tax_total()), (None, None));
}

#[test]
fn the_own_side_has_what_a_register_shows() {
    let company = support::books();
    let journal = forms::transaction(&company, 100).own_line.unwrap();
    assert_eq!(journal.id, LineId::new(101));
    assert_eq!(journal.cleared, Cleared::Reconciled);
    assert_eq!(journal.reconciled_on.unwrap().to_string(), "2024-05-31");
    assert_eq!(journal.other_side, OtherSide::Account(AccountId::new(10)));
    let row = company.stored_row(journal.origin).unwrap().unwrap();
    assert_eq!(row.get("target_id").unwrap().as_i64(), Some(101));

    let charge = forms::transaction(&company, 1000).own_line.unwrap();
    assert_eq!(charge.cleared, Cleared::MatchedByBankFeed);
    let invoice = forms::transaction(&company, 400).own_line.unwrap();
    assert_eq!(invoice.other_side, OtherSide::Split);
    let estimate = forms::transaction(&company, 2000);
    assert_eq!(estimate.account(), Some(AccountId::new(21)));
}

#[test]
fn each_kind_has_details_of_its_own() {
    let company = support::books();
    let details = |id| forms::transaction(&company, id).details;
    let due_date = |id| {
        let due_date = forms::transaction(&company, id).due_date;
        due_date.unwrap().to_string()
    };

    let Details::Payment(payment) = details(500) else {
        panic!("a payment has the details of one");
    };
    assert_eq!(payment.payment_method, Some(PaymentMethodId::new(1)));
    assert!(payment.is_deposited && payment.moved_money && !payment.is_bounced);
    let Details::Bill(bill) = details(200) else {
        panic!("a bill has the details of one");
    };
    assert_eq!(bill.terms, Some(TermsId::new(1)));
    assert_eq!(due_date(200), "2023-06-09");
    assert!(matches!(details(1200), Details::Bill(_)));
    assert!(matches!(details(1500), Details::Bill(_)));
    let Details::PurchaseOrder(order) = details(1400) else {
        panic!("a purchase order has the details of one");
    };
    assert_eq!(order.terms, Some(TermsId::new(1)));
    assert_eq!(due_date(1400), "2024-05-05");
    assert_eq!(order.expected.unwrap().to_string(), "2024-04-19");
    let message = order.vendor_message.as_deref();
    assert_eq!(message, Some("Deliver in the morning."));
    assert_eq!(order.template, Some(TemplateId::new(1)));
    let Details::Transfer(transfer) = details(1300) else {
        panic!("a transfer has the details of one");
    };
    assert_eq!(transfer.from, Some(AccountId::new(1)));
    assert_eq!(transfer.to, Some(AccountId::new(2)));
    let Details::Build(build) = details(2200) else {
        panic!("a build has the details of one");
    };
    assert_eq!(build.item, Some(ItemId::new(507)));
    assert_eq!(build.quantity, Some(1.into()));
    let Details::BillPayment(payment) = details(300) else {
        panic!("a payment of bills has the details of one");
    };
    assert!(!payment.print_later);
    let Details::Check(check) = details(2300) else {
        panic!("a payment of sales tax has the details of a check");
    };
    assert!(check.print_later);
    assert!(matches!(details(1700), Details::Check(check) if !check.print_later));
    assert!(matches!(details(2100), Details::InventoryAdjustment(_)));
    assert_eq!(details(600), Details::Deposit);
    assert_eq!(details(1000), Details::CardCharge);
    assert_eq!(details(1100), Details::CardCharge);
}

#[test]
fn a_transaction_that_is_not_there_is_nothing() {
    let company = support::books();
    assert_eq!(company.transaction(TransactionId::new(7)).unwrap(), None);
    assert_eq!(company.transaction(TransactionId::new(0)).unwrap(), None);
}

#[test]
fn a_transaction_leads_to_its_vendor_or_to_another_name() {
    let company = support::books();
    let with = |id| {
        let read = forms::transaction(&company, id);
        (read.name, read.customer(), read.vendor())
    };
    let (vendor, job, fuel) = (NameId::new(201), NameId::new(103), NameId::new(401));
    assert_eq!(with(200), (Some(vendor), None, Some(vendor)));
    assert_eq!(with(1600), (Some(job), Some(job), None));
    assert_eq!(with(1700), (Some(fuel), None, None));
    assert_eq!(with(100), (None, None, None));
}

#[test]
fn a_bill_is_open_until_it_is_paid() {
    let company = support::books();
    let open = |id| forms::transaction(&company, id).open_state;
    assert_eq!(open(200), OpenState::Paid);
    assert!(matches!(open(1500), OpenState::Open(left) if left.to_string() == "40.00"));
}

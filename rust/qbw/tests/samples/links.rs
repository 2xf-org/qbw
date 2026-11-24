//! Links against what they settle.

use std::collections::{HashMap, HashSet};

use qbw::ledger::PostingSource;
use qbw::links::{LineLink, LinkCode, Relation};
use qbw::query::{PostingFilter, TransactionQuery, TransactionSummary};
use qbw::transactions::{OpenState, PostingState, TransactionKind};
use qbw::{Company, LineId, Money, Page, TransactionId};

use crate::agreement::Agreement;
use crate::support;

/// The transactions of a kind that post.
fn of_kind(company: &Company, kind: TransactionKind) -> Vec<TransactionSummary> {
    let posted = PostingFilter::Only(PostingState::Posted);
    let query = TransactionQuery::new().kinds([kind]).posting(posted);
    company.transactions(&query, Page::all()).unwrap().rows
}

fn add(left: Money, right: Money) -> Money {
    left.checked_add(right).unwrap()
}

/// How often what settled the paid transactions of a kind comes to their total.
fn paid_in_full(company: &Company, kind: TransactionKind) -> Agreement {
    let mut agreement = Agreement::default();
    for row in of_kind(company, kind) {
        let total = row.total.filter(|total| !total.is_zero());
        if let (Some(total), OpenState::Paid) = (total, row.open) {
            agreement.add(company.paid_by(row.id).unwrap().unwrap().total == total);
        }
    }
    agreement
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn paid_invoices_and_bills_were_paid_in_full() {
    for company in support::samples() {
        let invoices = paid_in_full(&company, TransactionKind::Invoice);
        invoices.check("paid invoices with links of their total");
        let bills = paid_in_full(&company, TransactionKind::Bill);
        bills.check("paid bills with links of their total");
    }
}

/// How often what a transaction of a kind was used for, with what is left of it,
/// comes to its total.
fn used_up(company: &Company, kind: TransactionKind) -> Agreement {
    let mut agreement = Agreement::default();
    for row in of_kind(company, kind) {
        let Some(total) = row.total else {
            continue;
        };
        let links = company.links(row.id).unwrap().unwrap();
        let used = links
            .iter()
            .filter(|link| matches!(link.relation, Relation::AppliedTo | Relation::RefundedBy));
        let used = used.filter_map(|link| link.amount);
        let left = match row.open {
            OpenState::Open(open) => open,
            _ => Money::zero(total.currency()),
        };
        agreement.add(used.fold(left, add) == total);
    }
    agreement
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn payments_and_credits_are_what_they_were_used_for_and_what_is_left() {
    for company in support::samples() {
        for kind in [
            TransactionKind::Payment,
            TransactionKind::CreditMemo,
            TransactionKind::BillPaymentCheck,
            TransactionKind::BillPaymentCreditCard,
            TransactionKind::VendorCredit,
        ] {
            let agreement = used_up(&company, kind);
            agreement.check(&format!("{kind:?}: total with what it was used for"));
        }
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn deposits_have_the_payments_in_them() {
    for company in support::samples() {
        let mut deposits = Agreement::default();
        for row in of_kind(&company, TransactionKind::Deposit) {
            let links = company.line_links(row.id).unwrap().unwrap();
            let links = links.links.iter();
            let links = links.filter(|link| link.code == LinkCode::Deposit);
            let links: Vec<&LineLink> = links.collect();
            if links.is_empty() {
                continue;
            }
            let lines: HashSet<LineId> = links.iter().filter_map(|link| link.line).collect();
            let deposit = company.transaction(row.id).unwrap().unwrap();
            let zero = Money::zero(deposit.currency);
            let brought = links.iter().filter_map(|link| link.amount).fold(zero, add);
            let own = deposit.lines.iter().filter(|line| lines.contains(&line.id));
            let own = own.filter_map(|line| line.amount).fold(zero, add);
            deposits.add(brought == own);
        }
        deposits.check("deposits whose lines are the payments they bring");
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn both_ends_of_a_link_say_the_same() {
    for company in support::samples() {
        let ledger = company.ledger().unwrap();
        let own = ledger
            .postings()
            .filter(|posting| posting.source == PostingSource::OwnLine);
        let ids: HashSet<TransactionId> = own.map(|posting| posting.transaction).collect();
        // What each transaction says that it has with another.
        let mut said: HashMap<(TransactionId, TransactionId), Money> = HashMap::new();
        for &id in &ids {
            for link in &company.links(id).unwrap().unwrap() {
                if let Some(amount) = link.amount {
                    let pair = said
                        .entry((id, link.other.id))
                        .or_insert(Money::zero(amount.currency()));
                    *pair = add(*pair, amount);
                }
            }
        }
        let mut pairs = Agreement::default();
        for (&(one, other), amount) in &said {
            if ids.contains(&other) {
                pairs.add(said.get(&(other, one)) == Some(amount));
            }
        }
        pairs.check("pairs of transactions that say the same of each other");
    }
}

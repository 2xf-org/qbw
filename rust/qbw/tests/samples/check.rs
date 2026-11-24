//! The check, on books that it was measured on.

use std::collections::BTreeMap;

use qbw::check::{Check, CheckKind};

use crate::books;

/// What the check of one of the books gave: of each kind, how many figures agree
/// and how many there are.
struct Measured {
    /// The transactions that post, which tell the books apart.
    transactions: usize,
    agree: &'static [(CheckKind, usize, usize)],
    /// The differences that no cause with an amount explains.
    unexplained: usize,
}

const MEASURED: &[Measured] = &[
    Measured {
        transactions: 95_684,
        agree: &[
            (CheckKind::TransactionBalance, 95_684, 95_684),
            (CheckKind::LedgerBalance, 1, 1),
            (CheckKind::HeaderAndLines, 108_155, 108_155),
            (CheckKind::LineChain, 108_155, 108_155),
            (CheckKind::FormTotal, 104_532, 104_532),
            (CheckKind::AccountBalance, 282, 285),
            (CheckKind::CustomerBalance, 1_076, 1_077),
            (CheckKind::VendorBalance, 1_409, 1_409),
            (CheckKind::OpenForm, 45_561, 45_566),
            (CheckKind::OpenReceivable, 34_427, 34_432),
            (CheckKind::OpenPayable, 11_134, 11_134),
            (CheckKind::ReceivableAccount, 1, 2),
            (CheckKind::PayableAccount, 2, 2),
            (CheckKind::ItemQuantity, 762, 762),
            (CheckKind::ItemCount, 762, 762),
            (CheckKind::ItemValue, 762, 762),
            (CheckKind::ItemPostings, 762, 762),
            (CheckKind::OnOrder, 762, 762),
            (CheckKind::OnSalesOrders, 762, 762),
            (CheckKind::ToBeBuilt, 762, 762),
            (CheckKind::RequiredForBuilds, 762, 762),
            (CheckKind::InventoryAsset, 0, 2),
            (CheckKind::YearlyFigure, 3, 3),
        ],
        unexplained: 14,
    },
    Measured {
        transactions: 61_442,
        agree: &[
            (CheckKind::TransactionBalance, 61_442, 61_442),
            (CheckKind::LedgerBalance, 1, 1),
            (CheckKind::HeaderAndLines, 64_189, 64_189),
            (CheckKind::LineChain, 64_189, 64_189),
            (CheckKind::FormTotal, 62_377, 62_378),
            (CheckKind::AccountBalance, 167, 167),
            (CheckKind::CustomerBalance, 991, 991),
            (CheckKind::VendorBalance, 965, 965),
            (CheckKind::OpenForm, 18_669, 18_669),
            (CheckKind::OpenReceivable, 15_045, 15_045),
            (CheckKind::OpenPayable, 3_626, 3_626),
            (CheckKind::ReceivableAccount, 2, 2),
            (CheckKind::PayableAccount, 2, 2),
            (CheckKind::ItemQuantity, 366, 366),
            (CheckKind::ItemCount, 366, 366),
            (CheckKind::ItemValue, 366, 366),
            (CheckKind::ItemPostings, 366, 366),
            (CheckKind::OnOrder, 366, 366),
            (CheckKind::OnSalesOrders, 366, 366),
            (CheckKind::ToBeBuilt, 366, 366),
            (CheckKind::RequiredForBuilds, 366, 366),
            (CheckKind::InventoryAsset, 0, 1),
            (CheckKind::YearlyFigure, 1, 3),
        ],
        unexplained: 3,
    },
    Measured {
        transactions: 108_015,
        agree: &[
            (CheckKind::TransactionBalance, 108_015, 108_015),
            (CheckKind::LedgerBalance, 1, 1),
            (CheckKind::HeaderAndLines, 117_485, 117_485),
            (CheckKind::LineChain, 117_485, 117_485),
            (CheckKind::FormTotal, 106_110, 106_110),
            (CheckKind::AccountBalance, 324, 325),
            (CheckKind::CustomerBalance, 846, 846),
            (CheckKind::VendorBalance, 1_102, 1_102),
            (CheckKind::OpenForm, 34_809, 34_809),
            (CheckKind::OpenReceivable, 18_048, 18_048),
            (CheckKind::OpenPayable, 16_761, 16_761),
            (CheckKind::ReceivableAccount, 1, 1),
            (CheckKind::PayableAccount, 2, 2),
            (CheckKind::ItemQuantity, 1_326, 1_326),
            (CheckKind::ItemCount, 1_326, 1_326),
            (CheckKind::ItemValue, 1_326, 1_326),
            (CheckKind::ItemPostings, 1_321, 1_326),
            (CheckKind::SiteQuantity, 5_499, 5_511),
            (CheckKind::OnOrder, 1_326, 1_326),
            (CheckKind::OnSalesOrders, 1_325, 1_326),
            (CheckKind::ToBeBuilt, 1_326, 1_326),
            (CheckKind::RequiredForBuilds, 1_326, 1_326),
            (CheckKind::InventoryAsset, 0, 2),
            (CheckKind::YearlyFigure, 2, 3),
        ],
        unexplained: 21,
    },
];

/// Prints how often figures agree, and how many differences of a kind no cause
/// explains.
fn print(check: &Check) {
    let mut unexplained: BTreeMap<CheckKind, usize> = BTreeMap::new();
    for found in check.unexplained() {
        *unexplained.entry(found.what).or_default() += 1;
    }
    for compared in &check.compared {
        let (agreed, looked_at) = (compared.agreed, compared.looked_at);
        let left = unexplained.get(&compared.what).copied().unwrap_or_default();
        println!(
            "{}: {agreed} of {looked_at}, {left} unexplained",
            compared.what
        );
    }
}

#[test]
#[ignore = "needs company files in the folder QBW_SAMPLES names"]
fn no_fewer_figures_agree_than_did() {
    for company in books::books() {
        let check = company.check().unwrap();
        print(&check);
        assert!(check.left_out.is_nothing());
        let transactions = check.get(CheckKind::TransactionBalance).unwrap();
        let mut measured = MEASURED.iter();
        let measured = measured.find(|book| book.transactions == transactions.looked_at);
        let Some(measured) = measured else {
            println!("a book that was not measured");
            continue;
        };
        let compared = check.compared.iter();
        let compared: Vec<_> = compared
            .map(|compared| (compared.what, compared.looked_at))
            .collect();
        let expected = measured.agree.iter();
        let expected: Vec<_> = expected
            .map(|&(what, _, looked_at)| (what, looked_at))
            .collect();
        assert_eq!(compared, expected);
        for &(what, agreed, _) in measured.agree {
            assert!(check.get(what).unwrap().agreed >= agreed, "{what}");
        }
        assert!(check.unexplained().count() <= measured.unexplained);
    }
}

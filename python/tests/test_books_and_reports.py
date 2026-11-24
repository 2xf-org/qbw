"""Tests for transactions, registers, links, statements, open items, aging, and check."""

from __future__ import annotations

import datetime

import qbw

from .conftest import Fixtures


def test_transactions_search_register_and_links(fixtures: Fixtures) -> None:
    with qbw.Company.open(str(fixtures.books)) as company:
        inv_id = company.find("Invoice", "1041")
        assert inv_id is not None
        assert inv_id in company.find_all("Invoice", "1041")

        summary = company.transaction_summary(inv_id)
        assert summary is not None
        assert summary.number == "1041"
        assert summary.kind == "Invoice"

        tx = company.transaction(inv_id)
        assert tx is not None
        assert tx.id == inv_id
        assert len(tx.lines) > 0
        assert tx.damage == []

        unpaid = company.transactions(
            qbw.TransactionFilter(
                kinds=["Invoice"],
                paid=qbw.PaidState.Unpaid,
                order=qbw.TransactionOrder.DueDate,
                direction=qbw.SortDirection.Ascending,
            ),
            offset=0,
            limit=50,
        )
        assert unpaid.total_count >= 1
        assert unpaid.rows[0].number == "1041"
        assert unpaid.left_out.rows == 0

        links = company.links(inv_id)
        paid_by = company.paid_by(inv_id)
        assert links is not None
        assert paid_by is not None
        assert len(paid_by.links) >= 1
        payment_id = paid_by.links[0].transaction_id
        applied = company.applied_to(payment_id)
        assert applied is not None
        assert any(link.transaction_id == inv_id for link in applied.links)

        reg = company.register(1, start=datetime.date(2024, 1, 1), end=datetime.date(2024, 12, 31))
        assert reg is not None
        assert reg.account_id == 1
        assert reg.opening_balance.amount == "800.00"
        assert reg.closing_balance.amount == "837.00"
        assert reg.total_count == len(reg.rows)

        assert company.register(999_999) is None


def test_financial_statements_open_items_aging_and_check(fixtures: Fixtures) -> None:
    with qbw.Company.open(str(fixtures.books)) as company:
        day = datetime.date(2024, 6, 30)
        sheet = company.balance_sheet(day)
        assert sheet.as_of == day
        assert sheet.assets.total.amount == "1262.00"
        assert sheet.liabilities_and_equity == sheet.assets.total
        assert sheet.unplaced.amount == "0.00"
        assert sheet.left_out.rows == 0

        fy = company.fiscal_year(day)
        pnl = company.profit_and_loss(fy.start, fy.end)
        assert pnl.net_income == sheet.equity.net_income
        assert pnl.unplaced.amount == "0.00"

        tb = company.trial_balance(day)
        assert tb.debits == tb.credits
        assert len(tb.rows) > 0

        ar = company.open_items(qbw.Side.Receivable)
        ap = company.open_items(qbw.Side.Payable)
        assert ar.side == qbw.Side.Receivable
        assert ap.side == qbw.Side.Payable
        assert len(ar.items) > 0

        aging = company.aging(qbw.Side.Receivable, datetime.date(2024, 2, 15))
        assert len(aging.buckets) == 5
        assert aging.buckets[0].label == "Current"
        assert aging.totals[3].amount == "46.00"

        report = company.check()
        acc_cmp = next(
            item
            for item in report.compared
            if item.kind == "accounts with the balance QuickBooks keeps"
        )
        assert (acc_cmp.agreed, acc_cmp.looked_at) == (21, 22)
        assert report.unexplained_count == 1
        assert report.findings[0].causes == ["maybe QuickBooks' total is out of date"]

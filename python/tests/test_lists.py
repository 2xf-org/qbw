"""Tests for accounts, names, items, stock, valuation, orders, and supporting lists."""

from __future__ import annotations

import datetime

import qbw

from .conftest import Fixtures


def test_accounts_and_balances(fixtures: Fixtures) -> None:
    with qbw.Company.open(str(fixtures.books)) as company:
        all_accounts = company.accounts()
        active_accounts = company.accounts(active_only=True)
        assert len(all_accounts) >= len(active_accounts) > 0

        parts = company.account_named("Sales:Parts")
        assert parts is not None
        assert parts.depth == 1
        assert parts.account_type == "Income"
        assert parts.normal_side == qbw.NormalSide.Credit
        assert parts.statement == qbw.Statement.ProfitAndLoss
        assert parts.stored is not None
        assert parts.stored.amount.amount == "-100.00"
        assert parts.stored.amount.currency_code == "USD"

        by_id = company.account(parts.id)
        assert by_id == parts
        assert company.account(999_999) is None
        assert company.account_named("No Such Account") is None

        bal_2023 = company.account_balance(1, datetime.date(2023, 12, 31))
        bal_now = company.account_balance(1)
        assert bal_2023 is not None
        assert bal_now is not None
        assert bal_2023.amount.amount == "800.00"
        assert bal_now.amount.amount == "837.00"


def test_names_and_parties(fixtures: Fixtures) -> None:
    with qbw.Company.open(str(fixtures.books)) as company:
        all_names = company.names()
        customers = company.customers()
        vendors = company.vendors()
        employees = company.employees()
        others = company.other_names()

        assert len(all_names) == len(customers) + len(vendors) + len(employees) + len(others)
        assert all(c.kind == qbw.NameKind.Customer for c in customers)
        assert all(v.kind == qbw.NameKind.Vendor for v in vendors)
        assert all(e.kind == qbw.NameKind.Employee for e in employees)
        assert all(o.kind == qbw.NameKind.Other for o in others)

        ada = company.name_named("ada stores")
        assert ada is not None
        assert ada.kind == qbw.NameKind.Customer
        assert company.name(ada.id) == ada


def test_items_stock_valuation_and_orders(fixtures: Fixtures) -> None:
    with qbw.Company.open(str(fixtures.books)) as company:
        items = company.items()
        assert len(items) > 0
        stocked = [item for item in items if item.is_stocked]
        assert len(stocked) > 0

        first = stocked[0]
        assert company.item(first.id) == first
        assert company.item_named(first.full_name) == first

        stock = company.stock()
        assert len(stock.rows) == len(stocked)
        assert stock.left_out.rows == 0

        val_latest = company.valuation()
        assert val_latest.as_of is None
        assert val_latest.value == stock.value

        val_day = company.valuation(datetime.date(2024, 6, 30))
        assert val_day.as_of == datetime.date(2024, 6, 30)

        sales_orders = company.open_orders(qbw.Side.Receivable)
        purchase_orders = company.open_orders(qbw.Side.Payable)
        assert sales_orders.side == qbw.Side.Receivable
        assert purchase_orders.side == qbw.Side.Payable


def test_supporting_lists(fixtures: Fixtures) -> None:
    with qbw.Company.open(str(fixtures.books)) as company:
        currencies = company.currencies()
        codes = {c.code for c in currencies}
        assert {"USD", "EUR", "JPY"}.issubset(codes)

        assert len(company.terms()) > 0
        assert isinstance(company.classes(), list)
        assert len(company.payment_methods()) > 0
        assert len(company.ship_methods()) > 0
        assert len(company.sales_reps()) > 0
        assert len(company.sales_tax_codes()) > 0

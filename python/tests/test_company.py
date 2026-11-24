"""Tests for opening company files, metadata, preferences, users, and raw rows."""

from __future__ import annotations

import datetime

import qbw

from .conftest import Fixtures


def test_open_and_constructors(fixtures: Fixtures) -> None:
    with qbw.Company(str(fixtures.books)) as company:
        info = company.info()
        assert info.name == "Harbor Valves"
        assert info.legal_name == "Harbor Valves Ltd"
        assert info.left_out.rows == 0
        assert info.left_out.pages == 0
        assert "Harbor Valves" in repr(company)

        key = company.key
        file_info = company.file_info
        assert file_info.page_size == 4096
        assert file_info.page_count > 0
        assert file_info.table_count > 0
        assert "abmc_account_user" in company.table_names()

    options = qbw.OpenOptions(key=key, longest_value=16 * 1024 * 1024)
    with qbw.Company.open(str(fixtures.books), options=options) as reopened:
        assert reopened.info().name == "Harbor Valves"
        assert reopened.key == key

    raw_bytes = fixtures.books.read_bytes()
    with qbw.Company.from_bytes(raw_bytes) as from_mem:
        assert from_mem.info().name == "Harbor Valves"


def test_preferences_users_fiscal_year_and_stored_row(fixtures: Fixtures) -> None:
    with qbw.Company.open(str(fixtures.books)) as company:
        prefs = company.preferences()
        assert prefs.uses_multicurrency is True
        assert prefs.home_currency_code == "USD"
        assert prefs.fiscal_year_start == 1
        assert prefs.left_out.rows == 0

        fy = company.fiscal_year(datetime.date(2024, 6, 30))
        assert fy.start == datetime.date(2024, 1, 1)
        assert fy.end == datetime.date(2024, 12, 31)

        days = company.transaction_days()
        assert days is not None
        assert days.start is not None
        assert days.end is not None
        assert days.start <= days.end

        users = company.users()
        assert len(users) >= 1

        parts = company.account_named("Sales:Parts")
        assert parts is not None
        row = company.stored_row(parts.origin)
        assert row is not None
        assert row["sublevel"] == 1

        missing = company.stored_row(qbw.Origin(table="no_such_table", page=0, slot=0))
        assert missing is None


def test_one_currency_company(fixtures: Fixtures) -> None:
    with qbw.Company.open(str(fixtures.one_currency)) as company:
        prefs = company.preferences()
        assert prefs.uses_multicurrency is False
        assert prefs.home_currency_id == 0
        assert prefs.home_currency_code is None

        sheet = company.balance_sheet()
        assert sheet.assets.total == sheet.liabilities_and_equity
        assert sheet.assets.total.currency_id == 0
        assert sheet.assets.total.currency_code is None

"""Tests for structured QbwError codes and damaged company files."""

from __future__ import annotations

import pathlib

import pytest

import qbw

from .conftest import Fixtures


def test_error_codes(fixtures: Fixtures, tmp_path: pathlib.Path) -> None:
    with pytest.raises(qbw.QbwError) as missing_exc:
        qbw.Company.open(str(tmp_path / "does_not_exist.qbw"))
    assert missing_exc.value.code == "io"

    with pytest.raises(qbw.QbwError) as bad_format_exc:
        qbw.Company.from_bytes(b"not a database")
    assert bad_format_exc.value.code == "format"

    with pytest.raises(qbw.QbwError) as not_company_exc:
        qbw.Company.open(str(fixtures.not_a_company))
    assert not_company_exc.value.code == "not_a_company"

    wrong_key = qbw.StoreKey(mask=1, addend=1, sign=qbw.KeySign.Positive)
    with pytest.raises(qbw.QbwError) as wrong_key_exc:
        qbw.Company.open(str(fixtures.books), options=qbw.OpenOptions(key=wrong_key))
    assert wrong_key_exc.value.code == "wrong_key"

    with qbw.Company.open(str(fixtures.books)) as company:
        with pytest.raises(qbw.QbwError) as bad_kind_exc:
            company.find("NotAKind", "100")
        assert bad_kind_exc.value.code == "invalid_argument"

        with pytest.raises(qbw.QbwError) as bad_buckets_exc:
            company.aging(qbw.Side.Receivable, days=0)
        assert bad_buckets_exc.value.code == "invalid_argument"


def test_damaged_company_tracks_left_out(fixtures: Fixtures) -> None:
    with qbw.Company.open(str(fixtures.damaged_books)) as company:
        ar = company.open_items(qbw.Side.Receivable)
        ap = company.open_items(qbw.Side.Payable)
        assert ar.left_out.rows == 1
        assert ap.left_out.rows == 1
        sheet = company.balance_sheet()
        assert sheet.left_out.rows >= 1

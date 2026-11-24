"""Tests for zero-copy Apache Arrow export into PyArrow, Polars, and DuckDB."""

from __future__ import annotations

import datetime
from decimal import Decimal

import duckdb
import polars as pl
import pyarrow as pa
import pytest

import qbw

from .conftest import Fixtures


def test_arrow_tables_and_schemas(fixtures: Fixtures) -> None:
    with qbw.Company.open(str(fixtures.books)) as company:
        accounts = company.accounts_arrow()
        assert accounts.num_rows == len(company.accounts())
        acc_types = dict(zip(accounts.schema.names, accounts.schema.types, strict=True))
        assert acc_types["id"] == pa.int32()
        assert acc_types["full_name"] == pa.utf8()
        assert acc_types["stored_balance"] == pa.decimal128(28, 5)

        names = company.names_arrow()
        assert names.num_rows == len(company.names())
        customers_only = company.names_arrow(kind=qbw.NameKind.Customer)
        assert customers_only.num_rows == len(company.customers())

        items = company.items_arrow()
        assert items.num_rows == len(company.items())

        txs = company.transactions_arrow()
        assert txs.num_rows > 0
        tx_types = dict(zip(txs.schema.names, txs.schema.types, strict=True))
        assert tx_types["date"] == pa.date32()
        assert tx_types["home_total"] == pa.decimal128(28, 5)

        ar = company.open_items_arrow(qbw.Side.Receivable)
        assert ar.num_rows == len(company.open_items(qbw.Side.Receivable).items)

        stock = company.stock_arrow()
        assert stock.num_rows == len(company.stock().rows)


def test_ledger_stream_polars_and_duckdb(fixtures: Fixtures) -> None:
    with qbw.Company.open(str(fixtures.books)) as company:
        # Small batch_size exercises multi-batch streaming across C Stream Interface.
        reader = company.ledger_arrow(batch_size=8)
        batches = list(reader)
        assert len(batches) > 1
        table = pa.Table.from_batches(batches)
        assert table.num_rows > 8

        df = pl.from_arrow(table)  # pyright: ignore[reportUnknownMemberType]
        assert isinstance(df, pl.DataFrame)
        assert df.height == table.num_rows

        # Double-entry invariant: all home_amount postings sum to exactly 0.
        stream = company.ledger_arrow(batch_size=16)
        conn = duckdb.connect()
        try:
            conn.register("ledger", stream)
            rows = conn.sql(
                """
                SELECT COUNT(*), SUM(home_amount)
                FROM ledger
                """
            ).fetchall()
        finally:
            conn.close()
        count, net_home = rows[0]
        assert count == table.num_rows
        assert net_home == Decimal("0.00000")

        # Filtered by account and date range.
        checking_stream = company.ledger_arrow(
            end=datetime.date(2023, 12, 31),
            account_id=1,
            batch_size=16,
        )
        checking_table = checking_stream.read_all()
        conn = duckdb.connect()
        try:
            conn.register("checking", checking_table)
            total = conn.sql("SELECT SUM(amount) FROM checking").fetchall()[0][0]
        finally:
            conn.close()
        assert total == Decimal("800.00000")

        with pytest.raises(qbw.QbwError) as exc_info:
            company.ledger_arrow(batch_size=0)
        assert exc_info.value.code == "invalid_argument"

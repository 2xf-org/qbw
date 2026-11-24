"""Export company data and stream ledger postings into PyArrow, Polars, and DuckDB."""

from __future__ import annotations

import sys

import duckdb
import polars as pl

import qbw


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: arrow_analytics.py <company.qbw>", file=sys.stderr)
        return 2

    with qbw.Company.open(argv[1]) as company:
        accounts_table = company.accounts_arrow()
        accounts_df = pl.from_arrow(accounts_table)  # pyright: ignore[reportUnknownMemberType]
        print(f"accounts rows={accounts_table.num_rows} cols={accounts_table.num_columns}")
        if isinstance(accounts_df, pl.DataFrame):
            print(f"polars accounts shape={accounts_df.shape}")

        ledger_reader = company.ledger_arrow(batch_size=1024)
        conn = duckdb.connect()
        try:
            conn.register("ledger", ledger_reader)
            summary = conn.sql(
                """
                SELECT
                    COUNT(*) AS postings,
                    SUM(home_amount) AS net_home
                FROM ledger
                """
            ).fetchall()
        finally:
            conn.close()
        postings, net_home = summary[0]
        print(f"duckdb postings={postings} net_home={net_home}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))

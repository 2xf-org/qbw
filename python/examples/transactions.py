"""Search unpaid invoices, inspect lines, and follow payment links."""

from __future__ import annotations

import sys

import qbw


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: transactions.py <company.qbw>", file=sys.stderr)
        return 2

    with qbw.Company.open(argv[1]) as company:
        query = qbw.TransactionFilter(
            kinds=["Invoice"],
            paid=qbw.PaidState.Unpaid,
            order=qbw.TransactionOrder.DueDate,
            direction=qbw.SortDirection.Ascending,
        )
        page = company.transactions(query, offset=0, limit=50)
        print(f"Unpaid invoices: {page.total_count}")
        for summary in page.rows:
            open_amount = summary.open.amount if summary.open is not None else "0.00"
            print(
                f"  #{summary.number or '-'}  {summary.date.isoformat()}  "
                f"{summary.name or '-'}  open={open_amount}"
            )
            full = company.transaction(summary.id)
            if full is not None:
                print(f"    lines={len(full.lines)}")
            settled = company.paid_by(summary.id)
            if settled is not None and settled.links:
                print(f"    settled={settled.total.amount if settled.total else '0.00'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))

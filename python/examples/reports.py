"""Print Balance Sheet, Profit and Loss, and Trial Balance as of a date."""

from __future__ import annotations

import datetime
import sys

import qbw


def main(argv: list[str]) -> int:
    if len(argv) not in (2, 3):
        print("usage: reports.py <company.qbw> [YYYY-MM-DD]", file=sys.stderr)
        return 2

    as_of = datetime.date.fromisoformat(argv[2]) if len(argv) == 3 else None

    with qbw.Company.open(argv[1]) as company:
        sheet = company.balance_sheet(as_of)
        fy = company.fiscal_year(sheet.as_of)
        pnl = company.profit_and_loss(fy.start, sheet.as_of)
        tb = company.trial_balance(sheet.as_of)

        print(f"As of: {sheet.as_of.isoformat()}")
        print(f"Assets: {sheet.assets.total.amount}")
        print(f"Liabilities and Equity: {sheet.liabilities_and_equity.amount}")
        print(f"Income: {pnl.income.total.amount}")
        print(f"Net Income: {pnl.net_income.amount}")
        print(f"Trial Balance: debits={tb.debits.amount} credits={tb.credits.amount}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))

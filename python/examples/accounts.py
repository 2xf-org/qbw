"""List active accounts from a QuickBooks Desktop (.qbw) company file."""

from __future__ import annotations

import sys

import qbw


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: accounts.py <company.qbw>", file=sys.stderr)
        return 2

    with qbw.Company.open(argv[1]) as company:
        info = company.info()
        print(f"Company: {info.name}")
        for account in company.accounts(active_only=True):
            balance = account.stored.amount.amount if account.stored is not None else ""
            indent = "  " * account.depth
            print(f"{indent}{account.name:<32}  {account.account_type:<24}  {balance:>12}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))

"""Cross-check computed figures against QuickBooks' stored balances."""

from __future__ import annotations

import sys

import qbw


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: check.py <company.qbw>", file=sys.stderr)
        return 2

    with qbw.Company.open(argv[1]) as company:
        report = company.check()
        print(
            f"compared={len(report.compared)} findings={len(report.findings)} "
            f"unexplained={report.unexplained_count} left_out_rows={report.left_out.rows}"
        )
        for item in report.compared:
            print(f"  {item.kind:<28} {item.agreed}/{item.looked_at}")
        for finding in report.findings:
            causes = "; ".join(finding.causes)
            print(
                f"  DIFF {finding.kind} ({finding.subject}): "
                f"computed={finding.computed} expected={finding.expected} [{causes}]"
            )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))

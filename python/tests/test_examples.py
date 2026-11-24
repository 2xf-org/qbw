"""Tests that every runnable Python example executes cleanly on synthetic fixtures."""

from __future__ import annotations

import pathlib
import subprocess
import sys

from .conftest import Fixtures

EXAMPLES_DIR = pathlib.Path(__file__).resolve().parents[1] / "examples"


def run_example(name: str, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(EXAMPLES_DIR / f"{name}.py"), *args],
        capture_output=True,
        text=True,
        check=False,
    )


def test_accounts_example(fixtures: Fixtures) -> None:
    result = run_example("accounts", str(fixtures.books))
    assert result.returncode == 0, result.stderr
    assert "Harbor Valves" in result.stdout
    assert "Parts" in result.stdout


def test_reports_example(fixtures: Fixtures) -> None:
    result = run_example("reports", str(fixtures.books), "2024-06-30")
    assert result.returncode == 0, result.stderr
    assert "Assets: 1262.00" in result.stdout
    assert "Liabilities and Equity: 1262.00" in result.stdout


def test_transactions_example(fixtures: Fixtures) -> None:
    result = run_example("transactions", str(fixtures.books))
    assert result.returncode == 0, result.stderr
    assert "#1041" in result.stdout


def test_arrow_analytics_example(fixtures: Fixtures) -> None:
    result = run_example("arrow_analytics", str(fixtures.books))
    assert result.returncode == 0, result.stderr
    assert "polars accounts shape=" in result.stdout
    assert "net_home=0.00000" in result.stdout


def test_check_example(fixtures: Fixtures) -> None:
    result = run_example("check", str(fixtures.books))
    assert result.returncode == 0, result.stderr
    assert "unexplained=1" in result.stdout
    assert "accounts with the balance QuickBooks keeps" in result.stdout

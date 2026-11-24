"""Shared synthetic .qbw company fixtures for Python integration tests."""

from __future__ import annotations

import pathlib
import subprocess
from dataclasses import dataclass

import pytest

REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]


@dataclass(frozen=True)
class Fixtures:
    """Paths to synthetic .qbw company files generated for the test session."""

    books: pathlib.Path
    damaged_books: pathlib.Path
    one_currency: pathlib.Path
    not_a_company: pathlib.Path


@pytest.fixture(scope="session")
def fixtures(tmp_path_factory: pytest.TempPathFactory) -> Fixtures:
    """Generate fresh synthetic .qbw files once per test session."""
    output_dir = tmp_path_factory.mktemp("qbw-fixtures")
    subprocess.run(
        [
            "cargo",
            "run",
            "--quiet",
            "-p",
            "qbw-synthetic",
            "--example",
            "fixtures",
            "--",
            str(output_dir),
        ],
        cwd=REPO_ROOT,
        check=True,
    )
    return Fixtures(
        books=output_dir / "books.qbw",
        damaged_books=output_dir / "damaged_books.qbw",
        one_currency=output_dir / "one_currency.qbw",
        not_a_company=output_dir / "not_a_company.db",
    )

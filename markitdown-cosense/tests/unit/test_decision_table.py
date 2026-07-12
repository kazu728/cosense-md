"""Data-driven conversion tests backed by the shared fixture JSON.

The same fixture is the Rust core's conformance suite; here it is exercised
through the compiled `_core.convert` so the Python surface is validated too.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import TypedDict, cast

import pytest

from markitdown_cosense import _core

# Fixtures live at the repository root (language-neutral source of truth), which
# is three parents up from this file (tests/unit/ -> tests/ -> markitdown-cosense/ -> root).
FIXTURE_DIR = Path(__file__).resolve().parents[3] / "fixtures"


class FixtureCase(TypedDict):
    id: str
    source: list[str]
    expected: list[str]


def _load(name: str) -> list:
    table = cast(
        dict[str, list[FixtureCase]],
        json.loads((FIXTURE_DIR / name).read_text(encoding="utf-8")),
    )
    return [
        pytest.param(
            case["source"],
            case["expected"],
            id=f"{category}:{case['id']}",
        )
        for category, cases in table.items()
        for case in cases
    ]


DECISION_CASES = _load("cosense_decision_table.json")


@pytest.mark.parametrize("source,expected", DECISION_CASES)
def test_decision_table(source: list[str], expected: list[str]) -> None:
    assert _core.convert("\n".join(source)) == "\n".join(expected)

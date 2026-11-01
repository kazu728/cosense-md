"""Data-driven conversion tests backed by the shared fixture JSON.

The same fixture is the Rust core's conformance suite; here it is exercised
through the compiled `_core.convert` so the Python surface is validated too.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from markitdown_cosense import _core

# The fixture lives at the repository root (language-neutral source of truth):
# tests/unit/ -> tests/ -> markitdown-cosense/ -> root.
FIXTURE = Path(__file__).resolve().parents[3] / "fixtures/cosense_decision_table.json"

DECISION_CASES = [
    pytest.param(case["source"], case["expected"], id=f"{category}:{case['id']}")
    for category, cases in json.loads(FIXTURE.read_text(encoding="utf-8")).items()
    for case in cases
]


@pytest.mark.parametrize("source,expected", DECISION_CASES)
def test_decision_table(source: list[str], expected: list[str]) -> None:
    assert _core.convert("\n".join(source)) == "\n".join(expected)

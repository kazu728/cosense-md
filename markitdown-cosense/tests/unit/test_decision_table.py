from __future__ import annotations

import json
from pathlib import Path

import pytest

from markitdown_cosense import _core

FIXTURE = Path(__file__).resolve().parents[3] / "fixtures/cosense_decision_table.json"

DECISION_CASES = [
    pytest.param(case["source"], case["expected"], id=f"{category}:{case['id']}")
    for category, cases in json.loads(FIXTURE.read_text(encoding="utf-8")).items()
    for case in cases
]


@pytest.mark.parametrize("source,expected", DECISION_CASES)
def test_decision_table(source: list[str], expected: list[str]) -> None:
    assert _core.convert("\n".join(source)) == "\n".join(expected)

from __future__ import annotations

import io

import pytest
from markitdown import MarkItDown, StreamInfo

from markitdown_cosense._plugin import CosenseConverter


@pytest.fixture
def converter() -> CosenseConverter:
    return CosenseConverter()


def test_plugin_entry_point_converts() -> None:
    md = MarkItDown(enable_plugins=True)
    stream = io.BytesIO(b"[* Title]\n[tag]")
    result = md.convert_stream(stream, stream_info=StreamInfo(extension=".txt"))
    assert result.text_content == "# Title\n#tag"


@pytest.mark.parametrize(
    "snippet,expected",
    [
        ("[* Heading]\n[tag]", True),
        ("code:python\n print()", True),
        ("table:Users\n Name", True),
        ("Plain text without markers", False),
        ("", False),
    ],
    ids=["heading", "code", "table", "plain", "empty"],
)
def test_accepts_heuristics(
    converter: CosenseConverter, snippet: str, expected: bool
) -> None:
    stream = io.BytesIO(snippet.encode("utf-8"))
    assert converter.accepts(stream, StreamInfo(extension=".md")) is expected


def test_convert_non_utf8_does_not_raise(converter: CosenseConverter) -> None:
    stream = io.BytesIO("[* 見出し]".encode("shift_jis"))
    result = converter.convert(stream, StreamInfo(extension=".txt"))
    assert isinstance(result.text_content, str)

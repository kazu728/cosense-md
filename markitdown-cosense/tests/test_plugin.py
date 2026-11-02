from __future__ import annotations

import io
from pathlib import Path

import pytest
from markitdown import MarkItDown, StreamInfo

from markitdown_cosense import CosenseConverter


@pytest.fixture
def markitdown() -> MarkItDown:
    return MarkItDown(enable_plugins=True)


@pytest.fixture
def converter() -> CosenseConverter:
    return CosenseConverter()


def _convert_stream(md: MarkItDown, text: str, extension: str = ".txt") -> str:
    stream = io.BytesIO(text.encode("utf-8"))
    result = md.convert_stream(stream, stream_info=StreamInfo(extension=extension))
    return result.text_content


@pytest.mark.parametrize(
    "source,expected",
    [
        ("[* Title]\n[tag]", "# Title\n#tag"),
        ("code:python\n print('hello')", "```python\nprint('hello')\n```"),
        (
            "table:Data\n Name\tAge\n Alice\t30\n Bob\t25",
            "## Data\n\n| Name | Age |\n|---|---|\n| Alice | 30 |\n| Bob | 25 |",
        ),
    ],
    ids=["heading", "code", "table"],
)
def test_conversion_via_markitdown(
    markitdown: MarkItDown, source: str, expected: str
) -> None:
    assert _convert_stream(markitdown, source) == expected


@pytest.mark.parametrize(
    "snippet,extension,expected",
    [
        ("[* Heading]\n[tag]", ".txt", True),
        ("code:python\n print()", ".txt", True),
        ("table:Users\n Name", "", True),
        ("Plain text without markers", ".txt", False),
        ("", ".txt", False),
        ("[* Heading]\n[tag]", ".md", False),
    ],
    ids=["heading", "code", "table", "plain", "empty", "markdown-file"],
)
def test_accepts_heuristics(
    converter: CosenseConverter, snippet: str, extension: str, expected: bool
) -> None:
    stream = io.BytesIO(snippet.encode("utf-8"))
    assert converter.accepts(stream, StreamInfo(extension=extension)) is expected


def test_accepts_does_not_consume_stream(converter: CosenseConverter) -> None:
    payload = "[* note]\n[tag]"
    stream = io.BytesIO(payload.encode("utf-8"))
    assert converter.accepts(stream, StreamInfo(extension=".txt"))
    assert stream.read().decode("utf-8") == payload


def test_other_text_formats_are_left_alone(markitdown: MarkItDown) -> None:
    source = "class Error:\n    code: int\n    message: str"
    assert _convert_stream(markitdown, source, extension=".py") == source


def test_convert_non_utf8_does_not_raise(converter: CosenseConverter) -> None:
    stream = io.BytesIO("[* 見出し]".encode("shift_jis"))
    result = converter.convert(stream, StreamInfo(extension=".txt"))
    assert isinstance(result.text_content, str)


def test_markitdown_convert_from_file(markitdown: MarkItDown, tmp_path: Path) -> None:
    lines = ["[* File Heading]", "[img https://example.com/logo.png]"]
    path = tmp_path / "note.txt"
    path.write_text("\n".join(lines), encoding="utf-8")

    result = markitdown.convert(path)
    assert "# File Heading" in result.text_content
    assert "![img](https://example.com/logo.png)" in result.text_content

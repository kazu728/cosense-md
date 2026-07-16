"""Integration tests for the Cosense → Markdown plugin (MarkItDown + glue)."""

from __future__ import annotations

import io
from pathlib import Path

import pytest
from markitdown import MarkItDown, StreamInfo

from markitdown_cosense import _core, register_converters
from markitdown_cosense._plugin import MarkdownConverter


@pytest.fixture
def markitdown_with_cosense() -> MarkItDown:
    md = MarkItDown()
    register_converters(md)
    return md


@pytest.fixture
def converter() -> MarkdownConverter:
    return MarkdownConverter()


def _convert_stream(md: MarkItDown, text: str, extension: str = ".txt") -> str:
    stream = io.BytesIO(text.encode("utf-8"))
    result = md.convert_stream(stream, stream_info=StreamInfo(extension=extension))
    return result.text_content


class _NonSeekable(io.BytesIO):
    """A byte stream that refuses to rewind, to exercise the non-seekable path."""

    def seekable(self) -> bool:
        return False

    def seek(self, *args: object, **kwargs: object) -> int:
        raise OSError("not seekable")

    def tell(self, *args: object, **kwargs: object) -> int:
        raise OSError("not seekable")


# ---------------------------------------------------------------------------
# Conversion via MarkItDown
# ---------------------------------------------------------------------------


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
    markitdown_with_cosense: MarkItDown, source: str, expected: str
) -> None:
    assert _convert_stream(markitdown_with_cosense, source) == expected


def test_core_backs_the_plugin() -> None:
    # The plugin output is exactly the core's markdown output.
    assert _core.convert("[* Title]") == "# Title"


# ---------------------------------------------------------------------------
# accepts heuristics and stream handling
# ---------------------------------------------------------------------------


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
    converter: MarkdownConverter, snippet: str, expected: bool
) -> None:
    stream = io.BytesIO(snippet.encode("utf-8"))
    assert converter.accepts(stream, StreamInfo(extension=".md")) is expected


def test_accepts_does_not_consume_stream(converter: MarkdownConverter) -> None:
    payload = "[* note]\n[tag]"
    stream = io.BytesIO(payload.encode("utf-8"))
    assert converter.accepts(stream, StreamInfo(extension=".md"))
    assert stream.read().decode("utf-8") == payload


def test_accepts_refuses_non_seekable_stream(converter: MarkdownConverter) -> None:
    stream = _NonSeekable(b"[* Heading]\n[tag]")
    # Cannot rewind, so the converter declines rather than consuming it.
    assert converter.accepts(stream, StreamInfo(extension=".md")) is False


def test_convert_non_utf8_does_not_raise(converter: MarkdownConverter) -> None:
    stream = io.BytesIO("[* 見出し]".encode("shift_jis"))
    result = converter.convert(stream, StreamInfo(extension=".txt"))
    assert isinstance(result.text_content, str)


# ---------------------------------------------------------------------------
# Filesystem integration
# ---------------------------------------------------------------------------


def test_markitdown_convert_from_file(
    markitdown_with_cosense: MarkItDown, tmp_path: Path
) -> None:
    lines = ["[* File Heading]", "[img https://example.com/logo.png]"]
    path = tmp_path / "note.txt"
    path.write_text("\n".join(lines), encoding="utf-8")

    result = markitdown_with_cosense.convert(path)
    assert "# File Heading" in result.text_content
    assert "![img](https://example.com/logo.png)" in result.text_content

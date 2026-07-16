"""MarkItDown integration glue.

All Cosense→Markdown conversion lives in the compiled Rust core (`._core`). This
module only decides whether a stream looks like Cosense and hands its text to the
core; it holds no conversion rules of its own.
"""

from __future__ import annotations

from contextlib import suppress
from typing import BinaryIO

from markitdown import (
    DocumentConverter,
    DocumentConverterResult,
    MarkItDown,
    StreamInfo,
)

from . import _core

__plugin_interface_version__ = 1


def register_converters(markitdown: MarkItDown, **_: object) -> None:
    markitdown.register_converter(MarkdownConverter())


class MarkdownConverter(DocumentConverter):
    def accepts(
        self, file_stream: BinaryIO, stream_info: StreamInfo, **_: object
    ) -> bool:
        text = self._peek(file_stream)
        return bool(text) and _core.looks_like_cosense(text)

    def convert(
        self, file_stream: BinaryIO, stream_info: StreamInfo, **_: object
    ) -> DocumentConverterResult:
        with suppress(OSError):
            file_stream.seek(0)
        data = file_stream.read()
        # Lenient decode: a stray non-UTF-8 byte must never turn conversion into
        # an exception (detection uses the same policy, so the two never disagree).
        text = data.decode("utf-8", errors="replace")
        return DocumentConverterResult(_core.convert(text))

    def _peek(self, file_stream: BinaryIO, size: int = 4096) -> str:
        # Only peek streams we can rewind. A non-seekable stream is refused here
        # rather than partially consumed, so convert always sees it from byte 0.
        if not file_stream.seekable():
            return ""
        position = file_stream.tell()
        snippet = file_stream.read(size)
        file_stream.seek(position)
        return snippet.decode("utf-8", errors="replace") if snippet else ""

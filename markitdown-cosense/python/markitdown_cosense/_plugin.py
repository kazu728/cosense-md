"""MarkItDown integration glue.

All Cosense→Markdown conversion lives in the compiled Rust core (`._core`). This
module only decides whether a stream looks like Cosense and hands its text to the
core; it holds no conversion rules of its own.
"""

from __future__ import annotations

from typing import BinaryIO

from markitdown import (
    DocumentConverter,
    DocumentConverterResult,
    MarkItDown,
    StreamInfo,
)

from . import _core

__plugin_interface_version__ = 1

_PEEK_SIZE = 4096


def register_converters(markitdown: MarkItDown, **_: object) -> None:
    markitdown.register_converter(CosenseConverter())


class CosenseConverter(DocumentConverter):
    # MarkItDown buffers non-seekable input before calling converters, so the
    # stream can always be rewound here.
    def accepts(
        self, file_stream: BinaryIO, stream_info: StreamInfo, **_: object
    ) -> bool:
        position = file_stream.tell()
        snippet = file_stream.read(_PEEK_SIZE)
        file_stream.seek(position)
        return _core.looks_like_cosense(_decode(snippet))

    def convert(
        self, file_stream: BinaryIO, stream_info: StreamInfo, **_: object
    ) -> DocumentConverterResult:
        return DocumentConverterResult(_core.convert(_decode(file_stream.read())))


def _decode(data: bytes) -> str:
    # Lenient decode: a stray non-UTF-8 byte must never turn detection or
    # conversion into an exception, and both use the same policy.
    return data.decode("utf-8", errors="replace")

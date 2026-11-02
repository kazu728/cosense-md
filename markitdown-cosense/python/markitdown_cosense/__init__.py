from __future__ import annotations

from importlib.metadata import version
from typing import BinaryIO

from markitdown import (
    DocumentConverter,
    DocumentConverterResult,
    MarkItDown,
    StreamInfo,
)

from . import _core

__version__ = version("markitdown-cosense")
__plugin_interface_version__ = 1

_PEEK_SIZE = 4096


def register_converters(markitdown: MarkItDown, **_: object) -> None:
    markitdown.register_converter(CosenseConverter())


class CosenseConverter(DocumentConverter):
    # MarkItDown supplies seekable streams.
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
    return data.decode("utf-8", errors="replace")

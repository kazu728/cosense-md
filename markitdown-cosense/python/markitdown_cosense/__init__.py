"""markitdown-cosense: A markitdown plugin for converting Cosense notation to Markdown."""

from importlib.metadata import PackageNotFoundError, version

from ._plugin import __plugin_interface_version__, register_converters

try:
    __version__ = version("markitdown-cosense")
except PackageNotFoundError:  # running from a source tree without an install
    __version__ = "0.0.0"

__all__ = [
    "register_converters",
    "__plugin_interface_version__",
    "__version__",
]

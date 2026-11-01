from importlib.metadata import version

from ._plugin import __plugin_interface_version__, register_converters

__version__ = version("markitdown-cosense")

__all__ = [
    "__plugin_interface_version__",
    "__version__",
    "register_converters",
]

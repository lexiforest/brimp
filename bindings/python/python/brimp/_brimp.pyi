"""Public native exceptions. Private extension objects remain implementation details."""

from typing import Any

_Session: Any

class RuntimeError(Exception):
    """Base class for errors reported by the native runtime."""

class InvalidInput(RuntimeError):
    """Invalid arguments or unsupported input configuration."""

class JavaScriptError(RuntimeError):
    """Page JavaScript evaluation failed."""

class Timeout(RuntimeError):
    """The real operation deadline expired."""

class Closed(RuntimeError):
    """The page or session has been closed."""

class Reentry(RuntimeError):
    """A host callback attempted to reenter its page or session."""

class StaleHandle(RuntimeError):
    """A handle belongs to a previous navigation realm."""

class TransportError(RuntimeError):
    """A native network operation failed."""

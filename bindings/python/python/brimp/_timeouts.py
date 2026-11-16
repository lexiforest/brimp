"""Normalize requests-style timeout inputs to Brimp's single deadline."""

import math

from .exceptions import InvalidInput


UNSET = object()


def normalize_timeout(value):
    if value is UNSET:
        return None  # Inherit the native session timeout.
    if value is None:
        return 0.0  # Zero disables the deadline.
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise InvalidInput("timeout must be nonnegative numeric seconds or None")
    try:
        if not math.isfinite(value) or value < 0:
            raise InvalidInput("timeout must be finite and nonnegative")
        return float(value)
    except OverflowError:
        raise InvalidInput("timeout is too large") from None

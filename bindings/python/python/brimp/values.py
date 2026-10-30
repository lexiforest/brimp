"""Lossless copied bridge values. Live JavaScript identity uses explicit handles."""

import math
from dataclasses import dataclass


class _Undefined:
    def __repr__(self):
        return "undefined"


undefined = _Undefined()


class BigInt(int):
    """Explicit JavaScript BigInt, including values inside Number's safe range."""


@dataclass(frozen=True)
class HostFunction:
    """Reference to a handler registered by the current plugin."""

    name: str


def encode(value, ancestors=None):
    if value is undefined:
        return {"type": "undefined"}
    if value is None:
        return {"type": "null"}
    if isinstance(value, bool):
        return {"type": "boolean", "value": value}
    if isinstance(value, BigInt) or isinstance(value, int) and abs(value) > 2**53 - 1:
        return {"type": "bigInt", "value": str(value)}
    if isinstance(value, (int, float)):
        if math.isnan(value):
            special = "NaN"
        elif math.isinf(value):
            special = "Infinity" if value > 0 else "-Infinity"
        elif value == 0 and math.copysign(1, value) < 0:
            special = "-0"
        else:
            return {"type": "number", "value": value}
        return {"type": "specialNumber", "value": special}
    if isinstance(value, str):
        return {"type": "string", "value": value}
    if isinstance(value, (bytes, bytearray, memoryview)):
        return {"type": "binary", "value": list(bytes(value))}
    if isinstance(value, HostFunction):
        return {"type": "hostFunction", "value": value.name}
    from . import JSHandle

    if isinstance(value, JSHandle):
        return {"type": "reference", "value": value._reference()}
    ancestors = set() if ancestors is None else ancestors
    if id(value) in ancestors:
        raise TypeError("Cyclic host data requires a JSHandle")
    ancestors.add(id(value))
    try:
        if isinstance(value, (list, tuple)):
            return {"type": "array", "value": [encode(item, ancestors) for item in value]}
        if isinstance(value, dict) and all(isinstance(key, str) for key in value):
            return {
                "type": "object",
                "value": {key: encode(item, ancestors) for key, item in value.items()},
            }
        raise TypeError("Unsupported bridge value; use copied data or a JSHandle")
    finally:
        ancestors.remove(id(value))


def decode(wire, reference=None):
    kind, value = wire["type"], wire.get("value")
    if kind == "undefined":
        return undefined
    if kind == "null":
        return None
    if kind == "specialNumber":
        return {"NaN": math.nan, "Infinity": math.inf, "-Infinity": -math.inf, "-0": -0.0}[value]
    if kind == "bigInt":
        return BigInt(value)
    if kind == "binary":
        return bytes(value)
    if kind == "array":
        return [decode(item, reference) for item in value]
    if kind == "object":
        return {key: decode(item, reference) for key, item in value.items()}
    if kind == "reference":
        if reference is None:
            raise TypeError("References require a live realm")
        return reference(value)
    if (
        kind == "number"
        and isinstance(value, float)
        and value.is_integer()
        and abs(value) <= 2**53 - 1
    ):
        return int(value)
    if kind in ("number", "boolean", "string"):
        return value
    raise TypeError(f"Invalid bridge value: {kind}")

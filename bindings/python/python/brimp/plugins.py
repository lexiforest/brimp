"""Explicit JavaScript plugins. Handlers run on a session's managed dispatcher."""
from dataclasses import dataclass, field, replace
import inspect
import json
import math
import threading

from ._brimp import Reentry

_active = threading.local()


def _encode(value):
    if value is None: return {"type": "null"}
    if isinstance(value, bool): return {"type": "boolean", "value": value}
    if isinstance(value, (int, float)):
        if not math.isfinite(value): raise ValueError("host numbers must be finite")
        return {"type": "number", "value": value}
    if isinstance(value, str): return {"type": "string", "value": value}
    if isinstance(value, (bytes, bytearray, memoryview)):
        return {"type": "binary", "value": list(bytes(value))}
    if isinstance(value, (list, tuple)):
        return {"type": "array", "value": [_encode(item) for item in value]}
    if isinstance(value, dict) and all(isinstance(key, str) for key in value):
        return {"type": "object", "value": {key: _encode(item) for key, item in value.items()}}
    raise TypeError("host values must be JSON-like values or bytes")


def _decode(wire):
    kind, value = wire["type"], wire.get("value")
    if kind == "binary": return bytes(value)
    if kind == "array": return [_decode(item) for item in value]
    if kind == "object": return {key: _decode(item) for key, item in value.items()}
    return value


class HostContext:
    """Realm-bound notification sender; late results are discarded after cancellation."""
    def __init__(self, inner): self._inner = inner
    @property
    def session_id(self): return self._inner.session_id
    @property
    def url(self): return self._inner.url
    @property
    def origin(self): return self._inner.origin
    @property
    def page_id(self): return self._inner.page_id
    @property
    def realm_id(self): return self._inner.realm_id
    @property
    def cancelled(self): return self._inner.cancelled
    def emit(self, channel, value):
        self._inner.emit(channel, json.dumps(_encode(value), allow_nan=False))


@dataclass(frozen=True)
class Plugin:
    name: str
    source: str
    handlers: dict = field(default_factory=dict)
    options: dict = field(default_factory=dict)
    setup: object = None

    @classmethod
    def from_file(cls, path, *, name, handlers=None, options=None, setup=None):
        from pathlib import Path
        path = Path(path).resolve()
        return cls(name, path.read_text() + f"\n//# sourceURL={path.as_uri()}\n", handlers or {}, options or {}, setup)

    def _bind(self):
        scope = self.setup() if self.setup else {}
        if inspect.isawaitable(scope):
            if inspect.iscoroutine(scope): scope.close()
            raise TypeError("Plugin setup must be synchronous")
        return replace(self, handlers={**self.handlers, **scope.get("handlers", {})}, setup=None), scope.get("dispose")

    def _descriptor(self, session):
        def adapt(handler):
            def invoke(context, arguments):
                previous = getattr(_active, "session", None)
                _active.session = session
                try:
                    result = handler(HostContext(context), *[_decode(arg) for arg in json.loads(arguments)])
                    if inspect.isawaitable(result):
                        if inspect.iscoroutine(result): result.close()
                        raise TypeError("Python plugin handlers must be synchronous; asyncio is not supported")
                    return json.dumps(_encode(result), allow_nan=False)
                finally:
                    _active.session = previous
            return invoke
        source = f"context => ({self.source}\n)({{...context, options: {json.dumps(self.options, allow_nan=False)}}})"
        return self.name, source, [(name, adapt(handler)) for name, handler in self.handlers.items()]


def _check_session(session):
    if getattr(_active, "session", None) is session:
        raise Reentry("cannot create pages or close the session from its host callback")


def _bundled(name, options=None):
    from pathlib import Path
    source = (Path(__file__).parent / "bundled" / f"{name}.js").read_text()
    if options is not None:
        source = f"context => ({source})(context, {json.dumps(options, allow_nan=False)})"
    return Plugin("defuddle" if name == "extraction" else name, source)


def storage():
    """Functional realm-local in-memory storage; no filesystem durability or OPFS."""
    return _bundled("storage")


def canvas(*, rgba=(0, 0, 0, 0), character_width=0.5):
    """Stateful Canvas mock. Drawing has no effect on the configured readback color."""
    return _bundled("canvas", {"rgba": list(rgba), "characterWidth": character_width})


def services(**options):
    """Realm-local locks and deterministic notification/device-service mocks."""
    return _bundled("services", options)


def extraction():
    """Explicit Defuddle article extraction using DOM-only heuristics."""
    return _bundled("extraction")


def _callback(session, handler):
    def invoke(*args):
        previous = getattr(_active, "session", None)
        _active.session = session
        try:
            result = handler(*args)
            if inspect.isawaitable(result):
                if inspect.iscoroutine(result): result.close()
                raise TypeError("Python callbacks must be synchronous")
            return result
        finally:
            _active.session = previous
    return invoke


def geometry():
    """Functional points, rectangles, quads and 2D matrices; no DOM layout."""
    return _bundled("geometry")

def css_typed_om():
    """Unit arithmetic and inline property maps; no cascade or computed style."""
    return _bundled("css-typed-om")

def scheduling():
    """Prioritized tasks, cancellation and deterministic zero-budget idle callbacks."""
    return _bundled("scheduling")

def graphics(*, rgba=(0, 0, 0, 0), renderer="Deterministic JavaScript mock", webgpu=False, handlers=None):
    """WebGL state/readback and opt-in WebGPU buffer mocks; no shader execution."""
    plugin = _bundled("graphics", dict(rgba=list(rgba), renderer=renderer, webgpu=webgpu))
    return Plugin(plugin.name, plugin.source, handlers or {})

def audio(*, sample_rate=48000, handlers=None):
    """Audio graph/buffer state and silent analysis; no sound generation or decoding."""
    plugin = _bundled("audio", dict(sampleRate=sample_rate))
    return Plugin(plugin.name, plugin.source, handlers or {})


def catalog():
    """Return host-side capability/options metadata, without installing plugins."""
    from pathlib import Path
    return json.loads((Path(__file__).parent / "bundled/catalog.json").read_text())


class _MemoryStorageBackend:
    def __init__(self):
        self.origins = {}

    def transaction(self, origin, operations):
        values = dict(self.origins.get(origin, {}))
        results = []
        for operation in operations:
            kind, key = operation["type"], operation.get("key")
            if kind == "get": results.append(values.get(key))
            elif kind == "set":
                values[key] = operation["value"]
                results.append(None)
            elif kind == "delete":
                results.append(key in values)
                values.pop(key, None)
            else:
                values.clear()
                results.append(None)
        self.origins[origin] = values
        return results

    def close(self):
        self.origins.clear()


def host_storage(*, backend_factory=_MemoryStorageBackend):
    """Explicit async hostStorage API, partitioned by native origin.

    A backend provides synchronous transaction(origin, operations) and optional
    close(). It must atomically commit/rollback batches, including across sessions
    if the supplied backend shares durable state. The default is session memory.
    """
    def setup():
        backend = backend_factory()
        if not callable(getattr(backend, "transaction", None)):
            raise TypeError("backend requires transaction(origin, operations)")
        lock = threading.RLock()

        def transaction(context, operations):
            if context.origin is None:
                raise ValueError("Host storage requires a non-opaque origin")
            if not isinstance(operations, list):
                raise TypeError("transaction requires an operation array")
            for op in operations:
                if (not isinstance(op, dict) or op.get("type") not in ("get", "set", "delete", "clear")
                    or (op["type"] != "clear" and not isinstance(op.get("key"), str))
                    or (op["type"] == "set" and "value" not in op)):
                    raise TypeError("invalid storage operation")
            with lock:
                if context.cancelled:
                    raise RuntimeError("Storage request cancelled")
                return backend.transaction(context.origin, operations)

        def dispose():
            with lock:
                close = getattr(backend, "close", None)
                if close: close()

        return {"handlers": {"transaction": transaction}, "dispose": dispose}

    return Plugin("host-storage", _bundled("host-storage").source, setup=setup)

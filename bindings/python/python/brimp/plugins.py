"""Explicit JavaScript plugins. Handlers run on a session's managed dispatcher."""

from dataclasses import dataclass, field, replace
import inspect
import json
import threading

from ._brimp import Reentry

_active = threading.local()


from .values import encode as _encode, decode as _decode, HostFunction


class HostContext:
    """Realm-bound notification sender; late results are discarded after cancellation."""

    def __init__(self, inner):
        self._inner = inner

    @property
    def session_id(self):
        return self._inner.session_id

    @property
    def url(self):
        return self._inner.url

    @property
    def origin(self):
        return self._inner.origin

    @property
    def page_id(self):
        return self._inner.page_id

    @property
    def realm_id(self):
        return self._inner.realm_id

    @property
    def cancelled(self):
        return self._inner.cancelled

    def function(self, name):
        return HostFunction(name)

    def _from_reference(self, token):
        from . import JSHandle

        return JSHandle(self._inner.bind_reference(json.dumps(token)))

    def emit(self, channel, value):
        self._inner.emit(channel, json.dumps(_encode(value), allow_nan=False))


@dataclass(frozen=True)
class Plugin:
    """JavaScript installer plus synchronous host handlers and per-session setup.

    The installer receives global, host and options. It may return a Promise
    and a disposer. Host handlers receive HostContext followed by copied values.
    """

    name: str
    source: str
    handlers: dict = field(default_factory=dict)
    options: dict = field(default_factory=dict)
    setup: object = None

    @classmethod
    def from_file(cls, path, *, name, handlers=None, options=None, setup=None):
        from pathlib import Path

        path = Path(path).resolve()
        return cls(
            name,
            path.read_text() + f"\n//# sourceURL={path.as_uri()}\n",
            handlers or {},
            options or {},
            setup,
        )

    def _bind(self):
        scope = self.setup() if self.setup else {}
        if inspect.isawaitable(scope):
            if inspect.iscoroutine(scope):
                scope.close()
            raise TypeError("Plugin setup must be synchronous")
        return replace(
            self, handlers={**self.handlers, **scope.get("handlers", {})}, setup=None
        ), scope.get("dispose")

    def _descriptor(self, session):
        def adapt(handler):
            def invoke(context, arguments):
                previous = getattr(_active, "session", None)
                _active.session = session
                try:
                    context = HostContext(context)
                    result = handler(
                        context,
                        *[_decode(arg, context._from_reference) for arg in json.loads(arguments)],
                    )
                    if inspect.isawaitable(result):
                        if inspect.iscoroutine(result):
                            result.close()
                        raise TypeError(
                            "Python plugin handlers must be synchronous; asyncio is not supported"
                        )
                    return json.dumps(_encode(result), allow_nan=False)
                finally:
                    _active.session = previous

            return invoke

        source = f"context => ({self.source}\n)({{...context, options: {json.dumps(self.options, allow_nan=False)}}})"
        return (
            self.name,
            source,
            [(name, adapt(handler)) for name, handler in self.handlers.items()],
        )


def _check_session(session):
    if getattr(_active, "session", None) is session:
        raise Reentry("cannot create pages or close the session from its host callback")


def _bundled(name, options=None):
    from pathlib import Path

    source = (Path(__file__).parent / "bundled" / f"{name}.js").read_text()
    if options is not None:
        source = f"context => ({source})(context, {json.dumps(options, allow_nan=False)})"
    return Plugin("defuddle" if name == "extraction" else name, source)


def canvas(*, rgba=None, character_width=None):
    """Painted Canvas state with profile font metrics and a real PNG encoder."""
    options = {}
    if rgba is not None:
        options["rgba"] = list(rgba)
    if character_width is not None:
        options["characterWidth"] = character_width
    return _bundled("canvas", options)


def environment(**options):
    """Explicit core navigator overrides; Configured connection values."""
    return _bundled("environment", options)


def permissions(**options):
    """Notification and permission mocks; Credential placeholders."""
    return _bundled("permissions", options)


def devices(**options):
    """Bluetooth availability; Clipboard, geolocation, wake lock and sharing placeholders."""
    return _bundled("devices", options)


def media(**options):
    """Media devices, streams and source-buffer placeholders; Media session state and encrypted-media event payloads."""
    return _bundled("media", options)


def workers(**options):
    """Worker and service-worker placeholders."""
    return _bundled("workers", options)


def trusted_types(**options):
    """Trusted Types placeholders."""
    return _bundled("trusted-types", options)


def rtc():
    """Local WebRTC shapes and legacy placeholders; no network/media transport."""
    return _bundled("rtc")


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
                if inspect.iscoroutine(result):
                    result.close()
                raise TypeError("Python callbacks must be synchronous")
            return result
        finally:
            _active.session = previous

    return invoke


def graphics(*, rgba=None, renderer=None, webgpu=False, handlers=None):
    """WebGL state/readback with profile identity and opt-in WebGPU buffer mocks."""
    options = {"webgpu": webgpu}
    if rgba is not None:
        options["rgba"] = list(rgba)
    if renderer is not None:
        options["renderer"] = renderer
    plugin = _bundled("graphics", options)
    return Plugin(plugin.name, plugin.source, handlers or {})


def audio(*, sample_rate=None, handlers=None):
    """Audio graph/buffer state with profile rate; silent analysis, no decoding."""
    options = {} if sample_rate is None else {"sampleRate": sample_rate}
    plugin = _bundled("audio", options)
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
            if kind == "get":
                results.append(values.get(key))
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
                if (
                    not isinstance(op, dict)
                    or op.get("type") not in ("get", "set", "delete", "clear")
                    or (op["type"] != "clear" and not isinstance(op.get("key"), str))
                    or (op["type"] == "set" and "value" not in op)
                ):
                    raise TypeError("invalid storage operation")
            with lock:
                if context.cancelled:
                    raise RuntimeError("Storage request cancelled")
                return backend.transaction(context.origin, operations)

        def dispose():
            with lock:
                close = getattr(backend, "close", None)
                if close:
                    close()

        return {"handlers": {"transaction": transaction}, "dispose": dispose}

    return Plugin("host-storage", _bundled("host-storage").source, setup=setup)


def browser():
    """Fresh default browser subsystem descriptors; excludes host storage and extraction."""
    return [
        _bundled(name, {})
        for name in (
            "environment",
            "permissions",
            "devices",
            "media",
            "workers",
            "trusted-types",
            "canvas",
            "graphics",
            "audio",
            "rtc",
        )
    ]

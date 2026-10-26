"""Synchronous, in-process Session/Page API. Timeouts are in seconds."""
import base64
import inspect
import json
import re
from email.message import Message
from urllib.parse import urlencode

from ._brimp import (
    _Session, RuntimeError, InvalidInput, JavaScriptError, Timeout, Closed,
    StaleHandle, TransportError, Reentry,
)

from .values import encode as _encode, decode as _decode, undefined, BigInt
from .plugins import Plugin, HostContext, _check_session, _callback


def _snake_record(value):
    if isinstance(value, list): return [_snake_record(item) for item in value]
    if isinstance(value, dict): return {re.sub(r"(?<!^)(?=[A-Z])", "_", key).lower(): _snake_record(item) for key, item in value.items()}
    return value


def _event_record(wire):
    record = _snake_record(json.loads(wire))
    if record["event"] == "request" and record["request"]["body"] is not None:
        record["request"]["body"] = bytes(record["request"]["body"])
    if record["event"] == "websocket": record["data"] = bytes(record["data"])
    return record


def _headers(value):
    return list(value.items()) if hasattr(value, "items") else list(value or ())


class _Requests:
    def get(self, url, **kwargs): return self.request("GET", url, **kwargs)
    def post(self, url, **kwargs): return self.request("POST", url, **kwargs)
    def put(self, url, **kwargs): return self.request("PUT", url, **kwargs)
    def patch(self, url, **kwargs): return self.request("PATCH", url, **kwargs)
    def delete(self, url, **kwargs): return self.request("DELETE", url, **kwargs)
    def head(self, url, **kwargs): return self.request("HEAD", url, **kwargs)
    def options(self, url, **kwargs): return self.request("OPTIONS", url, **kwargs)


class Session(_Requests):
    """Own pages, shared cookies, request defaults and explicit plugins.

    Use as a context manager to close all owned pages. Timeouts are seconds.
    Each request method creates a new Page; use Page.request to reuse a page.
    """
    def __init__(self, *, headers=None, timeout=30.0, proxy=None, auth=None, plugins=(), user_agent=None, languages=None, platform=None, time_mode="system", time_origin=None, diagnostics=None):
        pairs = _headers(headers)
        if auth is not None:
            pairs.append(("Authorization", "Basic " + base64.b64encode(
                f"{auth[0]}:{auth[1]}".encode()).decode()))
        self._plugin_token = object()
        self._plugin_disposers = []
        bound = []
        try:
            for plugin in plugins:
                plugin, dispose = plugin._bind()
                bound.append(plugin)
                if dispose: self._plugin_disposers.append(dispose)
            self._inner = _Session(pairs, timeout, proxy, [plugin._descriptor(self._plugin_token) for plugin in bound],
                                   json.dumps(dict(userAgent=user_agent, languages=languages, platform=platform)),
                                   json.dumps(dict(mode=time_mode, epochMs=time_origin), allow_nan=False),
                                   json.dumps(diagnostics, allow_nan=False))
        except BaseException:
            self._dispose_plugins()
            raise

    def _dispose_plugins(self):
        disposers, self._plugin_disposers = self._plugin_disposers, []
        errors = []
        for dispose in reversed(disposers):
            try:
                result = dispose()
                if inspect.isawaitable(result):
                    if inspect.iscoroutine(result): result.close()
                    raise TypeError("Python plugin disposers must be synchronous")
            except BaseException as error: errors.append(error)
        if errors: raise errors[0]


    def new_page(self):
        """Create a blank page before attaching routes, listeners or init scripts."""
        _check_session(self._plugin_token)
        return Page(self._inner.new_page(), self)

    def request(self, method, url, **kwargs):
        """Navigate a new page. Keyword arguments are those of Page.request."""
        page = self.new_page()
        try:
            return page.request(method, url, **kwargs)
        except BaseException:
            page.close()
            raise

    def set_cookie(self, url, name, value): self._inner.set_cookie(url, name, value)
    @property
    def cookies(self): return json.loads(self._inner.cookies_json())
    def clear_cookies(self): self._inner.clear_cookies()
    def close(self):
        _check_session(self._plugin_token)
        try: self._inner.close()
        finally: self._dispose_plugins()
    def __enter__(self): return self
    def __exit__(self, *_): self.close()


class Page(_Requests):
    """A reusable page with a response snapshot and a live browser realm.

    Obtain pages from Session. Navigation replaces the realm and invalidates
    old handles. Closing a page cancels its work; the session also owns cleanup.
    """
    def advance_time(self, milliseconds, *, max_tasks=10000):
        """Advance logical milliseconds, processing tasks in chronological order."""
        return json.loads(self._inner.advance_time(milliseconds, max_tasks))

    def run_pending_tasks(self, *, max_tasks=10000):
        """Drain currently due tasks without advancing logical time."""
        return json.loads(self._inner.run_pending_tasks(max_tasks))

    def take_diagnostics(self):
        """Consume bounded records; JSONL output is unaffected."""
        return _snake_record(json.loads(self._inner.take_diagnostics()))

    def peek_diagnostics(self):
        """Inspect retained records, coverage and loss without consuming them."""
        return _snake_record(json.loads(self._inner.peek_diagnostics()))

    def summarize_diagnostics(self):
        """Group retained observations by layer/path/kind and reflection target.

        Does not consume records. Counts cover the retained, filtered window;
        missing records do not establish that an API was never accessed.
        """
        return _snake_record(json.loads(self._inner.summarize_diagnostics()))

    def __init__(self, inner, session):
        self._inner = inner
        self._session = session
        self.response = None
        self._subscriptions = set()
        self._routes = set()

    def on(self, event, handler):
        """Subscribe a synchronous event handler; return an unsubscribe function."""
        callback = _callback(self._session._plugin_token, lambda wire: handler(_event_record(wire)))
        subscription = self._inner.on(event, callback)
        self._subscriptions.add(subscription)
        def unsubscribe():
            subscription.unsubscribe()
            self._subscriptions.discard(subscription)
        return unsubscribe

    def route(self, pattern, handler, *, stage="request"):
        """Intercept a glob pattern at request, response or auth stage.

        Last matching route wins. Complete the route exactly once. The callback
        must be synchronous and cannot reenter this page. Returns a remover.
        """
        callback = _callback(self._session._plugin_token, lambda inner: handler(Route(inner)))
        registration = self._inner.route(pattern, stage, callback)
        self._routes.add(registration)
        def remove():
            registration.remove()
            self._routes.discard(registration)
        return remove

    def set_identity(self, *, user_agent=None, languages=None, platform=None):
        self._inner.set_identity(json.dumps(dict(userAgent=user_agent, languages=languages, platform=platform)))

    def configure_network(self, *, offline=False, latency=0.0, bypass_cache=False,
                          max_retained_bytes=32*1024*1024, max_retained_body_bytes=8*1024*1024,
                          buffered_download_bytes_per_second=None):
        self._inner.configure_network(offline, latency, bypass_cache, max_retained_bytes,
                                      max_retained_body_bytes, buffered_download_bytes_per_second)

    def set_extra_headers(self, headers): self._inner.set_extra_headers(_headers(headers))
    def clear_cache(self): self._inner.clear_cache()
    def response_body(self, request_id): return self._inner.response_body(request_id)

    def stream(self, method, url, *, headers=None, body=None, timeout=None,
               allow_redirects=True, max_redirects=20, auth=None):
        """Stream response bytes without replacing the DOM. Close abandoned streams."""
        pairs = _headers(headers)
        if auth is not None:
            pairs.append(("Authorization", "Basic " + base64.b64encode(f"{auth[0]}:{auth[1]}".encode()).decode()))
        if isinstance(body, str): body = body.encode()
        return StreamingResponse(self._inner.stream(method, url, pairs, body, timeout, allow_redirects, max_redirects), self)

    def request(self, method, url, *, headers=None, body=None, data=None,
                timeout=None, allow_redirects=True, max_redirects=20, auth=None):
        """Navigate this page and return it with a new response snapshot.

        body accepts strings or bytes; data encodes a form mapping. Use only one.
        timeout is seconds. HTTP error statuses remain accessible to the caller.
        """
        pairs = _headers(headers)
        if body is not None and data is not None:
            raise InvalidInput("specify body or data, not both")
        if data is not None:
            body = urlencode(data, doseq=True).encode()
            if not any(name.lower() == "content-type" for name, _ in pairs):
                pairs.append(("Content-Type", "application/x-www-form-urlencoded"))
        if isinstance(body, str): body = body.encode()
        if auth is not None:
            pairs = [(k, v) for k, v in pairs if k.lower() != "authorization"]
            pairs.append(("Authorization", "Basic " + base64.b64encode(
                f"{auth[0]}:{auth[1]}".encode()).decode()))
        result = self._inner.request(method, url, pairs, body, timeout,
                                     allow_redirects, max_redirects)
        self.response = Response(result)
        return self

    @property
    def status_code(self): return self.response.status_code
    @property
    def content(self): return self.response.content
    @property
    def text(self): return self.response.text
    @property
    def url(self): return self.response.url
    @property
    def headers(self): return self.response.headers
    def json(self): return self.response.json()
    def evaluate(self, source):
        """Evaluate browser JavaScript, await Promises, and copy rich bridge values."""
        return _decode(json.loads(self._inner.evaluate(source)))
    def evaluate_handle(self, source): return JSHandle(self._inner.evaluate_handle(source))
    def query_selector(self, selector):
        inner = self._inner.query_selector(selector)
        return ElementHandle(inner) if inner is not None else None
    def query_selector_all(self, selector):
        return [ElementHandle(inner) for inner in self._inner.query_selector_all(selector)]
    def add_init_script(self, source): self._inner.add_init_script(source)
    def click(self, selector): self._inner.click(selector)
    def hover(self, selector): self._inner.hover(selector)
    def tap(self, selector): self._inner.tap(selector)
    def focus(self, selector): self._inner.focus(selector)
    def type_text(self, selector, text): self._inner.type_text(selector, text)
    def fill(self, selector, text): self._inner.fill(selector, text)
    def press(self, selector, key, *, modifiers=0): self._inner.press(selector, key, modifiers)
    def drag_to(self, source, destination): self._inner.drag_to(source, destination)
    def set_input_files(self, selector, files): self._inner.set_input_files(selector, _files(files))
    def extract(self, *, content_selector=None, remove_images=False, language=None, debug=False):
        return json.loads(self._inner.extract(json.dumps({"contentSelector": content_selector, "removeImages": remove_images, "language": language, "debug": debug})))
    def html(self):
        """Serialize the current DOM, including page-script and host edits."""
        return self._inner.html()
    def close(self): self._inner.close()
    def __enter__(self): return self
    def __exit__(self, *_): self.close()


class Response:
    """Original HTTP response bytes and metadata; created by page navigation.

    headers and cookies are lists of pairs, not mutable session configuration.
    elapsed is seconds. text decodes content using encoding. html is the optional
    navigation-time HTML snapshot; use Page.html() for subsequent DOM edits.
    """
    html: str | None
    status_code: int
    reason: str
    url: str
    headers: list[tuple[str, str]]
    cookies: list[tuple[str, str]]
    history: list[dict]
    request: dict
    elapsed: float
    http_version: str | None
    downloaded_bytes: int
    uploaded_bytes: int
    header_bytes: int
    content: bytes
    encoding: str
    def __init__(self, inner):
        self.__dict__.update(json.loads(inner.metadata_json()))
        self.content = inner.content
        self.request["body"] = None if self.request["body"] is None else bytes(self.request["body"])
        for hop in self.history:
            body = hop["request"]["body"]
            hop["request"]["body"] = None if body is None else bytes(body)
        content_type = Message()
        content_type["content-type"] = next((v for k, v in self.headers if k.lower() == "content-type"), "")
        self.encoding = content_type.get_content_charset() or "utf-8"
        self.elapsed = self.elapsed["secs"] + self.elapsed["nanos"] / 1e9

    @property
    def text(self): return self.content.decode(self.encoding, errors="replace")
    def json(self): return json.loads(self.content)


class JSHandle:
    """Rooted JavaScript value. Dispose explicitly or use a context manager.

    Navigation invalidates handles from the previous realm.
    """
    def __init__(self, inner): self._inner = inner
    def _reference(self): return json.loads(self._inner.reference())
    def _from_reference(self, token): return JSHandle(self._inner.bind_reference(json.dumps(token)))
    def _command(self, **command):
        return _decode(json.loads(self._inner.command(json.dumps(command, allow_nan=False))), self._from_reference)
    def value(self): return self._command(op="value")
    def json_value(self): return json.loads(self._command(op="json"))
    def call(self, function, *args):
        return self._command(op="call", source=function, args=[_encode(arg) for arg in args], handle=True)
    def invoke(self, *args, this=undefined):
        return self._command(op="invoke", args=[_encode(arg) for arg in args], receiver=_encode(this), handle=True)
    def property(self, name): return self._command(op="property", name=name, handle=True)
    def set_property(self, name, value): return self._command(op="set", name=name, args=[_encode(value)])
    def delete_property(self, name): return self._command(op="delete", name=name)
    def property_names(self): return self._command(op="names")
    def dispose(self): self._inner.dispose()
    def __enter__(self): return self
    def __exit__(self, *_): self.dispose()


def _files(files):
    return json.dumps([{"name": file["name"], "bytes": list(file["content"]), "mimeType": file.get("mime_type", ""), "lastModified": file.get("last_modified", 0)} for file in files])


class ElementHandle(JSHandle):
    """Rooted DOM node with query, edit and input methods. Detachment keeps identity."""
    def focus(self): self._inner.focus()
    def click(self): self._inner.click()
    def set_input_files(self, files): self._inner.set_input_files(_files(files))
    def _value(self, function, *args):
        with self.call(function, *args) as handle:
            return handle.json_value()
    def get_attribute(self, name): return self._value("(node, name) => node.getAttribute(name)", name)
    def set_attribute(self, name, value): self._value("(node, name, value) => { node.setAttribute(name, value); return null; }", name, value)
    def text_content(self): return self._value("node => node.textContent")
    def html(self): return self._value("node => node.outerHTML")
    def set_text_content(self, text): self._value("(node, text) => { node.textContent = text; return null; }", text)
    def remove(self): self._value("node => { node.remove(); return null; }")
    def query_selector(self, selector):
        inner = self._inner.query_selector(selector)
        return ElementHandle(inner) if inner is not None else None
    def query_selector_all(self, selector):
        return [ElementHandle(inner) for inner in self._inner.query_selector_all(selector)]


class Route:
    """One intercepted request. Choose exactly one completion method.

    Response-stage interception is buffered, with a 32 MiB limit.
    """
    stage: str
    request: dict
    response: dict | None
    auth_challenge: str | None
    def __init__(self, inner):
        self._inner = inner
        self.__dict__.update(_snake_record(json.loads(inner.info_json())))
        if self.request["body"] is not None: self.request["body"] = bytes(self.request["body"])
    @property
    def pending(self): return self._inner.pending
    def body(self): return self._inner.body()
    def continue_request(self, *, url=None, method=None, headers=None, body=None, status=None):
        overrides = {key: value for key, value in {"url": url, "method": method, "headers": None if headers is None else _headers(headers), "body": None if body is None else list(body.encode() if isinstance(body, str) else body), "status": status}.items() if value is not None}
        self._inner.continue_request(json.dumps(overrides))
    def fulfill(self, *, status=200, headers=None, body=b""):
        self._inner.fulfill(status, _headers(headers), body.encode() if isinstance(body, str) else body)
    def fail(self, message="request aborted by route"): self._inner.fail(message)
    def authenticate(self, username, password): self._inner.authenticate(username, password)


class StreamingResponse:
    """A bounded chunk iterator. Close it when abandoning a response body."""
    status_code: int
    url: str
    headers: list[tuple[str, str]]
    def __init__(self, inner, page):
        self._inner, self._page = inner, page
        metadata = json.loads(inner.metadata_json())
        self.status_code, self.url, self.headers = metadata["status"], metadata["url"], metadata["headers"]
    def __iter__(self): return self
    def __next__(self):
        chunk = self._inner.next_chunk()
        if chunk is None: raise StopIteration
        return chunk
    def close(self): self._inner.cancel()
    def __enter__(self): return self
    def __exit__(self, *_): self.close()

"""Synchronous and asynchronous, in-process Session/Page APIs. Timeouts are in seconds."""

import base64
import inspect
import json as _json
import re
import weakref
from email.message import Message

from ._body import encode_body as _encode_body
from ._timeouts import UNSET as _TIMEOUT_UNSET, normalize_timeout as _normalize_timeout
from .fingerprints import Fingerprint
from .fingerprints import profile_json as _profile_json
from . import exceptions
from .exceptions import (
    BrimpError,
    RuntimeError,
    InvalidInput,
    JavaScriptError,
    Timeout,
    Closed,
    Reentry,
    StaleHandle,
    RequestException,
    RequestsError,
    TransportError,
    ConnectionError,
    DNSError,
    ProxyError,
    SSLError,
    CertificateVerifyError,
    RequestTimeout,
    HTTPError,
    IncompleteRead,
    TooManyRedirects,
    SessionClosed,
    ImpersonateError,
    InvalidURL,
    MissingSchema,
    InvalidSchema,
    InvalidProxyURL,
    InvalidHeader,
    InterfaceError,
    InvalidJSONError,
    JSONDecodeError,
)
from ._brimp import _Session

from .values import encode as _encode, decode as _decode, undefined, BigInt
from .plugins import Plugin, HostContext, _check_session, _callback, browser as _browser_plugins
from ._urls import (
    build_url as _build_url,
    validate_base_url as _validate_base_url,
    validate_request_url as _validate_request_url,
)


def _snake_record(value):
    if isinstance(value, list):
        return [_snake_record(item) for item in value]
    if isinstance(value, dict):
        return {
            re.sub(r"(?<!^)(?=[A-Z])", "_", key).lower(): _snake_record(item)
            for key, item in value.items()
        }
    return value


def _event_record(wire):
    record = _snake_record(_json.loads(wire))
    if record["event"] == "request" and record["request"]["body"] is not None:
        record["request"]["body"] = bytes(record["request"]["body"])
    if record["event"] == "websocket":
        record["data"] = bytes(record["data"])
    return record


def _headers(value):
    return list(value.items()) if hasattr(value, "items") else list(value or ())


def _proxy_options(proxy=None, proxies=None, proxy_auth=None, trust_env=None):
    if proxy is not None and proxies is not None:
        raise InvalidInput("specify proxy or proxies, not both")
    return {
        key: value
        for key, value in {
            "proxy": proxy,
            "proxies": proxies,
            "proxyAuth": proxy_auth,
            "trustEnv": trust_env,
        }.items()
        if value is not None
    }


class _Requests:
    def get(self, url, **kwargs):
        return self.request("GET", url, **kwargs)

    def post(self, url, **kwargs):
        return self.request("POST", url, **kwargs)

    def put(self, url, **kwargs):
        return self.request("PUT", url, **kwargs)

    def patch(self, url, **kwargs):
        return self.request("PATCH", url, **kwargs)

    def delete(self, url, **kwargs):
        return self.request("DELETE", url, **kwargs)

    def head(self, url, **kwargs):
        return self.request("HEAD", url, **kwargs)

    def options(self, url, **kwargs):
        return self.request("OPTIONS", url, **kwargs)

    def trace(self, url, **kwargs):
        return self.request("TRACE", url, **kwargs)

    def query(self, url, **kwargs):
        return self.request("QUERY", url, **kwargs)


class Session(_Requests):
    """Own pages, shared cookies, request defaults and browser subsystem plugins.

    Omit plugins for the browser defaults; an explicit list replaces that set.
    Pass plugins=[] for core-only execution.

    Use as a context manager to close all owned pages. timeout accepts numeric seconds,
    or None/zero to disable the deadline.
    The default is 30 seconds; requests inherit it unless explicitly overridden.
    Each request method creates a new Page; use Page.request to reuse a page.
    base_url resolves relative request URLs. params supplies default query values;
    both attributes can be changed before subsequent requests.
    proxy or proxies sets default proxy routing; proxy_auth supplies credentials.
    trust_env controls environment proxies. proxy="" forces direct connections.
    """

    def __init__(
        self,
        *,
        headers=None,
        base_url=None,
        params=None,
        timeout=30,
        proxy=None,
        proxies=None,
        proxy_auth=None,
        trust_env=True,
        auth=None,
        plugins=None,
        user_agent=None,
        languages=None,
        platform=None,
        profile=None,
        time_mode="system",
        time_origin=None,
        diagnostics=None,
        storage_path=None,
    ):
        if isinstance(profile, Fingerprint):
            profile = {"values": {"network": {"fingerprint": profile}}}
        _validate_base_url(base_url)
        self.base_url = base_url
        self.params = {} if params is None else params
        pairs = _headers(headers)
        if auth is not None:
            pairs.append(
                (
                    "Authorization",
                    "Basic " + base64.b64encode(f"{auth[0]}:{auth[1]}".encode()).decode(),
                )
            )
        self._default_headers = tuple(pairs)
        self._plugin_token = object()
        self._plugin_disposers = []
        self._routes = set()
        bound = []
        try:
            for plugin in _browser_plugins() if plugins is None else plugins:
                plugin, dispose = plugin._bind()
                bound.append(plugin)
                if dispose:
                    self._plugin_disposers.append(dispose)
            self._inner = _Session(
                pairs,
                _normalize_timeout(timeout),
                [plugin._descriptor(self._plugin_token) for plugin in bound],
                _json.dumps(dict(userAgent=user_agent, languages=languages, platform=platform)),
                _json.dumps(profile, allow_nan=False, default=_profile_json),
                _json.dumps(dict(mode=time_mode, epochMs=time_origin), allow_nan=False),
                _json.dumps(diagnostics, allow_nan=False),
                None if storage_path is None else str(storage_path),
                _json.dumps(_proxy_options(proxy, proxies, proxy_auth, trust_env)),
            )
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
                    if inspect.iscoroutine(result):
                        result.close()
                    raise TypeError("Python plugin disposers must be synchronous")
            except BaseException as error:
                errors.append(error)
        if errors:
            raise errors[0]

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

    def route(self, pattern, handler, *, stage="request"):
        """Register a synchronous route on all existing and future pages.

        Matching page routes take precedence. Returns a removal function.
        """
        _check_session(self._plugin_token)
        callback = _callback(self._plugin_token, lambda inner: handler(Route(inner)))
        registration = self._inner.route(pattern, stage, callback)
        self._routes.add(registration)

        def remove():
            registration.remove()
            self._routes.discard(registration)

        return remove

    def set_cookie(self, url, name, value):
        self._inner.set_cookie(url, name, value)

    @property
    def cookies(self):
        return _json.loads(self._inner.cookies_json())

    def clear_cookies(self):
        self._inner.clear_cookies()

    def close(self):
        _check_session(self._plugin_token)
        try:
            self._inner.close()
        finally:
            self._routes.clear()
            self._dispose_plugins()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()


class _HTTPResult:
    """Buffered response access shared by synchronous and asynchronous pages."""

    @property
    def status_code(self):
        return self._http_response().status_code

    @property
    def reason(self):
        return self._http_response().reason

    @property
    def content(self):
        return self._http_response().content

    @property
    def text(self):
        return self._http_response().text

    @property
    def url(self):
        return self._http_response().url

    @property
    def headers(self):
        return self._http_response().headers

    @property
    def cookies(self):
        return self._http_response().cookies

    @property
    def history(self):
        return self._http_response().history

    @property
    def request_info(self):
        """The original HTTP request record; request() navigates the page."""
        return self._http_response().request

    @property
    def elapsed(self):
        return self._http_response().elapsed

    @property
    def http_version(self):
        return self._http_response().http_version

    @property
    def downloaded_bytes(self):
        return self._http_response().downloaded_bytes

    @property
    def uploaded_bytes(self):
        return self._http_response().uploaded_bytes

    @property
    def header_bytes(self):
        return self._http_response().header_bytes

    @property
    def encoding(self):
        """Encoding used by text; override it to change response decoding."""
        return self._http_response().encoding

    @encoding.setter
    def encoding(self, value):
        self._http_response().encoding = value

    def json(self):
        """Decode the original HTTP response body as JSON."""
        try:
            return self._http_response().json()
        except _json.JSONDecodeError as error:
            raise JSONDecodeError(error.msg, error.doc, error.pos, response=self) from error

    def raise_for_status(self):
        """Raise HTTPError for a 4xx/5xx response; otherwise return None.

        HTTP statuses do not raise automatically during navigation. A page
        without an HTTP response raises InvalidInput.
        """
        if 400 <= self.status_code < 600:
            raise HTTPError(
                f"HTTP {self.status_code} {self.reason} for url: {self.url}", response=self
            )


class Page(_Requests, _HTTPResult):
    """An HTTP result and its live DOM and JavaScript environment.

    Obtain pages from Session. Navigation replaces the realm and invalidates
    old handles, and a successful request replaces the HTTP metadata. text and
    content contain the original response; html() serializes the current DOM.
    HTTP properties require a response: new_page() and load_html() have none.
    Headers and response cookies are lists of pairs. elapsed is seconds.
    Closing a page cancels its work; the session also owns cleanup. Pages returned
    by module-level helpers own a private session and close it on page closure.
    """

    def advance_time(self, milliseconds, *, max_tasks=10000):
        """Advance logical milliseconds, processing tasks in chronological order."""
        return _json.loads(self._inner.advance_time(milliseconds, max_tasks))

    def run_pending_tasks(self, *, max_tasks=10000):
        """Drain currently due tasks without advancing logical time."""
        return _json.loads(self._inner.run_pending_tasks(max_tasks))

    def profile_contract(self):
        """Describe installed profile interfaces, preserving their original field names."""
        return _json.loads(self._inner.profile_contract())

    def profile_report(self):
        """Return defaulted paths and capture metadata without renaming profile keys."""
        return _json.loads(self._inner.profile_report())

    def take_diagnostics(self):
        """Consume bounded records; JSONL output is unaffected."""
        return _snake_record(_json.loads(self._inner.take_diagnostics()))

    def peek_diagnostics(self):
        """Inspect retained records, coverage and loss without consuming them."""
        return _snake_record(_json.loads(self._inner.peek_diagnostics()))

    def summarize_diagnostics(self):
        """Group retained observations by layer/path/kind and reflection target.

        Does not consume records. Counts cover the retained, filtered window;
        missing records do not establish that an API was never accessed.
        """
        return _snake_record(_json.loads(self._inner.summarize_diagnostics()))

    def __init__(self, inner, session):
        self._inner = inner
        self._session = session
        self._response = None
        self._session_cleanup = None
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

    def load_html(self, html, url="about:blank"):
        """Load HTML at *url* without running its page scripts.

        The URL becomes the browsing-context URL before parsing, so relative
        URLs, document.URL and location.href use it. Subresources are not
        fetched. Init scripts still run.
        """
        result = self._inner.load_html(html, url)
        self._response = None
        return result

    def set_identity(self, *, user_agent=None, languages=None, platform=None):
        self._inner.set_identity(
            _json.dumps(dict(userAgent=user_agent, languages=languages, platform=platform))
        )

    def configure_network(
        self,
        *,
        offline=False,
        latency=0.0,
        bypass_cache=False,
        max_retained_bytes=32 * 1024 * 1024,
        max_retained_body_bytes=8 * 1024 * 1024,
        buffered_download_bytes_per_second=None,
    ):
        self._inner.configure_network(
            offline,
            latency,
            bypass_cache,
            max_retained_bytes,
            max_retained_body_bytes,
            buffered_download_bytes_per_second,
        )

    def set_extra_headers(self, headers):
        self._inner.set_extra_headers(_headers(headers))

    def clear_cache(self):
        self._inner.clear_cache()

    def response_body(self, request_id):
        return self._inner.response_body(request_id)

    def stream(
        self,
        method,
        url,
        *,
        params=None,
        quote="",
        headers=None,
        body=None,
        data=None,
        content=None,
        json=None,
        timeout=_TIMEOUT_UNSET,
        allow_redirects=True,
        max_redirects=20,
        auth=None,
        proxy=None,
        proxies=None,
        proxy_auth=None,
        trust_env=None,
    ):
        """Stream response bytes without replacing the DOM. Close abandoned streams.

        Body arguments follow request(): json, content, data or standalone body.
        Uploads are buffered; file-like, iterable and multipart uploads are unsupported.
        """
        url = self._request_url(url, params, quote)
        pairs = _headers(headers)
        body = _encode_body(
            body=body,
            data=data,
            content=content,
            json=json,
            headers=pairs,
            default_headers=self._session._default_headers,
        )
        if auth is not None:
            pairs.append(
                (
                    "Authorization",
                    "Basic " + base64.b64encode(f"{auth[0]}:{auth[1]}".encode()).decode(),
                )
            )
        return StreamingResponse(
            self._inner.stream(
                method,
                url,
                pairs,
                body,
                _normalize_timeout(timeout),
                allow_redirects,
                max_redirects,
                _json.dumps(_proxy_options(proxy, proxies, proxy_auth, trust_env)),
            ),
            self,
        )

    def request(
        self,
        method,
        url,
        *,
        params=None,
        quote="",
        headers=None,
        body=None,
        data=None,
        content=None,
        json=None,
        timeout=_TIMEOUT_UNSET,
        allow_redirects=True,
        max_redirects=20,
        auth=None,
        proxy=None,
        proxies=None,
        proxy_auth=None,
        trust_env=None,
    ):
        """Navigate this page and return it with a new response snapshot.

        json encodes a JSON value; content sends raw strings/bytes; data sends
        strings/bytes unchanged or URL-encodes a form mapping/sequence of pairs.
        Non-None json takes precedence over content, then data. body accepts raw
        strings/bytes and cannot be combined with those arguments. Explicit
        Content-Type headers override defaults. File/multipart uploads are unsupported.
        timeout accepts numeric seconds as one operation budget. None/zero disables it;
        omission inherits the session default. This is one navigation deadline, including JavaScript, not
        separate connection/read timers. HTTP error statuses remain accessible.
        params merges query values with session defaults. Relative URLs use the
        session's base_url. quote names extra characters to percent-encode;
        quote=False disables automatic URL quoting.
        Proxy overrides apply to this navigation and its redirects; subresources
        use session settings and routes. proxy="" forces a direct connection.
        """
        url = self._request_url(url, params, quote)
        pairs = _headers(headers)
        body = _encode_body(
            body=body,
            data=data,
            content=content,
            json=json,
            headers=pairs,
            default_headers=self._session._default_headers,
        )
        if auth is not None:
            pairs = [(k, v) for k, v in pairs if k.lower() != "authorization"]
            pairs.append(
                (
                    "Authorization",
                    "Basic " + base64.b64encode(f"{auth[0]}:{auth[1]}".encode()).decode(),
                )
            )
        result = self._inner.request(
            method,
            url,
            pairs,
            body,
            _normalize_timeout(timeout),
            allow_redirects,
            max_redirects,
            _json.dumps(_proxy_options(proxy, proxies, proxy_auth, trust_env)),
        )
        self._response = _Response(result)
        return self

    def _request_url(self, url, params, quote):
        try:
            url = _build_url(
                url,
                base_url=self._session.base_url,
                default_params=self._session.params,
                params=params,
                quote=quote,
            )
            return _validate_request_url(url)
        except InvalidURL:
            raise
        except ValueError as error:
            raise InvalidURL(str(error)) from error

    def _http_response(self):
        if self._response is None:
            raise InvalidInput("page has no HTTP response")
        return self._response

    def evaluate(self, source):
        """Evaluate browser JavaScript, await Promises, and copy rich bridge values."""
        return _decode(_json.loads(self._inner.evaluate(source)))

    def evaluate_handle(self, source):
        return JSHandle(self._inner.evaluate_handle(source))

    def query_selector(self, selector):
        inner = self._inner.query_selector(selector)
        return ElementHandle(inner) if inner is not None else None

    def query_selector_all(self, selector):
        return [ElementHandle(inner) for inner in self._inner.query_selector_all(selector)]

    def add_init_script(self, source):
        self._inner.add_init_script(source)

    def click(self, selector):
        self._inner.click(selector)

    def hover(self, selector):
        self._inner.hover(selector)

    def tap(self, selector):
        self._inner.tap(selector)

    def focus(self, selector):
        self._inner.focus(selector)

    def type_text(self, selector, text):
        self._inner.type_text(selector, text)

    def fill(self, selector, text):
        self._inner.fill(selector, text)

    def press(self, selector, key, *, modifiers=0):
        self._inner.press(selector, key, modifiers)

    def drag_to(self, source, destination):
        self._inner.drag_to(source, destination)

    def set_input_files(self, selector, files):
        self._inner.set_input_files(selector, _files(files))

    def extract(self, *, content_selector=None, remove_images=False, language=None, debug=False):
        return _json.loads(
            self._inner.extract(
                _json.dumps(
                    {
                        "contentSelector": content_selector,
                        "removeImages": remove_images,
                        "language": language,
                        "debug": debug,
                    }
                )
            )
        )

    def html(self):
        """Serialize the current DOM, including page-script and host edits."""
        return self._inner.html()

    def close(self):
        self._inner.close()
        if self._session_cleanup is not None:
            self._session_cleanup()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()


def request(method, url, **kwargs):
    """Request a live Page using a private session.

    Keyword arguments are those of Page.request. Close the page or use it as a
    context manager to release both page and session. Use Session directly for
    session configuration or cookies shared between multiple pages.
    """
    session = Session()
    try:
        page = session.request(method, url, **kwargs)
        page._session_cleanup = weakref.finalize(page, session.close)
        return page
    except BaseException:
        session.close()
        raise


def get(url, **kwargs):
    """GET a live Page. Accepts the keyword arguments of Page.request."""
    return request("GET", url, **kwargs)


def head(url, **kwargs):
    """HEAD a live Page. Accepts the keyword arguments of Page.request."""
    return request("HEAD", url, **kwargs)


def post(url, **kwargs):
    """POST a live Page. Accepts the keyword arguments of Page.request."""
    return request("POST", url, **kwargs)


def put(url, **kwargs):
    """PUT a live Page. Accepts the keyword arguments of Page.request."""
    return request("PUT", url, **kwargs)


def patch(url, **kwargs):
    """PATCH a live Page. Accepts the keyword arguments of Page.request."""
    return request("PATCH", url, **kwargs)


def delete(url, **kwargs):
    """DELETE a live Page. Accepts the keyword arguments of Page.request."""
    return request("DELETE", url, **kwargs)


def options(url, **kwargs):
    """OPTIONS a live Page. Accepts the keyword arguments of Page.request."""
    return request("OPTIONS", url, **kwargs)


def trace(url, **kwargs):
    """TRACE a live Page. Accepts the keyword arguments of Page.request."""
    return request("TRACE", url, **kwargs)


def query(url, **kwargs):
    """QUERY a live Page. Accepts the keyword arguments of Page.request."""
    return request("QUERY", url, **kwargs)


class _Response:
    """Private HTTP snapshot, replaced after each successful page request."""

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
        self.__dict__.update(_json.loads(inner.metadata_json()))
        self.content = inner.content
        self.request["body"] = None if self.request["body"] is None else bytes(self.request["body"])
        for hop in self.history:
            body = hop["request"]["body"]
            hop["request"]["body"] = None if body is None else bytes(body)
        content_type = Message()
        content_type["content-type"] = next(
            (v for k, v in self.headers if k.lower() == "content-type"), ""
        )
        self.encoding = content_type.get_content_charset() or "utf-8"
        self.elapsed = self.elapsed["secs"] + self.elapsed["nanos"] / 1e9

    @property
    def text(self):
        return self.content.decode(self.encoding, errors="replace")

    def json(self):
        return _json.loads(self.content)


class JSHandle:
    """Rooted JavaScript value. Dispose explicitly or use a context manager.

    Navigation invalidates handles from the previous realm.
    """

    def __init__(self, inner):
        self._inner = inner

    def _reference(self):
        return _json.loads(self._inner.reference())

    def _from_reference(self, token):
        return JSHandle(self._inner.bind_reference(_json.dumps(token)))

    def _command(self, **command):
        return _decode(
            _json.loads(self._inner.command(_json.dumps(command, allow_nan=False))),
            self._from_reference,
        )

    def value(self):
        return self._command(op="value")

    def json_value(self):
        return _json.loads(self._command(op="json"))

    def call(self, function, *args):
        return self._command(
            op="call", source=function, args=[_encode(arg) for arg in args], handle=True
        )

    def invoke(self, *args, this=undefined):
        return self._command(
            op="invoke", args=[_encode(arg) for arg in args], receiver=_encode(this), handle=True
        )

    def property(self, name):
        return self._command(op="property", name=name, handle=True)

    def set_property(self, name, value):
        return self._command(op="set", name=name, args=[_encode(value)])

    def delete_property(self, name):
        return self._command(op="delete", name=name)

    def property_names(self):
        return self._command(op="names")

    def dispose(self):
        self._inner.dispose()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.dispose()


def _files(files):
    return _json.dumps(
        [
            {
                "name": file["name"],
                "bytes": list(file["content"]),
                "mimeType": file.get("mime_type", ""),
                "lastModified": file.get("last_modified", 0),
            }
            for file in files
        ]
    )


class ElementHandle(JSHandle):
    """Rooted DOM node with query, edit and input methods. Detachment keeps identity."""

    def focus(self):
        self._inner.focus()

    def click(self):
        self._inner.click()

    def set_input_files(self, files):
        self._inner.set_input_files(_files(files))

    def _value(self, function, *args):
        with self.call(function, *args) as handle:
            return handle.json_value()

    def get_attribute(self, name):
        return self._value("(node, name) => node.getAttribute(name)", name)

    def set_attribute(self, name, value):
        self._value(
            "(node, name, value) => { node.setAttribute(name, value); return null; }", name, value
        )

    def text_content(self):
        return self._value("node => node.textContent")

    def html(self):
        return self._value("node => node.outerHTML")

    def set_text_content(self, text):
        self._value("(node, text) => { node.textContent = text; return null; }", text)

    def remove(self):
        self._value("node => { node.remove(); return null; }")

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
        self.__dict__.update(_snake_record(_json.loads(inner.info_json())))
        if self.request["body"] is not None:
            self.request["body"] = bytes(self.request["body"])

    @property
    def pending(self):
        return self._inner.pending

    def body(self):
        return self._inner.body()

    def continue_request(
        self,
        *,
        url=None,
        method=None,
        headers=None,
        body=None,
        status=None,
        proxy=None,
        proxy_auth=None,
    ):
        """Continue once, optionally changing request fields or its proxy.

        proxy=None inherits; proxy="" connects directly. Proxy overrides are
        request-stage only. Supply proxy_auth when changing authenticated proxies.
        """
        overrides = {
            key: value
            for key, value in {
                "url": url,
                "method": method,
                "headers": None if headers is None else _headers(headers),
                "body": None
                if body is None
                else list(body.encode() if isinstance(body, str) else body),
                "status": status,
            }.items()
            if value is not None
        }
        overrides.update(_proxy_options(proxy=proxy, proxy_auth=proxy_auth))
        self._inner.continue_request(_json.dumps(overrides))

    def fulfill(self, *, status=200, headers=None, body=b""):
        self._inner.fulfill(
            status, _headers(headers), body.encode() if isinstance(body, str) else body
        )

    def fail(self, message="request aborted by route"):
        self._inner.fail(message)

    def authenticate(self, username, password):
        self._inner.authenticate(username, password)


class StreamingResponse:
    """A bounded chunk iterator. Close it when abandoning a response body."""

    status_code: int
    url: str
    headers: list[tuple[str, str]]

    def __init__(self, inner, page):
        self._inner, self._page = inner, page
        metadata = _json.loads(inner.metadata_json())
        self.status_code, self.url, self.headers = (
            metadata["status"],
            metadata["url"],
            metadata["headers"],
        )

    def __iter__(self):
        return self

    def __next__(self):
        chunk = self._inner.next_chunk()
        if chunk is None:
            raise StopIteration
        return chunk

    def close(self):
        self._inner.cancel()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()


from ._async import AsyncSession, AsyncPage, AsyncJSHandle, AsyncElementHandle

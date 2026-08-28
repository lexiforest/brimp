import json as _json
import re as _re
from collections.abc import Iterable, Mapping
from pathlib import Path
from urllib.parse import urlencode, urlsplit, urlunsplit

from ._brimp import _Session


class BrimpError(OSError):
    code = "internal"

    def __init__(self, message: str, *, code: str | None = None):
        super().__init__(message)
        if code is not None:
            self.code = code


class ConnectionError(BrimpError):
    code = "transport"


class Timeout(BrimpError):
    code = "timeout"


class TooManyRedirects(ConnectionError):
    code = "too_many_redirects"


class InvalidRequest(BrimpError):
    code = "invalid_input"


class InvalidURL(InvalidRequest):
    pass


class HTTPError(BrimpError):
    code = "http_status"

    def __init__(self, message: str, *, response):
        super().__init__(message)
        self.response = response


class JavaScriptError(BrimpError):
    code = "javascript"


def _translate(error: RuntimeError) -> BrimpError:
    message = str(error)
    if not message.startswith("brimp ") or ": " not in message:
        return BrimpError(message)
    code, detail = message.removeprefix("brimp ").split(": ", 1)
    if code == "transport":
        if "redirect limit" in detail:
            return TooManyRedirects(detail)
        return ConnectionError(detail)
    if code == "timeout":
        return Timeout(detail)
    if code == "invalid_input":
        return InvalidURL(detail) if "URL" in detail or "url" in detail else InvalidRequest(detail)
    if code == "javascript":
        return JavaScriptError(detail)
    return BrimpError(detail, code=code)


class Headers(Mapping):
    def __init__(self, entries: Iterable[tuple[str, str]] = ()):
        self._entries = tuple((str(name), str(value)) for name, value in entries)
        self._values = {}
        self._names = {}
        for name, value in self._entries:
            key = name.lower()
            self._names.setdefault(key, name)
            self._values.setdefault(key, []).append(value)

    def __getitem__(self, name):
        return ", ".join(self._values[str(name).lower()])

    def __iter__(self):
        return iter(self._names.values())

    def __len__(self):
        return len(self._names)

    def get_all(self, name):
        return tuple(self._values.get(str(name).lower(), ()))

    @property
    def raw(self):
        return self._entries


class Response:
    def __init__(self, native):
        self.status_code = native.status_code
        self.reason = native.reason
        self.url = native.url
        self.headers = Headers(native.headers)
        self.content = bytes(native.content)
        self.html = native.html
        self.cookies = dict(native.cookies)
        self.elapsed = native.elapsed

    @property
    def ok(self):
        return self.status_code < 400

    @property
    def encoding(self):
        content_type = self.headers.get("content-type", "")
        match = _re.search(r"(?:^|;)\s*charset=([^;\s]+)", content_type, _re.IGNORECASE)
        return match.group(1).strip("\"'") if match else "utf-8"

    @property
    def text(self):
        try:
            return self.content.decode(self.encoding, errors="replace")
        except LookupError:
            return self.content.decode("utf-8", errors="replace")

    def json(self):
        return _json.loads(self.text)

    def raise_for_status(self):
        if self.status_code >= 400:
            raise HTTPError(
                f"{self.status_code} {self.reason} for url: {self.url}",
                response=self,
            )

    def __repr__(self):
        return f"<Response [{self.status_code}]>"


class Session:
    _PROTECTED_HEADERS = {"user-agent", "accept-language"}

    def __init__(
        self,
        *,
        persona_json: str | None = None,
        ca_bundle=None,
        enable_worker: bool = False,
        enable_streaming_networking: bool = False,
        storage_path=None,
        storage_quota_bytes: int | None = None,
    ):
        try:
            self._inner = _Session(
                persona_json,
                None if ca_bundle is None else str(Path(ca_bundle)),
                bool(enable_worker),
                bool(enable_streaming_networking),
                None if storage_path is None else str(Path(storage_path)),
                storage_quota_bytes,
            )
        except RuntimeError as error:
            raise _translate(error) from error
        self.headers = {}
        self.cookies = {}
        self._closed = False

    def get(
        self,
        url: str,
        *,
        params=None,
        headers=None,
        cookies=None,
        timeout: float = 30.0,
    ) -> Response:
        self._ensure_open()
        timeout = float(timeout)
        if timeout <= 0:
            raise InvalidRequest("timeout must be positive")
        url = _add_params(str(url), params)
        merged_headers = dict(self.headers)
        if headers:
            merged_headers.update(headers)
        protected = self._PROTECTED_HEADERS.intersection(
            str(name).lower() for name in merged_headers
        )
        if protected:
            names = ", ".join(sorted(protected))
            raise InvalidRequest(
                f"persona-owned headers cannot be overridden: {names}; configure a persona instead"
            )
        merged_cookies = dict(self.cookies)
        if cookies:
            merged_cookies.update(cookies)
        if merged_cookies:
            merged_headers["Cookie"] = "; ".join(
                f"{name}={value}" for name, value in merged_cookies.items()
            )
        native_headers = [(str(name), str(value)) for name, value in merged_headers.items()]
        try:
            timeout_ms = max(1, round(timeout * 1000))
            native = self._inner.get(url, timeout_ms, native_headers)
        except RuntimeError as error:
            raise _translate(error) from error
        response = Response(native)
        self.cookies.update(response.cookies)
        return response

    def evaluate(self, expression: str):
        self._ensure_open()
        try:
            return _json.loads(self._inner.evaluate(str(expression)))
        except RuntimeError as error:
            raise _translate(error) from error

    def screenshot(self, path=None, *, full_page: bool = False):
        self._ensure_open()
        try:
            content = bytes(self._inner.screenshot(bool(full_page)))
        except RuntimeError as error:
            raise _translate(error) from error
        if path is not None:
            Path(path).write_bytes(content)
        return content

    def close(self):
        if not self._closed:
            self._inner.close()
            self._closed = True

    def _ensure_open(self):
        if self._closed:
            raise BrimpError("session is closed", code="closed")

    def __enter__(self):
        self._ensure_open()
        return self

    def __exit__(self, _type, _value, _traceback):
        self.close()


def _add_params(url: str, params) -> str:
    if not params:
        return url
    parts = urlsplit(url)
    query = "&".join(filter(None, (parts.query, urlencode(params, doseq=True))))
    return urlunsplit((parts.scheme, parts.netloc, parts.path, query, parts.fragment))


def get(url: str, **kwargs) -> Response:
    session_options = {
        name: kwargs.pop(name)
        for name in (
            "persona_json",
            "ca_bundle",
            "enable_worker",
            "enable_streaming_networking",
            "storage_path",
            "storage_quota_bytes",
        )
        if name in kwargs
    }
    with Session(**session_options) as session:
        return session.get(url, **kwargs)


__all__ = [
    "BrimpError",
    "ConnectionError",
    "Headers",
    "HTTPError",
    "InvalidRequest",
    "InvalidURL",
    "JavaScriptError",
    "Response",
    "Session",
    "Timeout",
    "TooManyRedirects",
    "get",
]

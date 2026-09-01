import base64 as _base64
import json as _json
import math as _math
import os as _os
import re as _re
import secrets as _secrets
import threading as _threading
from collections.abc import Mapping, MutableMapping
from dataclasses import dataclass
from pathlib import Path
from urllib.parse import urlencode, urlsplit, urlunsplit

from ._brimp import _Session


class RequestError(OSError):
    code = "internal"

    def __init__(self, message: str, *, code: str | None = None):
        super().__init__(message)
        if code is not None:
            self.code = code


class ConnectionError(RequestError):
    code = "transport"


class ProxyError(ConnectionError):
    pass


class SSLError(ConnectionError):
    pass


class Timeout(RequestError):
    code = "timeout"


class TooManyRedirects(ConnectionError):
    code = "too_many_redirects"


class InvalidRequest(RequestError):
    code = "invalid_input"


class InvalidURL(InvalidRequest):
    pass


class InvalidHeader(InvalidRequest):
    pass


class InvalidProxyURL(InvalidRequest):
    pass


class CookieConflict(RequestError):
    code = "cookie_conflict"


class HTTPError(RequestError):
    code = "http_status"

    def __init__(self, message: str, *, page):
        super().__init__(message)
        self.page = page


class JavaScriptError(RequestError):
    code = "javascript"


class SessionClosed(RequestError):
    code = "closed"


class PageReleased(RequestError):
    code = "released"


def _translate(error: RuntimeError) -> RequestError:
    message = str(error)
    if not message.startswith("brimp ") or ": " not in message:
        return RequestError(message)
    code, detail = message.removeprefix("brimp ").split(": ", 1)
    if code == "transport":
        lowered = detail.lower()
        if "redirect limit" in lowered:
            return TooManyRedirects(detail)
        if "proxy" in lowered:
            return ProxyError(detail)
        if "ssl" in lowered or "certificate" in lowered:
            return SSLError(detail)
        return ConnectionError(detail)
    if code == "timeout":
        return Timeout(detail)
    if code == "invalid_input":
        lowered = detail.lower()
        if "proxy" in lowered:
            return InvalidProxyURL(detail)
        if "header" in lowered:
            return InvalidHeader(detail)
        return InvalidURL(detail) if "url" in lowered else InvalidRequest(detail)
    if code == "javascript":
        return JavaScriptError(detail)
    if code == "closed":
        return SessionClosed(detail)
    return RequestError(detail, code=code)


class Headers(MutableMapping):
    """A case-insensitive, insertion-ordered HTTP multi-dict."""

    def __init__(self, entries=()):
        if isinstance(entries, Headers):
            entries = entries.multi_items()
        elif isinstance(entries, Mapping):
            entries = entries.items()
        self._entries = []
        for name, value in entries or ():
            if value is not None:
                self._entries.append((str(name), str(value)))

    def __getitem__(self, name):
        values = self.get_all(name)
        if not values:
            raise KeyError(name)
        return ", ".join(values)

    def __setitem__(self, name, value):
        key = str(name).lower()
        self._entries = [entry for entry in self._entries if entry[0].lower() != key]
        if value is not None:
            self._entries.append((str(name), str(value)))

    def __delitem__(self, name):
        key = str(name).lower()
        before = len(self._entries)
        self._entries = [entry for entry in self._entries if entry[0].lower() != key]
        if len(self._entries) == before:
            raise KeyError(name)

    def __iter__(self):
        seen = set()
        for name, _ in self._entries:
            key = name.lower()
            if key not in seen:
                seen.add(key)
                yield name

    def __len__(self):
        return sum(1 for _ in self)

    def get_all(self, name):
        key = str(name).lower()
        return tuple(value for candidate, value in self._entries if candidate.lower() == key)

    def multi_items(self):
        return list(self._entries)

    @property
    def raw(self):
        return tuple(self._entries)

    def copy(self):
        return Headers(self._entries)

    def __repr__(self):
        return f"Headers({dict(self)!r})"


@dataclass(frozen=True)
class Cookie:
    name: str
    value: str
    domain: str = ""
    path: str = "/"
    secure: bool = False
    http_only: bool = False
    expires: int | None = None
    same_site: str | None = None
    host_only: bool = True


class Cookies(MutableMapping):
    """Domain/path-aware cookies, optionally backed by a live Session jar."""

    def __init__(self, cookies=None, *, session=None, entries=()):
        self._session = session
        self._defaults = {}
        self._entries = list(entries)
        if cookies:
            source = cookies.items() if isinstance(cookies, Mapping) else cookies
            for name, value in source:
                self[str(name)] = str(value)

    def _all(self):
        entries = list(self._entries)
        if self._session is not None and not self._session._closed:
            entries = [
                Cookie(name, value, domain, path, secure, http_only, expires, same_site, host_only)
                for name, value, domain, host_only, path, expires, http_only, secure, same_site
                in self._session._inner.cookies()
            ]
        entries.extend(Cookie(name, value) for name, value in self._defaults.items())
        return entries

    def __getitem__(self, name):
        matches = [cookie.value for cookie in self._all() if cookie.name == str(name)]
        if not matches:
            raise KeyError(name)
        if len(matches) > 1:
            raise CookieConflict(f"multiple cookies named {name!r} exist in different scopes")
        return matches[0]

    def __setitem__(self, name, value):
        self._defaults[str(name)] = str(value)

    def __delitem__(self, name):
        name = str(name)
        existed = name in self._defaults or any(cookie.name == name for cookie in self._all())
        self._defaults.pop(name, None)
        self._entries = [cookie for cookie in self._entries if cookie.name != name]
        if self._session is not None and not self._session._closed:
            self._session._inner.delete_cookies(name)
        if not existed:
            raise KeyError(name)

    def __iter__(self):
        return iter(dict.fromkeys(cookie.name for cookie in self._all()))

    def __len__(self):
        return len(tuple(iter(self)))

    def set(self, name, value, *, domain="", path="/", secure=False, url=None):
        name, value = str(name), str(value)
        if self._session is None or (not domain and url is None):
            if domain or path != "/" or secure:
                self._entries.append(Cookie(name, value, str(domain), str(path), bool(secure)))
            else:
                self._defaults[name] = value
            return
        target = str(url) if url is not None else f"{'https' if secure else 'http'}://{str(domain).lstrip('.')}{path}"
        attributes = [f"{name}={value}", f"Path={path}"]
        if domain:
            attributes.append(f"Domain={domain}")
        if secure:
            attributes.append("Secure")
        try:
            self._session._inner.store_cookie(target, "; ".join(attributes))
        except RuntimeError as error:
            raise _translate(error) from error

    def delete(self, name, *, url=None, domain=None, path=None):
        name = str(name)
        self._defaults.pop(name, None)
        self._entries = [cookie for cookie in self._entries if cookie.name != name]
        if self._session is not None and not self._session._closed:
            self._session._inner.delete_cookies(name, url, domain, path)

    def clear(self):
        self._defaults.clear()
        self._entries.clear()
        if self._session is not None and not self._session._closed:
            self._session._inner.clear_cookies()

    def get_dict(self):
        return {name: self[name] for name in self}

    def _take_request_pairs(self):
        pairs = dict(self._defaults)
        for cookie in self._entries:
            if not cookie.domain:
                pairs[cookie.name] = cookie.value
        self._defaults.clear()
        self._entries = [cookie for cookie in self._entries if cookie.domain]
        return list(pairs.items())

    def __repr__(self):
        return f"Cookies({self._all()!r})"


@dataclass(frozen=True)
class Request:
    method: str
    url: str
    headers: Headers
    body: bytes | None = None


@dataclass(frozen=True)
class Redirect:
    status_code: int
    reason: str
    url: str
    headers: Headers
    request: Request

    @property
    def ok(self):
        return self.status_code < 400


class Multipart:
    MAX_BYTES = 64 * 1024 * 1024

    def __init__(self):
        self._parts = []

    def addpart(self, *, name, data=None, local_path=None, filename=None, content_type=None):
        if (data is None) == (local_path is None):
            raise InvalidRequest("multipart part requires exactly one of data or local_path")
        path = None if local_path is None else Path(local_path)
        if path is not None and filename is None:
            filename = path.name
        self._parts.append((str(name), data, path, filename, content_type))
        return self

    def _encode(self):
        boundary = f"----BrimpFormBoundary{_secrets.token_hex(12)}"
        chunks = []
        size = 0

        def append(value):
            nonlocal size
            value = value.encode() if isinstance(value, str) else bytes(value)
            size += len(value)
            if size > self.MAX_BYTES:
                raise InvalidRequest(f"multipart body exceeds {self.MAX_BYTES} bytes")
            chunks.append(value)

        for name, data, path, filename, content_type in self._parts:
            payload = path.read_bytes() if path is not None else data
            if isinstance(payload, str):
                payload = payload.encode()
            if not isinstance(payload, (bytes, bytearray, memoryview)):
                raise InvalidRequest("multipart data must be str or bytes-like")
            escaped_name = name.replace('"', "%22").replace("\r", "%0D").replace("\n", "%0A")
            disposition = f'Content-Disposition: form-data; name="{escaped_name}"'
            if filename is not None:
                escaped_filename = str(filename).replace('"', "%22").replace("\r", "%0D").replace("\n", "%0A")
                disposition += f'; filename="{escaped_filename}"'
            append(f"--{boundary}\r\n{disposition}\r\n")
            if content_type is not None:
                append(f"Content-Type: {content_type}\r\n")
            append("\r\n")
            append(payload)
            append("\r\n")
        append(f"--{boundary}--\r\n")
        return b"".join(chunks), f"multipart/form-data; boundary={boundary}"


def _merge_headers(base, override):
    result = Headers(base or ())
    if override is None:
        return result
    if isinstance(override, Headers):
        entries = override.multi_items()
    elif isinstance(override, Mapping):
        entries = list(override.items())
    else:
        entries = list(override)
    replaced = {str(name).lower() for name, _ in entries}
    result._entries = [entry for entry in result._entries if entry[0].lower() not in replaced]
    result._entries.extend((str(name), str(value)) for name, value in entries if value is not None)
    return result


def _add_params(url: str, params) -> str:
    if not params:
        return url
    parts = urlsplit(url)
    query = "&".join(filter(None, (parts.query, urlencode(params, doseq=True))))
    return urlunsplit((parts.scheme, parts.netloc, parts.path, query, parts.fragment))


def _merge_params(base, override):
    if not base:
        return override
    if not override:
        return base
    base_items = list(base.items()) if isinstance(base, Mapping) else list(base)
    override_items = list(override.items()) if isinstance(override, Mapping) else list(override)
    names = {str(name) for name, _ in override_items}
    return [(name, value) for name, value in base_items if str(name) not in names] + override_items


def _prepare_body(*, data, content, json, multipart, headers):
    if sum(value is not None for value in (data, content, json, multipart)) > 1:
        raise InvalidRequest("use only one of data, content, json, or multipart")
    body = None
    content_type = None
    if multipart is not None:
        if not isinstance(multipart, Multipart):
            raise InvalidRequest("multipart must be a Multipart instance")
        body, content_type = multipart._encode()
    elif json is not None:
        body = _json.dumps(json, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
        content_type = "application/json"
    elif content is not None:
        if not isinstance(content, (str, bytes, bytearray, memoryview)):
            raise InvalidRequest("content must be str or bytes-like")
        body = content.encode() if isinstance(content, str) else bytes(content)
    elif data is not None:
        if isinstance(data, str):
            body = data.encode()
        elif isinstance(data, (bytes, bytearray, memoryview)):
            body = bytes(data)
        elif isinstance(data, Mapping) or isinstance(data, (list, tuple)):
            body = urlencode(data, doseq=True).encode()
            content_type = "application/x-www-form-urlencoded"
        else:
            raise InvalidRequest("data must be a mapping, sequence of pairs, str, or bytes-like")
    if body is not None:
        if content_type is not None and "content-type" not in headers:
            headers["Content-Type"] = content_type
        if "content-length" not in headers:
            headers["Content-Length"] = str(len(body))
    return body


class Session:
    def __init__(
        self,
        *,
        headers=None,
        cookies=None,
        params=None,
        auth=None,
        timeout: float = 30.0,
        allow_redirects: bool = True,
        max_redirects: int = 30,
        proxy: str | None = None,
        referer: str | None = None,
        default_encoding="utf-8",
        persona_json: str | None = None,
        ca_bundle=None,
        enable_worker: bool = False,
        enable_streaming_networking: bool = False,
        enable_canvas: bool = False,
        enable_webgl: bool = False,
        enable_webgpu: bool = False,
        enable_webaudio: bool = False,
        enable_webaudio_output: bool = False,
        storage_path=None,
        storage_quota_bytes: int | None = None,
        pool_size: int | None = None,
    ):
        if pool_size is None:
            pool_size = 2 * (_os.cpu_count() or 1)
        if isinstance(pool_size, bool) or not isinstance(pool_size, int) or pool_size <= 0:
            raise InvalidRequest("pool_size must be a positive integer")
        try:
            self._inner = _Session(
                persona_json,
                None if ca_bundle is None else str(Path(ca_bundle)),
                bool(enable_worker), bool(enable_streaming_networking), bool(enable_canvas),
                bool(enable_webgl), bool(enable_webgpu), bool(enable_webaudio),
                bool(enable_webaudio_output),
                None if storage_path is None else str(Path(storage_path)),
                storage_quota_bytes,
            )
        except RuntimeError as error:
            raise _translate(error) from error
        self.headers = Headers(headers or ())
        self.params = params
        self.auth = auth
        self.timeout = timeout
        self.allow_redirects = bool(allow_redirects)
        self.max_redirects = max_redirects
        self.proxy = proxy
        self.referer = referer
        self.default_encoding = default_encoding
        self.pool_size = pool_size
        self._closed = False
        self._pool_condition = _threading.Condition()
        self._available_pages = []
        self._page_count = 0
        self.cookies = Cookies(cookies, session=self)

    def new_page(self, *, proxy: str | None = None):
        self._ensure_open()
        chosen_proxy = self.proxy if proxy is None else proxy
        chosen_proxy = None if chosen_proxy is None else str(chosen_proxy)
        return Page(self._acquire_page(chosen_proxy), self, chosen_proxy)

    def _acquire_page(self, proxy):
        replaced = None
        while True:
            with self._pool_condition:
                self._ensure_open()
                for index, (candidate_proxy, inner) in enumerate(self._available_pages):
                    if candidate_proxy == proxy:
                        self._available_pages.pop(index)
                        return inner
                if self._page_count < self.pool_size:
                    self._page_count += 1
                    break
                if self._available_pages:
                    _, replaced = self._available_pages.pop()
                    break
                self._pool_condition.wait()
        if replaced is not None:
            replaced.close()
        try:
            inner = self._inner.new_page(proxy)
        except RuntimeError as error:
            with self._pool_condition:
                self._page_count = max(0, self._page_count - 1)
                self._pool_condition.notify()
            raise _translate(error) from error
        with self._pool_condition:
            if not self._closed:
                return inner
            self._page_count = max(0, self._page_count - 1)
            self._pool_condition.notify_all()
        inner.close()
        raise SessionClosed("session is closed")

    def _release_page(self, proxy, inner):
        with self._pool_condition:
            if not self._closed:
                self._available_pages.append((proxy, inner))
                self._pool_condition.notify()
                return
        inner.close()

    def _discard_page(self, inner):
        inner.close()
        with self._pool_condition:
            self._page_count = max(0, self._page_count - 1)
            self._pool_condition.notify()

    def request(self, method, url, *, proxy=None, **kwargs):
        page = self.new_page(proxy=proxy)
        try:
            return page.request(method, url, **kwargs)
        except BaseException:
            try:
                page.reset()
            except BaseException:
                page.close()
            raise

    def get(self, url, **kwargs): return self.request("GET", url, **kwargs)
    def head(self, url, **kwargs):
        kwargs.setdefault("allow_redirects", False)
        return self.request("HEAD", url, **kwargs)
    def options(self, url, **kwargs): return self.request("OPTIONS", url, **kwargs)
    def delete(self, url, **kwargs): return self.request("DELETE", url, **kwargs)
    def post(self, url, data=None, json=None, **kwargs): return self.request("POST", url, data=data, json=json, **kwargs)
    def put(self, url, data=None, **kwargs): return self.request("PUT", url, data=data, **kwargs)
    def patch(self, url, data=None, **kwargs): return self.request("PATCH", url, data=data, **kwargs)

    def close(self):
        with self._pool_condition:
            if self._closed:
                return
            self._closed = True
            self._available_pages.clear()
            self._page_count = 0
            self._pool_condition.notify_all()
        self._inner.close()

    def _ensure_open(self):
        if self._closed:
            raise SessionClosed("session is closed")

    def __enter__(self):
        self._ensure_open()
        return self

    def __exit__(self, _type, _value, _traceback): self.close()

    def __del__(self):
        try:
            if hasattr(self, "_pool_condition"):
                self.close()
        except BaseException:
            pass


class Page:
    _PROTECTED_HEADERS = {"user-agent", "accept-language"}

    def __init__(self, inner, session: Session, pool_key=None):
        self._inner = inner
        self._session = session
        self._pool_key = pool_key
        self._owns_session = False
        self._closed = False
        self._released = False
        self._navigated = False
        self._encoding = None

    def request(
        self, method, url, *, params=None, data=None, content=None, json=None,
        headers=None, cookies=None, auth=None, timeout=None, allow_redirects=None,
        max_redirects=None, referer=None, default_encoding=None, multipart=None,
    ):
        self._ensure_open()
        method = str(method).upper()
        url = _add_params(str(url), _merge_params(self._session.params, params))
        merged_headers = _merge_headers(self._session.headers, headers)
        chosen_referer = self._session.referer if referer is None else referer
        if chosen_referer is not None and "referer" not in merged_headers:
            merged_headers["Referer"] = str(chosen_referer)
        chosen_auth = self._session.auth if auth is None else auth
        if chosen_auth is not None:
            if not isinstance(chosen_auth, (tuple, list)) or len(chosen_auth) != 2:
                raise InvalidRequest("auth must be a (username, password) pair")
            token = _base64.b64encode(f"{chosen_auth[0]}:{chosen_auth[1]}".encode()).decode()
            merged_headers["Authorization"] = f"Basic {token}"
        protected = self._PROTECTED_HEADERS.intersection(name.lower() for name in merged_headers)
        if protected:
            raise InvalidRequest(
                f"persona-owned headers cannot be overridden: {', '.join(sorted(protected))}; configure a persona instead"
            )
        body = _prepare_body(data=data, content=content, json=json, multipart=multipart, headers=merged_headers)
        if method in {"GET", "HEAD"} and body is not None:
            raise InvalidRequest(f"{method} navigation cannot have a body")
        chosen_timeout = float(self._session.timeout if timeout is None else timeout)
        if not _math.isfinite(chosen_timeout) or chosen_timeout <= 0:
            raise InvalidRequest("timeout must be positive")
        chosen_redirects = self._session.allow_redirects if allow_redirects is None else bool(allow_redirects)
        chosen_limit = self._session.max_redirects if max_redirects is None else max_redirects
        if not isinstance(chosen_limit, int) or chosen_limit < 0:
            raise InvalidRequest("max_redirects must be a non-negative integer")
        cookie_pairs = self._session.cookies._take_request_pairs()
        if cookies:
            source = cookies.items() if isinstance(cookies, Mapping) else cookies
            cookie_pairs = list(dict(cookie_pairs + [(str(name), str(value)) for name, value in source]).items())
        try:
            native = self._inner.request(
                method, url, max(1, round(chosen_timeout * 1000)),
                merged_headers.multi_items(), cookie_pairs, body,
                chosen_redirects, chosen_limit,
            )
        except RuntimeError as error:
            raise _translate(error) from error
        self._apply_navigation(native, default_encoding)
        return self

    def _apply_navigation(self, native, default_encoding):
        self.status_code = native.status_code
        self.reason = native.reason
        self.url = native.url
        self.headers = Headers(native.headers)
        self.content = bytes(native.content)
        self.cookies = Cookies(entries=[Cookie(name, value) for name, value in native.cookies])
        self.elapsed = native.elapsed
        self.http_version = native.http_version
        self.downloaded_bytes = native.downloaded_bytes
        self.uploaded_bytes = native.uploaded_bytes
        self.header_bytes = native.header_bytes
        self.last_request = Request(
            native.request_method, native.request_url, Headers(native.request_headers),
            None if native.request_body is None else bytes(native.request_body),
        )
        self.history = tuple(
            Redirect(
                status_code, reason, url, Headers(headers),
                Request(method, request_url, Headers(request_headers), None if body is None else bytes(body)),
            )
            for status_code, reason, url, headers, method, request_url, request_headers, body
            in native.history
        )
        self.redirect_count = len(self.history)
        self._has_html = native.html is not None
        self._encoding = None
        self.default_encoding = self._session.default_encoding if default_encoding is None else default_encoding
        self._navigated = True

    def get(self, url, **kwargs): return self.request("GET", url, **kwargs)
    def head(self, url, **kwargs):
        kwargs.setdefault("allow_redirects", False)
        return self.request("HEAD", url, **kwargs)
    def options(self, url, **kwargs): return self.request("OPTIONS", url, **kwargs)
    def delete(self, url, **kwargs): return self.request("DELETE", url, **kwargs)
    def post(self, url, data=None, json=None, **kwargs): return self.request("POST", url, data=data, json=json, **kwargs)
    def put(self, url, data=None, **kwargs): return self.request("PUT", url, data=data, **kwargs)
    def patch(self, url, data=None, **kwargs): return self.request("PATCH", url, data=data, **kwargs)

    @property
    def ok(self):
        self._ensure_navigated()
        return self.status_code < 400

    @property
    def encoding(self):
        self._ensure_navigated()
        if self._encoding is not None:
            return self._encoding
        match = _re.search(r"(?:^|;)\s*charset=([^;\s]+)", self.headers.get("content-type", ""), _re.IGNORECASE)
        if match:
            return match.group(1).strip("\"'")
        fallback = self.default_encoding
        return str(fallback(self.content) if callable(fallback) else fallback)

    @encoding.setter
    def encoding(self, value): self._encoding = None if value is None else str(value)

    @property
    def text(self):
        self._ensure_navigated()
        try:
            return self.content.decode(self.encoding, errors="replace")
        except LookupError:
            return self.content.decode("utf-8", errors="replace")

    @property
    def html(self):
        self._ensure_navigated()
        if not self._has_html:
            return None
        try:
            return self._inner.html()
        except RuntimeError as error:
            raise _translate(error) from error

    def json(self, **kwargs): return _json.loads(self.text, **kwargs)

    def iter_content(self, chunk_size=None, decode_unicode=False):
        self._ensure_navigated()
        size = len(self.content) or 1 if chunk_size is None else int(chunk_size)
        if size <= 0:
            raise InvalidRequest("chunk_size must be positive")
        for offset in range(0, len(self.content), size):
            chunk = self.content[offset:offset + size]
            yield chunk.decode(self.encoding, errors="replace") if decode_unicode else chunk

    def iter_lines(self, chunk_size=None, decode_unicode=False, delimiter=None):
        self._ensure_navigated()
        source = self.text if decode_unicode else self.content
        yield from (source.splitlines() if delimiter is None else source.split(delimiter))

    def raise_for_status(self):
        self._ensure_navigated()
        if self.status_code >= 400:
            raise HTTPError(f"{self.status_code} {self.reason} for url: {self.url}", page=self)

    def evaluate(self, expression: str):
        self._ensure_open()
        try:
            return _json.loads(self._inner.evaluate(str(expression)))
        except RuntimeError as error:
            raise _translate(error) from error

    def screenshot(self, path=None, *, full_page: bool = False):
        self._ensure_open()
        try:
            result = bytes(self._inner.screenshot(bool(full_page)))
        except RuntimeError as error:
            raise _translate(error) from error
        if path is not None:
            Path(path).write_bytes(result)
        return result

    def extract(self, *, content_selector=None, remove_images=False, language=None, debug=False):
        self._ensure_open()
        options = {"removeImages": bool(remove_images), "debug": bool(debug)}
        if content_selector is not None: options["contentSelector"] = str(content_selector)
        if language is not None: options["language"] = str(language)
        try:
            return _json.loads(self._inner.extract(_json.dumps(options)))
        except RuntimeError as error:
            raise _translate(error) from error

    def click(self, selector):
        self._ensure_open()
        try: self._inner.click(str(selector))
        except RuntimeError as error: raise _translate(error) from error

    def hover(self, selector):
        self._ensure_open()
        try: self._inner.hover(str(selector))
        except RuntimeError as error: raise _translate(error) from error

    def type(self, selector, text):
        self._ensure_open()
        try: self._inner.type_text(str(selector), str(text))
        except RuntimeError as error: raise _translate(error) from error

    def tap(self, selector):
        self._ensure_open()
        try: self._inner.tap(str(selector))
        except RuntimeError as error: raise _translate(error) from error

    def close(self):
        if self._closed or self._released:
            return
        inner, self._inner = self._inner, None
        self._closed = True
        self._clear_navigation()
        if self._owns_session:
            self._session.close()
        else:
            self._session._discard_page(inner)

    def reset(self):
        if self._closed or self._released:
            return
        if self._owns_session:
            self.close()
            return
        inner, self._inner = self._inner, None
        self._released = True
        self._clear_navigation()
        try:
            inner.reset()
        except RuntimeError as error:
            self._session._discard_page(inner)
            raise _translate(error) from error
        self._session._release_page(self._pool_key, inner)

    def _clear_navigation(self):
        self._navigated = False
        self._encoding = None
        for name in (
            "status_code", "reason", "url", "headers", "content", "cookies",
            "elapsed", "http_version", "downloaded_bytes", "uploaded_bytes",
            "header_bytes", "last_request", "history", "redirect_count",
            "_has_html", "default_encoding",
        ):
            self.__dict__.pop(name, None)

    def _ensure_open(self):
        if self._released:
            raise PageReleased("page has been returned to its Session pool")
        if self._closed:
            raise SessionClosed("page is closed")
        self._session._ensure_open()

    def _ensure_navigated(self):
        self._ensure_open()
        if not self._navigated: raise InvalidRequest("page has not navigated")

    def __getattr__(self, name):
        if self.__dict__.get("_released", False):
            raise PageReleased("page has been returned to its Session pool")
        if self.__dict__.get("_closed", False):
            raise SessionClosed("page is closed")
        raise AttributeError(name)

    def __enter__(self):
        self._ensure_open()
        return self

    def __exit__(self, exception_type, _value, _traceback):
        try:
            self.reset()
        except BaseException:
            if exception_type is None:
                raise
        return False

    def __del__(self):
        try:
            self.close()
        except BaseException:
            pass

    def __repr__(self):
        if self._released:
            return "<Page [released]>"
        if self._closed:
            return "<Page [closed]>"
        return f"<Page [{self.status_code}] {self.url!r}>" if self._navigated else "<Page [new]>"


_SESSION_OPTION_NAMES = {
    "persona_json", "ca_bundle", "enable_worker", "enable_streaming_networking",
    "enable_canvas", "enable_webgl", "enable_webgpu", "enable_webaudio",
    "enable_webaudio_output", "storage_path", "storage_quota_bytes", "pool_size",
}


def request(method, url, **kwargs):
    session_options = {name: kwargs.pop(name) for name in tuple(kwargs) if name in _SESSION_OPTION_NAMES}
    session = Session(**session_options)
    try:
        page = session.request(method, url, **kwargs)
    except BaseException:
        session.close()
        raise
    page._owns_session = True
    return page


def get(url, **kwargs): return request("GET", url, **kwargs)
def head(url, **kwargs):
    kwargs.setdefault("allow_redirects", False)
    return request("HEAD", url, **kwargs)
def options(url, **kwargs): return request("OPTIONS", url, **kwargs)
def delete(url, **kwargs): return request("DELETE", url, **kwargs)
def post(url, data=None, json=None, **kwargs): return request("POST", url, data=data, json=json, **kwargs)
def put(url, data=None, **kwargs): return request("PUT", url, data=data, **kwargs)
def patch(url, data=None, **kwargs): return request("PATCH", url, data=data, **kwargs)


__all__ = [
    "ConnectionError", "Cookie", "CookieConflict", "Cookies", "Headers", "HTTPError",
    "InvalidHeader", "InvalidProxyURL", "InvalidRequest", "InvalidURL", "JavaScriptError",
    "Multipart", "Page", "PageReleased", "ProxyError", "Redirect", "Request", "RequestError", "SSLError",
    "Session", "SessionClosed", "Timeout", "TooManyRedirects", "delete", "get", "head",
    "options", "patch", "post", "put", "request",
]

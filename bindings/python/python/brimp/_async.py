"""Asyncio ownership and scheduling for the shared synchronous/native operations."""

import asyncio
from concurrent.futures import ThreadPoolExecutor
from contextlib import suppress
from functools import partial
import contextvars

from . import Session, JSHandle, ElementHandle, Closed, undefined, _HTTPResult
from .exceptions import SessionClosed
from ._brimp import _Cancellation
from .plugins import _check_session
from ._automation import Automation
from ._timeouts import UNSET, normalize_timeout


async def _drain(future):
    """Finish cleanup even if the caller receives repeated cancellation requests."""
    while not future.done():
        try:
            await asyncio.shield(future)
        except asyncio.CancelledError:
            continue
    return future.result()


class _AsyncRequests:
    async def get(self, url, **kwargs):
        return await self.request("GET", url, **kwargs)

    async def head(self, url, **kwargs):
        return await self.request("HEAD", url, **kwargs)

    async def post(self, url, **kwargs):
        return await self.request("POST", url, **kwargs)

    async def put(self, url, **kwargs):
        return await self.request("PUT", url, **kwargs)

    async def patch(self, url, **kwargs):
        return await self.request("PATCH", url, **kwargs)

    async def delete(self, url, **kwargs):
        return await self.request("DELETE", url, **kwargs)

    async def options(self, url, **kwargs):
        return await self.request("OPTIONS", url, **kwargs)

    async def trace(self, url, **kwargs):
        return await self.request("TRACE", url, **kwargs)

    async def query(self, url, **kwargs):
        return await self.request("QUERY", url, **kwargs)


class AsyncSession(_AsyncRequests):
    """An asyncio session with at most max_clients concurrent native operations.

    Accepts Session's keyword options. Native initialization is lazy; use
    async with or await close() to release resources. A session belongs to the
    event loop of its first use. Separate pages run concurrently; calls on one
    page are serialized. Completed requests do not occupy concurrency slots.
    Plugin and route callbacks remain synchronous in this API.
    """

    def __init__(self, *, max_clients=10, **kwargs):
        if isinstance(max_clients, bool) or not isinstance(max_clients, int) or max_clients < 1:
            raise ValueError("max_clients must be a positive integer")
        self.max_clients = max_clients
        self.base_url = kwargs.pop("base_url", None)
        self.params = kwargs.pop("params", None)
        if self.params is None:
            self.params = {}
        self._options = kwargs
        self._timeout = normalize_timeout(kwargs.get("timeout", 30))
        self._pages = set()
        self._loop = None
        self._executor = ThreadPoolExecutor(
            max_workers=max_clients, thread_name_prefix="brimp-async"
        )
        self._slots = asyncio.Semaphore(max_clients)
        self._initializing = None
        self._sync = None
        self._pending = {}
        self._closed = False
        self._closing = None

    def _check_loop(self):
        loop = asyncio.get_running_loop()
        if self._loop is None:
            self._loop = loop
        elif self._loop is not loop:
            raise RuntimeError("AsyncSession must be used on its original event loop")
        if self._sync is not None:
            _check_session(self._sync._plugin_token)
        return loop

    async def _ready(self):
        loop = self._check_loop()
        if self._closed:
            raise SessionClosed("session is closed")
        if self._initializing is None:
            self._initializing = loop.run_in_executor(
                self._executor,
                partial(Session, base_url=self.base_url, params=self.params, **self._options),
            )
        self._sync = await asyncio.shield(self._initializing)
        if self._closed:
            raise SessionClosed("session is closed")
        self._sync.base_url, self._sync.params = self.base_url, self.params

    async def _run(self, function, *, cleanup=None):
        await self._ready()
        async with self._slots:
            if self._closed:
                raise SessionClosed("session is closed")
            token = _Cancellation()
            context = contextvars.copy_context()
            future = self._loop.run_in_executor(self._executor, context.run, token.run, function)
            self._pending[future] = token
            try:
                result = await asyncio.shield(future)
            except asyncio.CancelledError:
                token.cancel()
                # Wait for native completion before returning the slot or locks.
                with suppress(Exception):
                    result = await _drain(future)
                    if cleanup is not None:
                        await _drain(self._loop.run_in_executor(None, cleanup, result))
                raise
            except Exception as error:
                if self._closed:
                    raise SessionClosed("session is closed") from error
                raise
            else:
                if self._closed:
                    if cleanup is not None:
                        await _drain(self._loop.run_in_executor(None, cleanup, result))
                    raise SessionClosed("session is closed")
                return result
            finally:
                self._pending.pop(future, None)

    async def new_page(self):
        """Create a blank AsyncPage. The session owns its lifetime."""
        page = await self._run(lambda: self._sync.new_page(), cleanup=lambda page: page.close())
        return self._wrap_page(page)

    def _wrap_page(self, page):
        for existing in tuple(self._pages):
            if existing._page._inner.id == page._inner.id:
                return existing
        page = AsyncPage(page, self)
        self._pages.add(page)
        return page

    @property
    def pages(self):
        return [] if self._sync is None else [self._wrap_page(page) for page in self._sync.pages]

    async def request(self, method, url, **kwargs):
        """Navigate a new AsyncPage; accepts Page.request keyword arguments."""
        page = await self.new_page()
        try:
            return await page.request(method, url, **kwargs)
        except BaseException:
            await page.close()
            raise

    async def route(self, pattern, handler, *, stage="request"):
        """Register a synchronous route for existing and future pages."""
        await self._ready()
        return self._sync.route(pattern, handler, stage=stage)

    async def set_cookie(self, url, name, value):
        await self._run(lambda: self._sync.set_cookie(url, name, value))

    async def clear_cookies(self):
        await self._run(lambda: self._sync.clear_cookies())

    @property
    def cookies(self):
        """Snapshot of the shared cookie jar; empty before first use."""
        self._check_loop()
        return [] if self._sync is None else self._sync.cookies

    async def close(self):
        """Cancel pending work, close all pages and drain native worker threads."""
        self._check_loop()
        if self._closing is None:
            self._closed = True
            self._closing = asyncio.create_task(self._shutdown())
        try:
            await asyncio.shield(self._closing)
        except asyncio.CancelledError:
            await _drain(self._closing)
            raise

    async def _shutdown(self):
        for page in tuple(self._pages):
            page._notify_closed()
        self._pages.clear()
        for token in self._pending.values():
            token.cancel()
        try:
            if self._initializing is not None:
                try:
                    session = await asyncio.shield(self._initializing)
                except Exception:
                    return
                await self._loop.run_in_executor(None, session.close)
        finally:
            if self._pending:
                await asyncio.gather(*self._pending, return_exceptions=True)
            await self._loop.run_in_executor(None, self._executor.shutdown, True)

    async def __aenter__(self):
        try:
            await self._ready()
            return self
        except BaseException:
            await self.close()
            raise

    async def __aexit__(self, *_):
        await self.close()


class AsyncPage(Automation, _AsyncRequests, _HTTPResult):
    """A live async page with synchronous buffered HTTP properties and json().

    Await navigation, JavaScript, DOM and handle operations. Cancellation stops
    the active native call; queued calls on this page wait for its cleanup.
    Use async with an existing page, or await close().
    """

    def __init__(self, page, session):
        self._page, self._session = page, session
        self._lock = asyncio.Lock()
        self._closed = False
        self._closing = None
        self._expectations = set()
        self._close_listener = self.on("close", lambda _: self._notify_closed())

    async def _automate(self, command, *, timeout=UNSET):
        return await self._call(self._page._automate, command, timeout=timeout)

    async def current_url(self):
        return await self._call(self._page.current_url)

    def _popup_page(self, record):
        return self._session._wrap_page(self._page._popup_page(record))

    async def wait_for_network_idle(self, *, quiet=0.5, timeout=UNSET):
        return await self._call(self._page.wait_for_network_idle, quiet=quiet, timeout=timeout)

    def _notify_closed(self):
        self._closed = True
        self._session._pages.discard(self)
        for expectation in tuple(self._expectations):
            expectation._closed()

    def _http_response(self):
        return self._page._http_response()

    def _wrap(self, value):
        if isinstance(value, ElementHandle):
            return AsyncElementHandle(value, self)
        if isinstance(value, JSHandle):
            return AsyncJSHandle(value, self)
        if isinstance(value, list):
            return [self._wrap(item) for item in value]
        if isinstance(value, dict):
            return {key: self._wrap(item) for key, item in value.items()}
        return value

    async def _call(self, function, *args, **kwargs):
        self._session._check_loop()
        async with self._lock:
            if self._closed:
                raise Closed("page is closed")
            result = await self._session._run(partial(function, *args, **kwargs))
            return self._wrap(result)

    async def request(self, method, url, **kwargs):
        """Navigate this page using the same options as Page.request."""
        await self._call(self._page.request, method, url, **kwargs)
        return self

    def on(self, event, handler):
        """Register a synchronous event callback; returns an unsubscribe function."""
        self._session._check_loop()
        return self._page.on(event, handler)

    def route(self, pattern, handler, *, stage="request"):
        """Register a synchronous route callback; returns a removal function."""
        self._session._check_loop()
        return self._page.route(pattern, handler, stage=stage)

    async def close(self):
        """Close this page, interrupting its work without closing sibling pages."""
        self._notify_closed()
        self._session._pages.discard(self)
        loop = self._session._check_loop()
        if self._closing is None:
            self._closed = True
            self._closing = loop.run_in_executor(None, self._page.close)
        try:
            await asyncio.shield(self._closing)
        except asyncio.CancelledError:
            await _drain(self._closing)
            raise

    async def __aenter__(self):
        self._session._check_loop()
        if self._closed or self._session._closed:
            raise Closed("page is closed")
        return self

    async def __aexit__(self, *_):
        await self.close()

    async def advance_time(self, milliseconds, *, max_tasks=10000):
        return await self._call(self._page.advance_time, milliseconds, max_tasks=max_tasks)

    async def run_pending_tasks(self, *, max_tasks=10000):
        return await self._call(self._page.run_pending_tasks, max_tasks=max_tasks)

    async def profile_contract(self):
        return await self._call(self._page.profile_contract)

    async def profile_report(self):
        return await self._call(self._page.profile_report)

    async def take_diagnostics(self):
        return await self._call(self._page.take_diagnostics)

    async def peek_diagnostics(self):
        return await self._call(self._page.peek_diagnostics)

    async def summarize_diagnostics(self):
        return await self._call(self._page.summarize_diagnostics)

    async def load_html(self, html, url="about:blank"):
        return await self._call(self._page.load_html, html, url)

    async def set_identity(self, *, user_agent=None, languages=None, platform=None):
        return await self._call(
            self._page.set_identity, user_agent=user_agent, languages=languages, platform=platform
        )

    async def configure_network(
        self,
        *,
        offline=False,
        latency=0.0,
        bypass_cache=False,
        max_retained_bytes=32 * 1024 * 1024,
        max_retained_body_bytes=8 * 1024 * 1024,
        buffered_download_bytes_per_second=None,
    ):
        return await self._call(
            self._page.configure_network,
            offline=offline,
            latency=latency,
            bypass_cache=bypass_cache,
            max_retained_bytes=max_retained_bytes,
            max_retained_body_bytes=max_retained_body_bytes,
            buffered_download_bytes_per_second=buffered_download_bytes_per_second,
        )

    async def set_extra_headers(self, headers):
        return await self._call(self._page.set_extra_headers, headers)

    async def clear_cache(self):
        return await self._call(self._page.clear_cache)

    async def response_body(self, request_id):
        return await self._call(self._page.response_body, request_id)

    async def evaluate(self, source, *, timeout=UNSET):
        return await self._call(self._page.evaluate, source, timeout=timeout)

    async def evaluate_handle(self, source):
        return await self._call(self._page.evaluate_handle, source)

    async def query_selector(self, selector):
        return await self._call(self._page.query_selector, selector)

    async def query_selector_all(self, selector):
        return await self._call(self._page.query_selector_all, selector)

    async def add_init_script(self, source):
        return await self._call(self._page.add_init_script, source)

    async def click(self, selector):
        return await self._call(self._page.click, selector)

    async def hover(self, selector):
        return await self._call(self._page.hover, selector)

    async def tap(self, selector):
        return await self._call(self._page.tap, selector)

    async def focus(self, selector):
        return await self._call(self._page.focus, selector)

    async def type_text(self, selector, text):
        return await self._call(self._page.type_text, selector, text)

    async def fill(self, selector, text):
        return await self._call(self._page.fill, selector, text)

    async def press(self, selector, key, *, modifiers=0):
        return await self._call(self._page.press, selector, key, modifiers=modifiers)

    async def drag_to(self, source, destination):
        return await self._call(self._page.drag_to, source, destination)

    async def set_input_files(self, selector, files):
        return await self._call(self._page.set_input_files, selector, files)

    async def extract(
        self, *, content_selector=None, remove_images=False, language=None, debug=False
    ):
        return await self._call(
            self._page.extract,
            content_selector=content_selector,
            remove_images=remove_images,
            language=language,
            debug=debug,
        )

    async def html(self):
        return await self._call(self._page.html)


class AsyncJSHandle:
    """An async rooted JavaScript value; await dispose() to release its root."""

    def __init__(self, handle, page):
        self._handle, self._page = handle, page

    async def __aenter__(self):
        return self

    async def __aexit__(self, *_):
        await self.dispose()

    async def value(self):
        return await self._page._call(self._handle.value)

    async def json_value(self):
        return await self._page._call(self._handle.json_value)

    async def call(self, function, *args):
        return await self._page._call(self._handle.call, function, *args)

    async def invoke(self, *args, this=undefined):
        return await self._page._call(self._handle.invoke, *args, this=this)

    async def property(self, name):
        return await self._page._call(self._handle.property, name)

    async def set_property(self, name, value):
        return await self._page._call(self._handle.set_property, name, value)

    async def delete_property(self, name):
        return await self._page._call(self._handle.delete_property, name)

    async def property_names(self):
        return await self._page._call(self._handle.property_names)

    async def dispose(self):
        return await self._page._call(self._handle.dispose)


class AsyncElementHandle(AsyncJSHandle):
    """An async DOM element with query, edit and input operations."""

    async def focus(self):
        return await self._page._call(self._handle.focus)

    async def click(self):
        return await self._page._call(self._handle.click)

    async def set_input_files(self, files):
        return await self._page._call(self._handle.set_input_files, files)

    async def get_attribute(self, name):
        return await self._page._call(self._handle.get_attribute, name)

    async def set_attribute(self, name, value):
        return await self._page._call(self._handle.set_attribute, name, value)

    async def text_content(self):
        return await self._page._call(self._handle.text_content)

    async def html(self):
        return await self._page._call(self._handle.html)

    async def set_text_content(self, text):
        return await self._page._call(self._handle.set_text_content, text)

    async def remove(self):
        return await self._page._call(self._handle.remove)

    async def query_selector(self, selector):
        return await self._page._call(self._handle.query_selector, selector)

    async def query_selector_all(self, selector):
        return await self._page._call(self._handle.query_selector_all, selector)

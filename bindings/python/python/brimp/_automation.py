"""Locators, frame scopes and race-free event expectations."""

import asyncio
import inspect
import threading
import time

from ._timeouts import UNSET, normalize_timeout
from .exceptions import Closed, Timeout


class Locator:
    """A strict, reusable selector. Each operation resolves the current DOM."""

    def __init__(self, context, selectors, index=None):
        self._context, self._selectors, self._index = context, selectors, index

    def locator(self, selector: str):
        if self._index is not None:
            raise ValueError("Chain selectors before selecting nth()")
        return Locator(self._context, [*self._selectors, selector])

    def nth(self, index: int):
        if isinstance(index, bool) or not isinstance(index, int) or index < 0:
            raise ValueError("index must be a nonnegative integer")
        return Locator(self._context, self._selectors, index)

    def _run(self, op, timeout=UNSET, **command):
        command.update(op=op, selectors=self._selectors)
        if self._index is not None:
            command["index"] = self._index
        return self._context._automate(command, timeout=timeout)

    def wait_for(self, *, state="visible", timeout=UNSET):
        return self._run("wait", timeout, state=state)

    def count(self):
        return self._run("count")

    def text_content(self, *, timeout=UNSET):
        return self._run("text", timeout)

    def get_attribute(self, name: str, *, timeout=UNSET):
        return self._run("attribute", timeout, name=name)

    def click(self, *, timeout=UNSET):
        return self._run("input", timeout, input={"action": "click"})

    def hover(self, *, timeout=UNSET):
        return self._run("input", timeout, input={"action": "hover"})

    def fill(self, text: str, *, timeout=UNSET):
        return self._run("input", timeout, input={"action": "fill", "text": text})

    def type_text(self, text: str, *, timeout=UNSET):
        return self._run("input", timeout, input={"action": "type", "text": text})

    def press(self, key: str, *, modifiers=0, timeout=UNSET):
        return self._run(
            "input", timeout, input={"action": "press", "key": key, "modifiers": modifiers}
        )


class Frame:
    """A frame selector path, resolved afresh after frame or document replacement.

    Methods return awaitables when the owning page is an AsyncPage.
    """

    def __init__(self, page, selectors):
        self._page, self._selectors = page, selectors

    def _automate(self, command, *, timeout=UNSET):
        return self._page._automate({**command, "frames": self._selectors}, timeout=timeout)

    def frame_locator(self, selector: str):
        return Frame(self._page, [*self._selectors, selector])

    def frames(self):
        result = self._automate({"op": "frames"})

        def wrap(paths):
            return [self.frame_locator(path) for path in paths]

        if inspect.isawaitable(result):

            async def finish():
                return wrap(await result)

            return finish()
        return wrap(result)

    def url(self):
        return self.evaluate("location.href")

    def locator(self, selector: str):
        return Locator(self, [selector])

    def evaluate(self, source: str, *, timeout=UNSET):
        return self._automate({"op": "evaluate", "source": source}, timeout=timeout)

    def wait_for_function(self, source: str, *, arg=None, timeout=UNSET):
        return self._automate({"op": "function", "source": source, "arg": arg}, timeout=timeout)

    def wait_for_selector(self, selector: str, *, state="visible", timeout=UNSET):
        return self.locator(selector).wait_for(state=state, timeout=timeout)

    def wait_for_load_state(self, state="load", *, timeout=UNSET):
        if state not in ("load", "domcontentloaded"):
            raise ValueError("state must be load or domcontentloaded")
        return self._automate({"op": "load", "state": state}, timeout=timeout)


class EventExpectation:
    """Subscribe before an action and remove the subscription on every exit path."""

    def __init__(self, page, event, predicate, timeout, transform=None):
        if page._closed:
            raise Closed("page is closed")
        self._page, self._predicate = page, predicate
        self._transform = transform or (lambda value: value)
        self._timeout = normalize_timeout(timeout)
        if self._timeout is None:
            self._timeout = page._session._timeout
        self._deadline = time.monotonic() + self._timeout if self._timeout else None
        self._done = threading.Event()
        self._result = self._error = None
        self._loop = None
        self._future = None
        self._remove = page.on(event, self._receive)
        page._expectations.add(self)

    def _receive(self, record):
        try:
            if self._predicate is not None and not self._predicate(record):
                return
            self._finish(record, None)
        except Exception as error:  # noqa: BLE001 - deliver predicate errors to the waiter
            self._finish(None, error)

    def _finish(self, result, error):
        if self._done.is_set():
            return
        self._result, self._error = result, error
        self._done.set()
        if self._loop is not None:
            self._loop.call_soon_threadsafe(self._wake)

    def _wake(self):
        if not self._future.done():
            self._future.set_result(None)

    def _dispose(self):
        self._remove()
        self._page._expectations.discard(self)

    def _remaining(self):
        return max(0, self._deadline - time.monotonic()) if self._deadline else None

    def _closed(self):
        self._finish(None, Closed("page closed while waiting for event"))

    @property
    def value(self):
        if self._future is not None:
            return self._wait_async()
        try:
            if not self._done.wait(self._remaining()):
                raise Timeout("event wait timed out")
            if self._error is not None:
                raise self._error
            return self._transform(self._result)
        finally:
            self._dispose()

    def __enter__(self):
        return self

    def __exit__(self, kind, *_):
        if kind is None:
            _ = self.value
        else:
            self._dispose()

    async def __aenter__(self):
        self._loop = asyncio.get_running_loop()
        self._future = self._loop.create_future()
        return self

    async def _wait_async(self):
        try:
            if not self._done.is_set():
                await asyncio.wait_for(asyncio.shield(self._future), self._remaining())
            if self._error is not None:
                raise self._error
            return self._transform(self._result)
        except TimeoutError:
            raise Timeout("event wait timed out") from None
        finally:
            self._dispose()

    async def __aexit__(self, kind, *_):
        if kind is None:
            await self._wait_async()
        else:
            self._dispose()


class Automation:
    """Shared operation builders for synchronous and asynchronous pages."""

    def locator(self, selector: str):
        return Locator(self, [selector])

    def frame_locator(self, selector: str):
        return Frame(self, [selector])

    def frames(self):
        return Frame(self, []).frames()

    def wait_for_selector(self, selector: str, *, state="visible", timeout=UNSET):
        return self.locator(selector).wait_for(state=state, timeout=timeout)

    def wait_for_function(self, source: str, *, arg=None, timeout=UNSET):
        return self._automate({"op": "function", "source": source, "arg": arg}, timeout=timeout)

    def wait_for_load_state(self, state="load", *, timeout=UNSET):
        return Frame(self, []).wait_for_load_state(state, timeout=timeout)

    def expect_event(self, event: str, predicate=None, *, timeout=UNSET):
        return EventExpectation(self, event, predicate, timeout)

    def expect_response(self, url: str, *, timeout=UNSET):
        return self.expect_event(
            "response", lambda record: record["response"]["url"] == url, timeout=timeout
        )

    def expect_popup(self, *, timeout=UNSET):
        return EventExpectation(self, "popup", None, timeout, self._popup_page)

    def expect_navigation(self, *, wait_until="load", timeout=UNSET):
        if wait_until not in ("load", "domcontentloaded"):
            raise ValueError("wait_until must be load or domcontentloaded")
        name = "dom_content_loaded" if wait_until == "domcontentloaded" else "load"
        return self.expect_event(
            "lifecycle", lambda record: record["name"] == name, timeout=timeout
        )

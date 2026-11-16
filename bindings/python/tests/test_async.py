"""Async APIs use the real native runtime and a local HTTP server."""

import asyncio
import json
import queue
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from types import SimpleNamespace

import pytest

import brimp


@pytest.fixture
def server():
    state = SimpleNamespace(gate=threading.Event(), started=queue.Queue())

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path.startswith("/slow"):
                state.started.put(self.path)
                state.gate.wait(10)
            self.send_response(404 if self.path.startswith("/missing") else 200)
            self.send_header(
                "Content-Type", "text/html" if self.path == "/html" else "application/json"
            )
            self.end_headers()
            content = (
                b"<h1>original</h1><script>document.querySelector('h1').textContent = 'rendered'</script>"
                if self.path == "/html"
                else json.dumps(
                    {"path": self.path, "cookie": self.headers.get("Cookie", "")}
                ).encode()
            )
            try:
                self.wfile.write(content)
            except (BrokenPipeError, ConnectionResetError):
                pass

        def do_POST(self):
            content = self.rfile.read(int(self.headers.get("Content-Length", "0")))
            self.send_response(200)
            self.end_headers()
            self.wfile.write(content)

        def log_message(self, *_):
            pass

    httpd = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=httpd.serve_forever, daemon=True)
    thread.start()
    state.url = f"http://127.0.0.1:{httpd.server_port}"
    yield state
    state.gate.set()
    httpd.shutdown()
    httpd.server_close()
    thread.join()


def test_requests_defaults_http_errors_and_page_lifetime(server):
    async def run():
        async with brimp.AsyncSession(
            base_url=server.url + "/", params={"key": "default"}
        ) as session:
            await session.set_cookie(server.url, "shared", "yes")
            page = await session.get("echo", params={"key": "override", "q": "a b"})
            assert isinstance(page, brimp.AsyncPage)
            assert page.status_code == 200
            assert page.json() == {"path": "/echo?key=override&q=a+b", "cookie": "shared=yes"}
            assert session.cookies
            session.params = {}
            session.base_url = server.url + "/api/"
            assert await page.get("next") is page
            assert page.json()["path"] == "/api/next"
            assert await page.post("form", data=[("a", 1), ("a", 2)]) is page
            assert page.text == "a=1&a=2"
            await page.get("/missing")
            with pytest.raises(brimp.HTTPError) as error:
                page.raise_for_status()
            assert error.value.page is page
            assert error.value.status_code == 404
            await session.clear_cookies()
            assert session.cookies == []
            await page.close()
            assert page.status_code == 404
            with pytest.raises(brimp.Closed):
                await page.evaluate("42")
            async with await session.get("/html") as other:
                assert (
                    await other.evaluate("document.querySelector('h1').textContent") == "rendered"
                )
        assert other.status_code == 200
        with pytest.raises(brimp.Closed):
            await session.get("/echo")
        await session.close()
        assert not any(thread.is_alive() for thread in session._executor._threads)

    asyncio.run(run())


def test_concurrency_limit_and_event_loop_responsiveness(server):
    async def run():
        async with brimp.AsyncSession(max_clients=2, plugins=[]) as session:
            pages = [await session.new_page() for _ in range(3)]
            tasks = [
                asyncio.create_task(page.get(server.url + f"/slow/{i}"))
                for i, page in enumerate(pages)
            ]
            await asyncio.to_thread(server.started.get, True, 3)
            await asyncio.to_thread(server.started.get, True, 3)
            # The loop remains responsive while both workers are waiting on HTTP.
            for _ in range(10):
                await asyncio.sleep(0.005)
            assert server.started.empty()
            assert not any(task.done() for task in tasks)
            server.gate.set()
            results = await asyncio.wait_for(asyncio.gather(*tasks), 3)
            assert len(results) == 3
            assert all(page.status_code == 200 for page in results)
            # Open pages do not hold request slots.
            assert await pages[0].evaluate("6 * 7") == 42

    asyncio.run(run())


def test_cancel_active_and_queued_requests_preserves_session(server):
    async def run():
        async with brimp.AsyncSession(max_clients=1, plugins=[]) as session:
            page = await session.new_page()
            other = await session.new_page()
            active = asyncio.create_task(page.get(server.url + "/slow/active"))
            await asyncio.to_thread(server.started.get, True, 3)
            queued = asyncio.create_task(other.get(server.url + "/slow/queued"))
            await asyncio.sleep(0)
            queued.cancel()
            with pytest.raises(asyncio.CancelledError):
                await queued
            active.cancel()
            with pytest.raises(asyncio.CancelledError):
                await asyncio.wait_for(active, 2)
            assert not session._pending
            assert server.started.empty()
            assert (await other.get(server.url + "/ok")).status_code == 200
            assert (await page.get(server.url + "/again")).status_code == 200

    asyncio.run(run())


@pytest.mark.parametrize("expression", ["new Promise(() => {})", "while (true) {}"])
def test_cancel_javascript_and_reuse_page(expression):
    async def run():
        async with brimp.AsyncSession(plugins=[], timeout=5) as session:
            page = await session.new_page()
            task = asyncio.create_task(page.evaluate(expression))
            await asyncio.sleep(0.05)
            task.cancel()
            with pytest.raises(asyncio.CancelledError):
                await asyncio.wait_for(task, 2)
            assert not session._pending
            assert await page.evaluate("21 * 2") == 42

    asyncio.run(run())


@pytest.mark.parametrize("function", ["() => new Promise(() => {})", "() => { while (true) {} }"])
def test_async_handles_and_cancelled_handle_call(function):
    async def run():
        async with brimp.AsyncSession(plugins=[]) as session:
            page = await session.new_page()
            await page.load_html("<main><b>before</b></main>")
            async with await page.query_selector("main") as root:
                child = await root.query_selector("b")
                assert isinstance(child, brimp.AsyncElementHandle)
                await child.set_text_content("after")
                assert await child.text_content() == "after"
                assert "<b>after</b>" in await page.html()
                async with await child.call("node => ({node, answer: 42})") as handle:
                    result = await handle.call("(value, node) => value.node === node", child)
                    async with result:
                        assert await result.json_value() is True
                    assert await handle.property_names() == ["node", "answer"]
                    pending = asyncio.create_task(handle.call(function))
                    await asyncio.sleep(0.05)
                    pending.cancel()
                    with pytest.raises(asyncio.CancelledError):
                        await asyncio.wait_for(pending, 2)
                    assert await handle.property_names() == ["node", "answer"]
                await child.dispose()
                with pytest.raises(brimp.StaleHandle):
                    await child.text_content()
            await page.load_html("<p>new</p>")
            assert len(await page.query_selector_all("p")) == 1

    asyncio.run(run())


def test_close_cancels_active_and_queued_work(server):
    async def run():
        session = brimp.AsyncSession(max_clients=1, plugins=[])
        pages = [await session.new_page() for _ in range(2)]
        active = asyncio.create_task(pages[0].get(server.url + "/slow"))
        await asyncio.to_thread(server.started.get, True, 3)
        queued = asyncio.create_task(pages[1].evaluate("42"))
        await asyncio.sleep(0)
        await asyncio.wait_for(session.close(), 2)
        results = await asyncio.gather(active, queued, return_exceptions=True)
        assert all(isinstance(result, brimp.RuntimeError) for result in results)
        assert not session._pending
        assert not any(thread.is_alive() for thread in session._executor._threads)
        await session.close()

    asyncio.run(run())


def test_cancel_waiting_on_page_lock_does_not_cancel_active_operation():
    async def run():
        async with brimp.AsyncSession(plugins=[]) as session:
            page = await session.new_page()
            active = asyncio.create_task(
                page.evaluate("new Promise(r => setTimeout(() => r(42), 150))")
            )
            await asyncio.sleep(0.02)
            queued = asyncio.create_task(page.evaluate("throw Error('must not run')"))
            await asyncio.sleep(0)
            queued.cancel()
            with pytest.raises(asyncio.CancelledError):
                await queued
            assert await active == 42

    asyncio.run(run())


def test_close_page_interrupts_work_but_preserves_other_pages():
    async def run():
        async with brimp.AsyncSession(plugins=[]) as session:
            page = await session.new_page()
            other = await session.new_page()
            pending = asyncio.create_task(page.evaluate("new Promise(() => {})"))
            await asyncio.sleep(0.02)
            await asyncio.wait_for(page.close(), 2)
            with pytest.raises(brimp.Closed):
                await pending
            assert await other.evaluate("42") == 42

    asyncio.run(run())


@pytest.mark.parametrize("max_clients", [0, -1, True, 1.5])
def test_invalid_concurrency(max_clients):
    with pytest.raises(ValueError, match="positive integer"):
        brimp.AsyncSession(max_clients=max_clients)


def test_initialization_failure_and_unused_session_close():
    async def run():
        async with brimp.AsyncSession(plugins=[]) as session:
            pass
        unused = brimp.AsyncSession()
        await unused.close()
        with pytest.raises(brimp.Closed):
            await unused.new_page()
        invalid = brimp.AsyncSession(base_url="relative")
        with pytest.raises(ValueError):
            async with invalid:
                pass
        assert not any(thread.is_alive() for thread in invalid._executor._threads)

    asyncio.run(run())


def test_cancel_close_waits_for_cleanup_and_disposes_plugins_once():
    entered, release = threading.Event(), threading.Event()
    disposed = []

    def dispose():
        entered.set()
        release.wait(3)
        disposed.append(True)

    plugin = brimp.Plugin("cleanup", "() => {}", setup=lambda: {"dispose": dispose})

    async def run():
        session = brimp.AsyncSession(plugins=[plugin])
        await session.new_page()
        closing = asyncio.create_task(session.close())
        await asyncio.to_thread(entered.wait, 2)
        assert entered.is_set()
        closing.cancel()
        await asyncio.sleep(0)
        closing.cancel()
        await asyncio.sleep(0)
        assert not closing.done()
        release.set()
        with pytest.raises(asyncio.CancelledError):
            await closing
        await session.close()
        assert disposed == [True]
        assert not any(thread.is_alive() for thread in session._executor._threads)

    try:
        asyncio.run(run())
    finally:
        release.set()


def test_cancel_request_closes_its_new_page(server):
    async def run():
        async with brimp.AsyncSession(plugins=[]) as session:
            created = []
            original = session.new_page

            async def capture():
                page = await original()
                created.append(page)
                return page

            session.new_page = capture
            task = asyncio.create_task(session.get(server.url + "/slow"))
            await asyncio.to_thread(server.started.get, True, 3)
            task.cancel()
            with pytest.raises(asyncio.CancelledError):
                await task
            with pytest.raises(brimp.Closed):
                await created[0].evaluate("42")
            assert (await session.get(server.url + "/ok")).status_code == 200

    asyncio.run(run())


def test_event_loop_affinity():
    session = brimp.AsyncSession(plugins=[])
    with asyncio.Runner() as owner, asyncio.Runner() as other:
        owner.run(session.new_page())
        try:
            with pytest.raises(RuntimeError, match="original event loop"):
                other.run(session.new_page())
        finally:
            owner.run(session.close())


def test_synchronous_callbacks_and_native_failures(server):
    async def run():
        plugin = brimp.Plugin(
            "host",
            "({global, host}) => { global.hostAnswer = () => host.call('answer'); }",
            handlers={"answer": lambda context: 42},
        )
        async with brimp.AsyncSession(plugins=[plugin]) as session:
            page = await session.new_page()
            remove = page.route(
                server.url + "/intercept",
                lambda route: route.fulfill(
                    body="<h1>route</h1>", headers={"Content-Type": "text/html"}
                ),
            )
            await page.get(server.url + "/intercept")
            assert "<h1>route</h1>" in await page.html()
            assert await page.evaluate("hostAnswer()") == 42
            remove()
            with pytest.raises(brimp.JavaScriptError):
                await page.evaluate("throw Error('async error')")
            with pytest.raises(brimp.InvalidInput):
                await page.get(server.url, body="one", data={"two": 2})
            assert not session._pending
            assert await page.evaluate("42") == 42

    asyncio.run(run())


def test_cancel_page_initialization():
    entered = threading.Event()
    plugin = brimp.Plugin(
        "pending",
        "async ({host}) => { await host.call('entered'); await new Promise(() => {}); }",
        handlers={"entered": lambda context: entered.set()},
    )

    async def run():
        async with brimp.AsyncSession(plugins=[plugin], timeout=5) as session:
            task = asyncio.create_task(session.new_page())
            assert await asyncio.to_thread(entered.wait, 2)
            task.cancel()
            with pytest.raises(asyncio.CancelledError):
                await asyncio.wait_for(task, 2)
            assert not session._pending

    asyncio.run(run())

import asyncio
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

import brimp
import pytest


@pytest.fixture(scope="module")
def server():
    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):
            self.send_response(200)
            self.send_header("Content-Type", "text/html")
            self.end_headers()
            self.wfile.write(
                b'<title>destination</title><button style="width:100px;height:40px">Go</button>'
            )

        def log_message(self, *_):
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    yield f"http://127.0.0.1:{server.server_port}"
    server.shutdown()
    server.server_close()
    thread.join()


def test_locator_re_resolves_and_checks_actionability():
    with brimp.Session(plugins=[]) as session:
        page = session.new_page()
        page.load_html('<button id="go" disabled style="width:100px;height:40px">old</button>')
        button = page.locator("#go")
        page.evaluate(
            "setTimeout(() => { const replacement = document.createElement('button'); replacement.id = 'go'; replacement.style.cssText = 'width:100px;height:40px'; replacement.textContent = 'new'; document.querySelector('#go').replaceWith(replacement); replacement.onclick = () => window.clicked = true; }, 30)"
        )
        button.click(timeout=2)
        assert page.evaluate("clicked") is True
        assert button.text_content() == "new"
        page.evaluate("document.body.insertAdjacentHTML('beforeend', '<button>second</button>')")
        with pytest.raises(brimp.InvalidInput, match="Strict locator"):
            page.locator("button").click(timeout=1)
        assert page.locator("button").nth(1).text_content() == "second"
        assert page.locator("button").count() == 2


def test_waits_and_frame_input():
    with brimp.Session(plugins=[]) as session:
        page = session.new_page()
        page.load_html(
            '<iframe id="frame" srcdoc="&lt;input id=field style=width:100px;height:30px&gt;"></iframe>'
        )
        frame = page.frame_locator("#frame")
        assert len(page.frames()) == 1
        frame.locator("#field").fill("hello", timeout=2)
        assert frame.evaluate("document.querySelector('#field').value") == "hello"
        assert frame.evaluate("({value: 42, nested: [true]})") == {"value": 42, "nested": [True]}
        page.evaluate("setTimeout(() => window.answer = 42, 20)")
        assert page.wait_for_function("() => window.answer", timeout=1) == 42
        page.wait_for_selector("#missing", state="detached", timeout=1)
        with pytest.raises(brimp.Timeout):
            page.wait_for_selector("#missing", timeout=0.05)
        assert page.evaluate("2 + 2") == 4
        with pytest.raises(brimp.JavaScriptError):
            page.wait_for_function("Promise.reject('')", timeout=1)


def test_navigation_response_and_popup(server):
    with brimp.Session(plugins=[]) as session:
        page = session.new_page()
        with page.expect_response(server + "/") as response, page.expect_navigation():
            page.get(server + "/", wait_until="domcontentloaded")
        assert response.value["response"]["status"] == 200
        page.wait_for_load_state(timeout=2)
        page.evaluate("void fetch('/background')")
        page.wait_for_network_idle(quiet=0.02, timeout=2)
        with page.expect_popup(timeout=5) as popup_event:
            page.evaluate("void (window.child = window.open(" + repr(server + "/popup") + "))")
        popup = popup_event.value
        assert popup in session.pages
        assert popup.current_url() == server + "/popup"
        assert popup.evaluate("document.title") == "destination"
        popup.close()
        assert page.evaluate("child.closed") is True
        with page.expect_navigation(timeout=5):
            page.evaluate("location.href = " + repr(server + "/next"))
        assert page.current_url() == server + "/next"
        with page.expect_popup(timeout=5) as second_event:
            page.evaluate("void window.open(" + repr(server + "/second") + ")")
        second = second_event.value
        page.close()
        assert second.evaluate("document.title") == "destination"
        assert page not in session.pages
        with second.expect_popup(timeout=5) as third_event:
            second.evaluate("void (window.child = window.open('about:blank'))")
        third = third_event.value
        pending = third.expect_event("response", timeout=2)
        second.evaluate("child.close()")
        with pytest.raises(brimp.Closed):
            _ = pending.value


def test_event_timeout_and_failure_remove_subscriptions():
    with brimp.Session(plugins=[]) as session:
        page = session.new_page()
        with pytest.raises(brimp.Timeout), page.expect_event("response", timeout=0.02):
            pass
        assert not page._expectations
        with pytest.raises(ValueError), page.expect_event("response"):
            raise ValueError("action failed")
        assert not page._expectations
        pending = page.expect_event("response", timeout=None)
        page.close()
        with pytest.raises(brimp.Closed):
            _ = pending.value


def test_async_wait_cancellation_and_expectations(server):
    async def run():
        async with brimp.AsyncSession(plugins=[]) as session:
            page = await session.new_page()
            task = asyncio.create_task(page.wait_for_function("false", timeout=None))
            await asyncio.sleep(0.03)
            task.cancel()
            with pytest.raises(asyncio.CancelledError):
                await task
            assert await page.evaluate("42") == 42
            async with page.expect_response(server + "/") as response:
                await page.get(server + "/")
            assert (await response.value)["response"]["status"] == 200
            assert await page.locator("button").text_content() == "Go"
            async with page.expect_popup(timeout=5) as event:
                await page.evaluate("void window.open(" + repr(server + "/popup") + ")")
            popup = await event.value
            assert await popup.evaluate("document.title") == "destination"

    asyncio.run(run())

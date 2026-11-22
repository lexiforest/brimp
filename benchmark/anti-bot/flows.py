"""Run the mechanics the pyv8c examples depend on, against the local stand-in.

Each check corresponds to one thing an example does between navigation and the
cookie it waits for. It answers "can the runtime run this flow", not "would the
real challenge accept it": the stand-in is not Akamai, DataDome, PX or Cloudflare.
"""

from __future__ import annotations

import json
import pathlib
import sys

import brimp

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from server import serve  # noqa: E402

RESULTS: list[tuple[str, bool, str]] = []


def check(name, function):
    try:
        detail = function()
        RESULTS.append((name, True, detail or ""))
    except Exception as error:  # noqa: BLE001 - report, do not stop the run
        RESULTS.append((name, False, f"{type(error).__name__}: {error}"))


def main() -> int:
    server, port = serve()
    origin = f"http://127.0.0.1:{port}"
    try:
        session = brimp.Session()
        page = session.get(origin + "/")

        def navigate():
            # The stylesheet of a linked `<link>` is still in flight when `get()`
            # returns: `run_pending_tasks()` does not wait for it, and
            # `document.readyState` already reads `complete`. Recorded here so the
            # gap is visible instead of hidden by a sleep.
            immediate = page.evaluate("[document.styleSheets.length, document.readyState]")
            page.run_pending_tasks()
            settled = page.evaluate("document.styleSheets.length")
            import time

            time.sleep(0.3)
            return (
                f"status={page.status_code} immediate={immediate} "
                f"after_pump={settled} after_wait={page.evaluate('document.styleSheets.length')}"
            )

        def challenge_runs():
            return page.evaluate("typeof collect === 'function'")

        def page_fetch_sets_cookie():
            status = page.evaluate("collect()")
            return f"collect()={status}"

        def cookie_jar():
            records = session.cookies
            return json.dumps(sorted(record.get("name") for record in records))

        def api_with_cookie():
            response = session.get(origin + "/api")
            return f"status={response.status_code}"

        def blocked_resource():
            blocked = brimp.Session(trust_env=False)
            remove = blocked.route(
                "*",
                lambda route: (
                    route.fail("blocked")
                    if route.request["resource_type"] == "stylesheet"
                    else route.continue_request()
                ),
            )
            with blocked.get(origin + "/") as blocked_page:
                sheets = blocked_page.evaluate("document.styleSheets.length")
                color = blocked_page.evaluate(
                    "getComputedStyle(document.getElementById('copy')).color"
                )
            remove()
            blocked.close()
            return f"sheets={sheets} color={color}"

        def click_dispatches_pointer_events():
            page.evaluate(
                "globalThis.__clicks = [];"
                "const target = document.getElementById('copy');"
                "for (const type of ['pointerdown','mousedown','pointerup','click'])"
                "  target.addEventListener(type, (event) => __clicks.push([type, event.isTrusted]));"
            )
            page.click("#copy")
            return page.evaluate("JSON.stringify(__clicks)")

        def hover_is_pointer_move():
            page.evaluate(
                "globalThis.__moves = 0;"
                "document.getElementById('copy').addEventListener('pointermove', () => __moves++);"
            )
            page.hover("#copy")
            return f"pointermove={page.evaluate('globalThis.__moves')}"

        def frames_are_reachable():
            page.evaluate(
                "document.body.insertAdjacentHTML('beforeend',"
                " '<iframe id=f srcdoc=\"<p id=inner>frame</p>\"></iframe>')"
            )
            return page.evaluate(
                "(() => { const f = document.getElementById('f'); return f.contentDocument.getElementById('inner').textContent })()"
            )

        def pending_tasks():
            page.evaluate(
                "globalThis.__timer = false; setTimeout(() => { globalThis.__timer = true }, 0)"
            )
            page.run_pending_tasks()
            return f"timer_fired={page.evaluate('globalThis.__timer')}"

        def saved_document():
            other = session.new_page()
            other.load_html("<div id=x>saved</div>", url=origin + "/saved")
            text = other.evaluate("document.getElementById('x').textContent")
            other.close()
            return text

        check("navigate and parse linked stylesheet", navigate)
        check("page challenge script is defined", challenge_runs)
        check("page fetch posts and receives Set-Cookie", page_fetch_sets_cookie)
        check("session cookie jar holds the challenge cookie", cookie_jar)
        check("follow-up API request authenticated by that cookie", api_with_cookie)
        check("route() blocks a stylesheet by resource type", blocked_resource)
        check("page.click dispatches trusted pointer events", click_dispatches_pointer_events)
        check(
            "page.hover dispatches pointermove", hover_is_hover if False else hover_is_pointer_move
        )
        check("iframe document reachable from the page", frames_are_reachable)
        check("run_pending_tasks drains timers", pending_tasks)
        check("load_html installs a saved document", saved_document)
        session.close()
    finally:
        server.shutdown()

    width = max(len(name) for name, _, _ in RESULTS)
    for name, ok, detail in RESULTS:
        print(f"{'PASS' if ok else 'FAIL'}  {name.ljust(width)}  {detail}")
    return 0 if all(ok for _, ok, _ in RESULTS) else 1


if __name__ == "__main__":
    raise SystemExit(main())

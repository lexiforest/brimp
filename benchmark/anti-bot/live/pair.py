"""Run each example target in Brimp and in the Safari on this machine.

The question the pyv8c examples answer is whether the site lets you through, and
that only means something next to a real browser at the same moment: an IP can be
challenged, rate-limited or already trusted, and then no client passes or every
client does. Safari is driven through `do JavaScript`, which does not set
`navigator.webdriver`.
"""

from __future__ import annotations

import json
import subprocess
import tempfile
import time

import brimp

TARGETS = [
    ("cloudflare/5s", "https://steamdb.info/login/"),
    ("akamai", "https://www.radissonhotels.com/zh-cn/"),
    ("datadome", "https://www.neimanmarcus.com/"),
    ("f5shape", "https://www.southwest.com/"),
    ("reese84", "https://orion.nv.gov/portal/public/"),
]

STATE = """JSON.stringify({
  url: location.href.slice(0, 80),
  title: (document.title || '').slice(0, 40),
  text: (document.body?.innerText || '').replace(/\\s+/g, ' ').trim().slice(0, 60),
  cookieNames: document.cookie.split('; ').filter(Boolean).map((part) => part.split('=')[0]).slice(0, 8),
})"""


def osascript(expression: str) -> str:
    completed = subprocess.run(["osascript", "-e", expression], capture_output=True, text=True)
    return completed.stdout.strip() or completed.stderr.strip()


def safari_state(url: str, wait: float) -> dict:
    osascript(f'tell application "Safari" to set URL of front document to "{json.dumps(url)}"')
    time.sleep(wait)
    handle = tempfile.NamedTemporaryFile("w", suffix=".js", delete=False)
    handle.write(STATE)
    handle.close()
    raw = osascript(
        f'tell application "Safari" to do JavaScript (read file (POSIX file "{handle.name}" as text) '
        "as «class utf8») in front document"
    )
    try:
        return json.loads(raw)
    except json.JSONDecodeError:
        return {"error": raw[:120]}


def brimp_state(url: str, wait: float) -> dict:
    try:
        return _brimp_state(url, wait)
    except Exception as error:  # noqa: BLE001 - a failure is a result here
        return {"error": f"{type(error).__name__}: {str(error)[:100]}"}


def _brimp_state(url: str, wait: float) -> dict:
    with brimp.Session() as session:
        page = session.get(url, timeout=60)
        deadline = time.monotonic() + wait
        while time.monotonic() < deadline:
            page.run_pending_tasks()
            time.sleep(0.25)
        state = json.loads(page.evaluate(STATE))
        state["status"] = page.status_code
        state["cookies"] = sorted(record["name"] for record in session.cookies)[:10]
        return state


def main() -> int:
    wait = 20.0
    results = []
    for name, url in TARGETS:
        brimp_seen = brimp_state(url, wait)
        try:
            safari_seen = safari_state(url, wait)
        except Exception as error:  # noqa: BLE001
            safari_seen = {"error": f"{type(error).__name__}: {str(error)[:100]}"}
        results.append({"target": name, "url": url, "brimp": brimp_seen, "safari": safari_seen})
        print(
            f"{name:<14} brimp {str(brimp_seen.get('status') or brimp_seen.get('error'))[:38]:<40}"
            f" title={brimp_seen.get('title')!r:<26} |"
            f" safari title={safari_seen.get('title')!r:<26}"
            f" same={brimp_seen.get('title') == safari_seen.get('title')}"
        )
    with open("live/pair.json", "w", encoding="utf-8") as handle:
        json.dump(results, handle, indent=1, ensure_ascii=False)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

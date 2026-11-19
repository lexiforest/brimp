"""Log every request a challenge page makes, and the cookies it ends up with."""
from __future__ import annotations

import argparse
import json
import time

import brimp


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("url")
    parser.add_argument("--seconds", type=float, default=25)
    arguments = parser.parse_args()

    seen = []
    with brimp.Session() as session:
        def policy(route):
            request = route.request
            seen.append([request["resource_type"], request["method"], request["url"][:120]])
            route.continue_request()

        remove = session.route("*", policy)
        page = session.get(arguments.url, timeout=45)
        started = time.monotonic()
        while time.monotonic() - started < arguments.seconds:
            page.run_pending_tasks()
            time.sleep(0.25)
        remove()
        cookies = {record["name"]: record["value"][:40] for record in session.cookies}
        print(json.dumps({
            "status": page.status_code,
            "title": page.evaluate("document.title"),
            "cookies": cookies,
            "requests": seen,
        }, indent=1)[:4000])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

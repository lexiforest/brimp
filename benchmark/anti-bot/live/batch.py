"""Run every target from the pyv8c examples and report what Brimp observed."""

from __future__ import annotations

import json
import time

import brimp

TARGETS = [
    ("cloudflare/5s", "https://steamdb.info/login/", "cf_clearance"),
    ("akamai", "https://www.radissonhotels.com/zh-cn/", "_abck"),
    ("datadome", "https://www.neimanmarcus.com/", "datadome"),
    ("f5shape", "https://www.southwest.com/", "f5avr"),
    ("reese84", "https://orion.nv.gov/portal/public/", "reese84"),
    ("px3", "https://www.skyscanner.com.au/sttc/px/captcha-v2/index.html", "_px3"),
]


def run(name, url, marker, seconds=25.0):
    record = {"target": name, "url": url, "marker": marker}
    started = time.monotonic()
    try:
        with brimp.Session() as session:
            page = session.get(url, timeout=60)
            record["status"] = page.status_code
            record["title"] = page.evaluate("document.title")
            record["ua_ok"] = page.evaluate("navigator.userAgent").startswith(
                "Mozilla/5.0 (Macintosh"
            )
            deadline = started + seconds
            while time.monotonic() < deadline:
                page.run_pending_tasks()
                time.sleep(0.25)
            record["cookies"] = {r["name"]: r["value"][:24] for r in session.cookies}
            record["final_title"] = page.evaluate("document.title")
            record["body"] = (page.evaluate("document.body?.innerText || ''") or "")[:120].replace(
                "\n", " "
            )
            # The marker cookie can also be the *pre-challenge* one; report both.
            values = {r["name"]: r["value"] for r in session.cookies}
            record["marker_present"] = any(key.startswith(marker) for key in values)
            record["akamai_pass"] = "~0~" in values.get("_abck", "")
    except Exception as error:  # noqa: BLE001 - a failure is a result here
        record["error"] = f"{type(error).__name__}: {str(error)[:120]}"
    record["elapsed"] = round(time.monotonic() - started, 1)
    return record


def main() -> int:
    results = [run(*target) for target in TARGETS]
    for record in results:
        marker = "PASS" if record.get("marker_present") else "no "
        extra = " (akamai ~0~)" if record.get("akamai_pass") else ""
        print(
            f"{marker}  {record['target']:<14} status={record.get('status')} "
            f"{record.get('elapsed')}s  cookies={sorted(record.get('cookies', {}))}  "
            f"title={(record.get('final_title') or record.get('error') or '')[:40]!r}{extra}"
        )
    with open("live/results.json", "w", encoding="utf-8") as handle:
        json.dump(results, handle, indent=1)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

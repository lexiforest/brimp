"""Load a live challenge page in Brimp and report which challenge cookies appeared.

The pyv8c examples each end with one question: did the site's own challenge script
run and leave its cookie? This asks exactly that, without the parts of those
scripts that Brimp has no API for yet (coordinate mouse, press-and-hold).
"""
from __future__ import annotations

import argparse
import json
import time

import brimp

# The cookie each vendor sets once its client-side challenge has been accepted.
MARKERS = {
    "akamai": lambda name, value: name == "_abck" and "~0~" in value,
    "cloudflare": lambda name, value: name == "cf_clearance",
    "datadome": lambda name, value: name == "datadome",
    "f5": lambda name, value: name.startswith("f5avr") or name.startswith("TS"),
    "incapsula": lambda name, value: name in {"reese84", "nlbi_1", "visid_incap_1"},
    "px": lambda name, value: name.startswith("_px"),
}


def cookies(session):
    return {record["name"]: record["value"] for record in session.cookies}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("url")
    parser.add_argument("--seconds", type=float, default=20)
    parser.add_argument("--json", default=None)
    parser.add_argument("--headers", action="store_true")
    arguments = parser.parse_args()

    observed = {"url": arguments.url, "pumps": 0, "events": []}
    started = time.monotonic()
    with brimp.Session() as session:
        page = session.get(arguments.url, timeout=45)
        observed["status"] = page.status_code
        observed["final_url"] = page.url
        if arguments.headers:
            observed["headers"] = dict(page.headers)
        observed["initial_cookies"] = cookies(session)
        observed["title"] = page.evaluate("document.title")
        observed["ua"] = page.evaluate("navigator.userAgent")
        deadline = started + arguments.seconds
        previous = set(observed["initial_cookies"])
        while time.monotonic() < deadline:
            page.run_pending_tasks()
            time.sleep(0.25)
            observed["pumps"] += 1
            current = cookies(session)
            for name in sorted(set(current) - previous):
                observed["events"].append({"cookie": name, "value": current[name][:80]})
            previous = set(current)
        observed["cookies"] = cookies(session)
        observed["final_url"] = page.url
        observed["final_title"] = page.evaluate("document.title")
        observed["elapsed"] = round(time.monotonic() - started, 1)
        try:
            observed["body_head"] = page.evaluate("document.body.innerText.slice(0, 200)")
        except Exception as error:  # noqa: BLE001
            observed["body_head"] = f"{type(error).__name__}: {error}"

    matched = [
        vendor
        for vendor, predicate in MARKERS.items()
        if any(predicate(name, value) for name, value in observed["cookies"].items())
    ]
    observed["passed_markers"] = matched
    print(json.dumps(observed, indent=1)[:3000])
    if arguments.json:
        with open(arguments.json, "w", encoding="utf-8") as handle:
            json.dump(observed, handle, indent=1)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

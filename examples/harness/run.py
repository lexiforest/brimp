"""Run from the checkout with PYTHONPATH=bindings/python/python."""

import json
from pathlib import Path

from brimp import Session


def fixture(route):
    if route.request["url"] == "https://fixture.invalid/config":
        route.fulfill(
            headers={"content-type": "application/json", "access-control-allow-origin": "*"},
            body=b'{"prefix":"Hello "}',
        )
    else:
        route.fail("No fixture for this request")


source = Path(__file__).with_name("page.js").read_text()
with Session(time_mode="logical", time_origin=1000, diagnostics={"capacity": 4096}) as session:
    page = session.new_page()
    page.route("*", fixture)
    with page.evaluate_handle(source + "\n;compute") as fn:
        page.click("#run")
        page.take_diagnostics()
        results = []
        for name in ["Ada", "Grace"]:
            with fn.call("(fn, input) => fn(input)", name) as result:
                results.append(result.json_value())
        assert results == [
            dict(text="Hello Ada", calls=1, clicks=1, now=1005),
            dict(text="Hello Grace", calls=2, clicks=1, now=1010),
        ]
        snapshot = page.peek_diagnostics()
        report = page.summarize_diagnostics()
        assert report["retained"] == len(snapshot["records"])
        assert page.take_diagnostics() == snapshot
        print(json.dumps(dict(results=results, report=report), indent=2))

import json
import sys
import tempfile
from pathlib import Path

import brimp


def main(url):
    session = brimp.Session()
    page = session.new_page()
    session.headers["X-Binding"] = "python"
    session.cookies["manual"] = "yes"
    response = page.get(url, params={"q": ["one", "two"]})
    title = page.evaluate("document.title")
    value = page.evaluate("({answer: 42, values: [true, null]})")
    page.hover("#submit")
    page.click("#submit")
    page.type("#name", "agent")
    page.tap("#tap")
    input_result = page.evaluate("({value: document.querySelector('#name').value, events: inputEvents})")
    evaluation_errors = {}
    for name, expression in {
        "javascript": "throw new Error('boom')",
        "unsupported": "undefined",
    }.items():
        try:
            page.evaluate(expression)
        except brimp.BrimpError as error:
            evaluation_errors[name] = error.code
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "page.png"
        screenshot = page.screenshot(path)
        png = path.read_bytes() == screenshot and screenshot.startswith(b"\x89PNG\r\n\x1a\n")
    inspection = page.get(url + "/inspect").json()
    result = {
        "title": title,
        "text": "Hello bindings" in response.html,
        "response": response.status_code == 200 and response.ok and "Hello bindings" in response.text,
        "query": response.url.endswith("?q=one&q=two"),
        "headers": response.headers["CONTENT-TYPE"] == "text/html; charset=utf-8",
        "state": inspection["header"] == "python" and "manual=yes" in inspection["cookie"] and "server=ready" in inspection["cookie"],
        "value": value,
        "input": input_result["value"] == "agent" and all(event["trusted"] for event in input_result["events"]),
        "hover": any(event["id"] == "submit" and event["type"] == "pointermove" for event in input_result["events"]),
        "touch": any(event["id"] == "tap" and event["type"] == "click" and event["pointerType"] == "touch" for event in input_result["events"]),
        "png": png,
        "oneShot": brimp.get(url).status_code == 200,
    }
    result.update(evaluation_errors)
    missing = page.get(url + "/missing")
    try:
        missing.raise_for_status()
    except brimp.HTTPError as error:
        result["http"] = error.response is missing
    try:
        page.get(url + "/hang", timeout=0.05)
    except brimp.Timeout as error:
        result["timeout"] = error.code == "timeout"
    session.close()
    session.close()
    try:
        page.evaluate("document.title")
    except brimp.BrimpError as error:
        result["closed"] = error.code
    print(json.dumps(result, sort_keys=True))


main(sys.argv[1])

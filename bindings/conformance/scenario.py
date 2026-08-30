import json
import sys
import tempfile
from pathlib import Path

import brimp


def main(url):
    session = brimp.Session()
    session.headers["X-Binding"] = "python"
    session.cookies["manual"] = "yes"
    response = session.get(url, params={"q": ["one", "two"]})
    title = session.evaluate("document.title")
    value = session.evaluate("({answer: 42, values: [true, null]})")
    evaluation_errors = {}
    for name, expression in {
        "javascript": "throw new Error('boom')",
        "unsupported": "undefined",
    }.items():
        try:
            session.evaluate(expression)
        except brimp.BrimpError as error:
            evaluation_errors[name] = error.code
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "page.png"
        screenshot = session.screenshot(path)
        png = path.read_bytes() == screenshot and screenshot.startswith(b"\x89PNG\r\n\x1a\n")
    inspection = session.get(url + "/inspect").json()
    result = {
        "title": title,
        "text": "Hello bindings" in response.html,
        "response": response.status_code == 200 and response.ok and "Hello bindings" in response.text,
        "query": response.url.endswith("?q=one&q=two"),
        "headers": response.headers["CONTENT-TYPE"] == "text/html; charset=utf-8",
        "state": inspection["header"] == "python" and "manual=yes" in inspection["cookie"] and "server=ready" in inspection["cookie"],
        "value": value,
        "png": png,
        "oneShot": brimp.get(url).status_code == 200,
    }
    result.update(evaluation_errors)
    missing = session.get(url + "/missing")
    try:
        missing.raise_for_status()
    except brimp.HTTPError as error:
        result["http"] = error.response is missing
    try:
        session.get(url + "/hang", timeout=0.05)
    except brimp.Timeout as error:
        result["timeout"] = error.code == "timeout"
    session.close()
    session.close()
    try:
        session.evaluate("document.title")
    except brimp.BrimpError as error:
        result["closed"] = error.code
    print(json.dumps(result, sort_keys=True))


main(sys.argv[1])

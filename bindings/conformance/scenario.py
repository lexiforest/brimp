import json
import sys
import tempfile
from pathlib import Path

import brimp


def main(url):
    session = brimp.Session(params={"base": "yes", "q": "old"})
    page = session.new_page()
    session.headers["X-Binding"] = "python"
    session.cookies["manual"] = "yes"
    response = page.get(url, params={"q": ["one", "two"]})
    initial_text = "Hello bindings" in response.html
    initial_response = response.status_code == 200 and response.ok and "Hello bindings" in response.text
    initial_query = response.url.endswith("?base=yes&q=one&q=two")
    initial_headers = response.headers["CONTENT-TYPE"] == "text/html; charset=utf-8"
    initial_transfer = (
        response.http_version.startswith("HTTP/")
        and response.downloaded_bytes == len(response.content)
        and response.header_bytes > 0
    )
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
        except brimp.RequestError as error:
            evaluation_errors[name] = error.code
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "page.png"
        screenshot = page.screenshot(path)
        png = path.read_bytes() == screenshot and screenshot.startswith(b"\x89PNG\r\n\x1a\n")
    inspection = page.get(url + "/inspect").json()
    result = {
        "title": title,
        "text": initial_text,
        "response": initial_response,
        "query": initial_query,
        "headers": initial_headers,
        "transfer": initial_transfer,
        "state": inspection["header"] == "python" and "manual=yes" in inspection["cookie"] and "server=ready" in inspection["cookie"],
        "value": value,
        "input": input_result["value"] == "agent" and all(event["trusted"] for event in input_result["events"]),
        "hover": any(event["id"] == "submit" and event["type"] == "pointermove" for event in input_result["events"]),
        "touch": any(event["id"] == "tap" and event["type"] == "click" and event["pointerType"] == "touch" for event in input_result["events"]),
        "png": png,
    }
    with brimp.get(url) as one_shot:
        result["oneShot"] = one_shot.status_code == 200
    page.post(url + "/echo", json={"binding": "python"})
    posted = page.json()
    result["post"] = posted["method"] == "POST" and posted["body"] == '{"binding":"python"}'
    page.get(url + "/redirect")
    result["redirect"] = (
        page.url == url + "/final"
        and page.redirect_count == 1
        and page.history[0].status_code == 302
    )
    result.update(evaluation_errors)
    page.get(url + "/missing")
    try:
        page.raise_for_status()
    except brimp.HTTPError as error:
        result["http"] = error.page is page
    try:
        page.get(url + "/hang", timeout=0.05)
    except brimp.Timeout as error:
        result["timeout"] = error.code == "timeout"
    session.close()
    session.close()
    try:
        page.evaluate("document.title")
    except brimp.RequestError as error:
        result["closed"] = error.code
    print(json.dumps(result, sort_keys=True))


main(sys.argv[1])

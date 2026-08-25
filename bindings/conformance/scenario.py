import json
import sys

import brimp


def main(url):
    session = brimp.Session()
    response = session.get(url)
    result = {
        "title": session.evaluate("document.title"),
        "text": "Hello bindings" in response.html,
        "value": session.evaluate("({answer: 42, values: [true, null]})"),
        "png": session.screenshot().startswith(b"\x89PNG\r\n\x1a\n"),
    }
    for name, expression in {
        "javascript": "throw new Error('boom')",
        "unsupported": "undefined",
    }.items():
        try:
            session.evaluate(expression)
        except brimp.BrimpError as error:
            result[name] = error.code
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

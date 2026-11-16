"""Buffered body inputs across sync, async and streaming entry points."""

import asyncio
import io
import json
import threading
from collections import UserDict
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlencode

import pytest

import brimp


@pytest.fixture(scope="module")
def server():
    seen = []

    class Handler(BaseHTTPRequestHandler):
        def do_POST(self):
            body = self.rfile.read(int(self.headers.get("Content-Length", "0")))
            seen.append((self.command, self.path, body))
            if self.path.startswith("/redirect/"):
                self.send_response(int(self.path.rsplit("/", 1)[1]))
                self.send_header("Location", "/echo")
                self.send_header("Content-Length", "0")
                self.end_headers()
                return
            payload = json.dumps(
                {
                    "method": self.command,
                    "body": list(body),
                    "content_types": self.headers.get_all("Content-Type", []),
                }
            ).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

        do_GET = do_PUT = do_PATCH = do_DELETE = do_QUERY = do_POST

        def log_message(self, *_):
            pass

    httpd = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=httpd.serve_forever, daemon=True)
    thread.start()
    yield f"http://127.0.0.1:{httpd.server_port}", seen
    httpd.shutdown()
    httpd.server_close()
    thread.join()


CASES = [
    (
        {"json": {"name": "雪", "flag": True, "n": None}},
        b'{"name":"\\u96ea","flag":true,"n":null}',
        "application/json",
    ),
    ({"json": [1, "a", False]}, b'[1,"a",false]', "application/json"),
    ({"json": False}, b"false", "application/json"),
    ({"json": 0}, b"0", "application/json"),
    ({"json": ""}, b'""', "application/json"),
    ({"json": {}}, b"{}", "application/json"),
    ({"data": "snow=雪"}, "snow=雪".encode(), "application/octet-stream"),
    ({"data": b"\x00\xffpayload"}, b"\x00\xffpayload", "application/octet-stream"),
    ({"content": "雪"}, "雪".encode(), "application/octet-stream"),
    ({"content": bytearray(b"\x00\xff")}, b"\x00\xff", "application/octet-stream"),
    ({"content": b""}, b"", "application/octet-stream"),
    ({"data": ""}, b"", "application/octet-stream"),
    (
        {"data": {"a": "a b", "n": [1, 2]}},
        urlencode({"a": "a b", "n": [1, 2]}).encode(),
        "application/x-www-form-urlencoded",
    ),
    ({"data": [("x", "1"), ("x", "2")]}, b"x=1&x=2", "application/x-www-form-urlencoded"),
    ({"data": UserDict({"x": "y"})}, b"x=y", "application/x-www-form-urlencoded"),
    ({"data": {}}, b"", "application/x-www-form-urlencoded"),
    ({"data": "ignored", "content": b"chosen"}, b"chosen", "application/octet-stream"),
    ({"data": "ignored", "content": b"ignored", "json": False}, b"false", "application/json"),
    ({"json": None, "content": None, "data": "chosen"}, b"chosen", "application/octet-stream"),
]


@pytest.mark.parametrize("kwargs,expected,mime", CASES)
@pytest.mark.parametrize("stream", [False, True])
def test_encoded_body_reaches_server(server, kwargs, expected, mime, stream):
    url, _ = server
    with brimp.Session(plugins=[], trust_env=False) as session:
        page = session.new_page()
        if stream:
            with page.stream("POST", url, **kwargs) as response:
                result = json.loads(b"".join(response))
        else:
            page.post(url, **kwargs)
            result = page.json()
            assert page.request_info["body"] == expected
        assert bytes(result["body"]) == expected
        assert result["content_types"] == [mime]


@pytest.mark.parametrize("kind", ["json", "content", "data"])
@pytest.mark.parametrize("stream", [False, True])
def test_explicit_content_type_overrides_defaults(server, kind, stream):
    url, _ = server
    kwargs = {kind: {"a": 1} if kind == "json" else "text"}
    with brimp.Session(
        headers={"cOnTeNt-TyPe": "session/type"}, plugins=[], trust_env=False
    ) as session:
        page = session.new_page()
        for headers, expected in [
            (None, "session/type"),
            ({"CONTENT-TYPE": "request/type"}, "request/type"),
        ]:
            if stream:
                with page.stream("POST", url, headers=headers, **kwargs) as response:
                    result = json.loads(b"".join(response))
            else:
                result = page.post(url, headers=headers, **kwargs).json()
            assert result["content_types"] == [expected]


@pytest.mark.parametrize("code", [302, 303, 307, 308])
@pytest.mark.parametrize("stream", [False, True])
def test_redirects_replay_or_drop_encoded_body(server, code, stream):
    url, _ = server
    with brimp.Session(plugins=[], trust_env=False) as session:
        page = session.new_page()
        if stream:
            with page.stream("POST", f"{url}/redirect/{code}", json={"x": 1}) as response:
                result = json.loads(b"".join(response))
        else:
            result = page.post(f"{url}/redirect/{code}", json={"x": 1}).json()
        if code in (307, 308):
            assert result["method"] == "POST"
            assert bytes(result["body"]) == b'{"x":1}'
            assert result["content_types"] == ["application/json"]
        else:
            assert result["method"] == "GET"
            assert result["body"] == []
            assert result["content_types"] == []


@pytest.mark.parametrize("option", ["data", "content", "json"])
@pytest.mark.parametrize("stream", [False, True])
def test_body_conflicts_fail_before_network(server, option, stream):
    url, seen = server
    count = len(seen)
    with brimp.Session(plugins=[], trust_env=False) as session:
        page = session.new_page()
        with pytest.raises(brimp.InvalidInput, match="specify body"):
            if stream:
                page.stream("POST", url, body=b"", **{option: ""})
            else:
                page.post(url, body=b"", **{option: ""})
    assert len(seen) == count


@pytest.mark.parametrize("option", ["data", "content", "body"])
def test_file_and_iterable_uploads_are_not_consumed(server, option):
    url, seen = server
    count = len(seen)
    consumed = []

    def chunks():
        consumed.append(True)
        yield b"file bytes"

    with brimp.Session(plugins=[], trust_env=False) as session:
        page = session.new_page()
        source = io.BytesIO(b"file bytes")
        for value in (source, chunks()):
            for stream in (False, True):
                with pytest.raises(TypeError, match="uploads are unsupported"):
                    if stream:
                        page.stream("POST", url, **{option: value})
                    else:
                        page.post(url, **{option: value})
        assert source.tell() == 0
        assert consumed == []
    assert len(seen) == count


@pytest.mark.parametrize("option", ["files", "multipart"])
def test_multipart_options_remain_unsupported(server, option):
    url, seen = server
    count = len(seen)
    with pytest.raises(TypeError, match=option):
        brimp.post(url, **{option: {"file": b"contents"}})
    assert len(seen) == count


def test_invalid_json_fails_without_sending(server):
    url, seen = server
    count = len(seen)
    with brimp.Session(plugins=[], trust_env=False) as session:
        with pytest.raises(TypeError):
            session.post(url, json={"object": object()})
    assert len(seen) == count


@pytest.mark.parametrize("method", ["post", "put", "patch", "delete", "query"])
def test_module_helpers_encode_json(server, method):
    url, _ = server
    with getattr(brimp, method)(url, json={"x": 1}) as page:
        assert bytes(page.json()["body"]) == b'{"x":1}'
        assert page.json()["method"] == method.upper()


def test_async_uses_shared_encoding_and_errors(server):
    url, _ = server

    async def run():
        async with brimp.AsyncSession(plugins=[], trust_env=False) as session:
            pages = await asyncio.gather(
                *(
                    session.post(url, **kwargs)
                    for kwargs in [{"json": {"x": 1}}, {"data": b"\x00\xff"}, {"content": "雪"}]
                )
            )
            assert [bytes(page.json()["body"]) for page in pages] == [
                b'{"x":1}',
                b"\x00\xff",
                "雪".encode(),
            ]
            assert bytes((await pages[0].put(url, content="again")).json()["body"]) == b"again"
            with pytest.raises(TypeError, match="uploads are unsupported"):
                await session.post(url, data=io.BytesIO(b"file"))

    asyncio.run(run())

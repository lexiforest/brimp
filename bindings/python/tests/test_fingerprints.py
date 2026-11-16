"""Fingerprint data, profile application and CLI-owned Pro catalog lifecycle."""

import asyncio
from dataclasses import asdict
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import subprocess
import socket
import struct
import threading
from urllib.parse import parse_qs, urlsplit

import pytest

import brimp


@pytest.fixture
def environment(tmp_path, monkeypatch):
    monkeypatch.setenv("IMPERSONATE_CONFIG_DIR", str(tmp_path))
    monkeypatch.setenv("IMPERSONATE_API_KEY", "fixture-key")
    monkeypatch.setenv("NO_PROXY", "127.0.0.1,localhost")
    return tmp_path


@pytest.fixture
def server():
    state = {"seen": [], "mode": "ok"}

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):
            state["seen"].append((self.path, self.headers.get("Authorization")))
            if self.path.startswith("/v1/fingerprints"):
                if state["mode"] == "unauthorized":
                    self.send_response(401)
                    self.end_headers()
                    return
                skip = int(parse_qs(urlsplit(self.path).query)["skip"][0])
                payload = {
                    "items": [
                        {
                            "name": "fixture" + str(skip),
                            "data": {
                                "client": "chrome",
                                "client_version": "150",
                                "headers": {
                                    "User-Agent": "FixtureBrowser/1",
                                    "X-Fingerprint": "pro",
                                },
                            },
                        }
                    ]
                }
                if skip == 0:
                    payload["pagination"] = {"has_more": True, "next_skip": 100}
                if state["mode"] == "bad-pagination":
                    payload["pagination"] = {"has_more": True, "next_skip": 0}
                if state["mode"] == "bad-data":
                    payload["items"][0]["data"]["tls_ciphers"] = 123
            else:
                payload = {key.lower(): value for key, value in self.headers.items()}
            body = json.dumps(payload).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *_):
            pass

    httpd = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    worker = threading.Thread(target=httpd.serve_forever, daemon=True)
    worker.start()
    state["url"] = f"http://127.0.0.1:{httpd.server_port}"
    yield state
    httpd.shutdown()
    httpd.server_close()
    worker.join()


def cli(*args, check=True):
    root = Path(__file__).resolve().parents[3]
    binary = Path(
        os.environ.get(
            "BRIMP_CLI", root / "target/debug" / ("brimp.exe" if os.name == "nt" else "brimp")
        )
    )
    result = subprocess.run(
        [str(binary), "fingerprints", *args], text=True, capture_output=True, timeout=30
    )
    if check:
        assert result.returncode == 0, result.stderr
    return result


def test_dataclass_defaults_and_independent_mutability():
    first, second = brimp.Fingerprint(), brimp.Fingerprint()
    first.headers["X-Test"] = "one"
    first.tls_ciphers.append("AES128-SHA")
    assert second.headers == {} and second.tls_ciphers == []
    assert second.tls_alpn is True
    assert second.tls_signed_cert_timestamps is False
    assert second.tls_trust_anchors is None
    assert brimp.Fingerprint(**asdict(first)) == first
    assert not hasattr(brimp, "FingerprintManager")


@pytest.mark.parametrize("nested", [False, True])
def test_profile_uses_fingerprint_headers_and_identity(server, nested):
    fp = brimp.Fingerprint(
        headers={
            "User-Agent": "FixtureBrowser/1",
            "X-Fingerprint": "custom",
            "Accept-Language": "fr-FR,fr;q=0.8",
            "Host": "",
        }
    )
    profile = {"values": {"network": {"fingerprint": fp}}} if nested else fp
    with brimp.Session(profile=profile, trust_env=False, plugins=[]) as session:
        page = session.get(server["url"])
        headers = page.json()
        assert headers["user-agent"] == "FixtureBrowser/1"
        assert headers["x-fingerprint"] == "custom"
        assert headers["accept-language"] == "fr-FR,fr;q=0.8"
        assert headers["host"].startswith("127.0.0.1:")
        assert page.evaluate("navigator.userAgent") == "FixtureBrowser/1"
        page.get(server["url"], headers={"X-Fingerprint": "request"})
        assert page.json()["x-fingerprint"] == "request"


def test_explicit_identity_overrides_fingerprint(server):
    fp = brimp.Fingerprint(headers={"User-Agent": "Fingerprint/1", "Accept": "application/json"})
    profile = {"values": {"identity": {"user_agent": "Explicit/1"}, "network": {"fingerprint": fp}}}
    with brimp.Session(profile=profile, plugins=[], trust_env=False) as session:
        page = session.get(server["url"])
        assert page.json()["user-agent"] == page.evaluate("navigator.userAgent") == "Explicit/1"


def test_async_profile(server):
    async def run():
        async with brimp.AsyncSession(
            profile=brimp.Fingerprint(headers={"X-Fingerprint": "async"}),
            plugins=[],
            trust_env=False,
        ) as session:
            page = await session.get(server["url"])
            assert page.json()["x-fingerprint"] == "async"

    asyncio.run(run())


def test_cli_update_list_and_export(environment, server):
    # The explicit environment key wins even over an unreadable stored config.
    (environment / "config.json").write_text("invalid config")
    result = cli("update", "--api-root", server["url"] + "/v1")
    assert "Updated 2 fingerprints" in result.stdout
    assert server["seen"] == [
        ("/v1/fingerprints?skip=0&limit=100", "Bearer fixture-key"),
        ("/v1/fingerprints?skip=100&limit=100", "Bearer fixture-key"),
    ]
    assert len(json.loads(cli("list").stdout)) == 2
    fp = brimp.Fingerprint(**json.loads(cli("show", "fixture0").stdout))
    assert fp.client == "chrome"
    profile = json.loads(cli("show", "fixture100", "--profile").stdout)
    with brimp.Session(profile=profile, plugins=[], trust_env=False) as session:
        assert session.get(server["url"]).json()["x-fingerprint"] == "pro"


@pytest.mark.parametrize("mode", ["unauthorized", "bad-pagination", "bad-data"])
def test_failed_update_preserves_catalog(environment, server, mode):
    path = environment / "fingerprints.json"
    path.write_text('{"old": {}}')
    server["mode"] = mode
    result = cli("update", "--api-root", server["url"] + "/v1", check=False)
    assert result.returncode != 0
    assert path.read_text() == '{"old": {}}'
    assert "fixture-key" not in result.stderr


def test_config_and_environment_key_precedence(environment, server, monkeypatch):
    assert "stored-key" not in cli("config", "--api-key", "stored-key").stdout
    config = environment / "config.json"
    assert json.loads(config.read_text())["api_key"] == "stored-key"
    if os.name != "nt":
        assert config.stat().st_mode & 0o077 == 0
    monkeypatch.delenv("IMPERSONATE_API_KEY")
    cli("update", "--api-root", server["url"] + "/v1")
    assert server["seen"][0][1] == "Bearer stored-key"


def test_runtime_does_not_read_catalog(environment, server):
    (environment / "fingerprints.json").write_text("invalid catalog")
    (environment / "config.json").write_text("invalid config")
    with brimp.Session(plugins=[], trust_env=False) as session:
        assert session.get(server["url"]).status_code == 200


@pytest.mark.parametrize(
    "fingerprint",
    [{"tls_ciphers": 1}, {"http_version": "invalid"}, {"headers": {"X-Test": "bad\r\nvalue"}}],
)
def test_invalid_profile_fingerprint(fingerprint):
    with pytest.raises(brimp.BrimpError):
        brimp.Session(profile={"values": {"network": {"fingerprint": fingerprint}}}, plugins=[])


@pytest.mark.parametrize("alpn,sct", [(False, False), (True, True)])
def test_fingerprint_changes_tls_client_hello(alpn, sct):
    captured = []
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        listener.listen()
        listener.settimeout(5)

        def receive():
            with listener.accept()[0] as connection:
                connection.settimeout(5)
                with connection.makefile("rb") as source:
                    record = source.read(5)
                    assert record[0] == 22
                    captured.append(source.read(int.from_bytes(record[3:5], "big")))

        worker = threading.Thread(target=receive)
        worker.start()
        fp = brimp.Fingerprint(
            tls_ciphers=["ECDHE-RSA-AES256-GCM-SHA384"],
            tls_supported_groups=["P-256"],
            tls_alpn=alpn,
            tls_signed_cert_timestamps=sct,
        )
        with brimp.Session(profile=fp, plugins=[], trust_env=False, timeout=3) as session:
            with pytest.raises(brimp.RequestException):
                session.get(f"https://127.0.0.1:{listener.getsockname()[1]}")
        worker.join(5)
        assert not worker.is_alive()
    hello = captured[0]
    assert hello[0] == 1
    offset = 39 + hello[38]
    size = int.from_bytes(hello[offset : offset + 2], "big")
    ciphers = [
        value[0] for value in struct.iter_unpack("!H", hello[offset + 2 : offset + 2 + size])
    ]
    assert 0xC030 in ciphers and 0xC02F not in ciphers
    offset += 2 + size
    offset += 1 + hello[offset]
    size = int.from_bytes(hello[offset : offset + 2], "big")
    offset += 2
    end = offset + size
    extensions = {}
    while offset < end:
        kind, size = struct.unpack("!HH", hello[offset : offset + 4])
        offset += 4
        extensions[kind] = hello[offset : offset + size]
        offset += size
    assert (16 in extensions) is alpn
    assert (18 in extensions) is sct
    assert extensions[10] == b"\x00\x02\x00\x17"  # P-256 only.


def test_advanced_protocol_options_are_accepted():
    fp = brimp.Fingerprint(
        tls_version="",
        http3_settings="1:65536;6:262144",
        http3_pseudo_headers_order="m,a,s,p",
        http3_tls_extension_order="0-10-13-16-43-51",
        http3_headers={"X-H3": "three"},
        http3_header_order="x-h3",
        http3_tls_supported_groups=["X25519", "P-256"],
        quic_transport_parameters="1:30000;3:1472;4:10485760",
        quic_cid_length="firefox",
        quic_initial_packet_number=0,
        tls_trust_anchors=[],
        ws_headers={"X-WS": "socket"},
        ws_header_order="x-ws",
        ws_disable_session_ticket=True,
        ws_tls_cert_compression=[],
    )
    for _ in range(3):
        with brimp.Session(profile=fp, plugins=[]) as session:
            assert session.new_page().evaluate("42") == 42

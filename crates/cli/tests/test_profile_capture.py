"""Capture kit export, local submission validation and saved-profile reload."""

import json
import os
from pathlib import Path
import queue
import threading
import subprocess
from urllib.error import HTTPError
from urllib.parse import parse_qs, urlsplit
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[3]
CLI = Path(
    os.environ.get(
        "BRIMP_CLI", ROOT / "target/debug" / ("brimp.exe" if os.name == "nt" else "brimp")
    )
)


def test_export_only_contains_capture_code(tmp_path):
    plugin = tmp_path / "custom.js"
    plugin.write_text("""({global, profile}) => {
      global.runtimeInstallerSentinel = true;
      profile.values('custom', {answer: 42}, {
        capture: ({read}) => ({answer: read(() => 7)})
      });
    }""")
    kit = tmp_path / "kit"
    result = subprocess.run(
        [
            str(CLI),
            "profile",
            "capture",
            "--output",
            str(tmp_path / "profile.json"),
            "--directory",
            str(kit),
            "--no-serve",
            "--plugin",
            str(plugin),
        ],
        capture_output=True,
        timeout=20,
    )
    assert result.returncode == 0, result.stderr
    script = (kit / "capture.js").read_text()
    assert "runtimeInstallerSentinel" not in script
    assert "__brimp" not in script
    assert "read(() => 7)" in script
    assert "capture.js" in (kit / "index.html").read_text()
    assert not (tmp_path / "profile.json").exists()
    subprocess.run(["node", "--check", str(kit / "capture.js")], check=True, capture_output=True)


def test_local_capture_validates_then_saves_and_reloads(tmp_path):
    output = tmp_path / "profile.json"
    process = subprocess.Popen(
        [
            str(CLI),
            "profile",
            "capture",
            "--output",
            str(output),
            "--no-plugins",
            "--timeout",
            "20s",
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    try:
        # Windows selectors support sockets, not subprocess pipes.
        lines = queue.Queue()
        reader = threading.Thread(target=lambda: lines.put(process.stdout.readline()), daemon=True)
        reader.start()
        try:
            line = lines.get(timeout=12)
        except queue.Empty:
            raise AssertionError("capture server did not start") from None
        assert line.startswith("Open "), (line, process.poll())
        url = line.removeprefix("Open ").strip()
        parsed = urlsplit(url)
        origin = f"{parsed.scheme}://{parsed.netloc}"
        token = parse_qs(parsed.query)["token"][0]
        with urlopen(url, timeout=5) as response:
            assert b"Capture this browser" in response.read()
        with urlopen(origin + "/capture.js", timeout=5) as response:
            assert b"captureBrowserProfile" in response.read()

        def submit(profile, capture_token=token):
            request = Request(
                origin + "/profile",
                data=json.dumps(profile).encode(),
                headers={
                    "Content-Type": "application/json",
                    "Origin": origin,
                    "X-Brimp-Capture": capture_token,
                },
            )
            try:
                with urlopen(request, timeout=5) as response:
                    return response.status, json.load(response)
            except HTTPError as error:
                return error.code, json.load(error)

        assert submit({}, "wrong")[0] == 403
        assert submit({"values": {"identity": {"unknown": 1}}})[0] == 400
        assert submit({"values": {"audio": {"sample_rate": 48000}}})[0] == 400
        assert not output.exists()
        profile = {
            "format": "brimp-profile",
            "version": 1,
            "values": {"identity": {"user_agent": "Captured/1"}},
            "capture": {"results": {"identity.user_agent": {"status": "captured"}}},
        }
        assert submit(profile)[0] == 200
        assert process.wait(timeout=5) == 0
        assert json.loads(output.read_text()) == profile
        result = subprocess.run(
            [str(CLI), "run", "-", "--no-plugins", "--profile", str(output)],
            input="navigator.userAgent",
            text=True,
            capture_output=True,
            timeout=10,
        )
        assert result.returncode == 0, result.stderr
        assert "Captured/1" in result.stdout
    finally:
        if process.poll() is None:
            process.terminate()
        reader.join(timeout=5)
        process.communicate(timeout=5)

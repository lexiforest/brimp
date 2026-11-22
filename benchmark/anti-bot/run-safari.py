"""Run the same signal probe in the Safari on this machine.

This is the reference the profile claims, so a difference here is what a sensor
would notice. It drives Safari through `do JavaScript` rather than WebDriver:
`safaridriver` sets `navigator.webdriver` and installs its own automation hooks,
which changes the page it measures.
"""

from __future__ import annotations

import json
import pathlib
import subprocess
import sys
import tempfile
import time

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from server import serve  # noqa: E402

SIGNALS = (pathlib.Path(__file__).parent / "signals.js").read_text()


def osascript(expression: str) -> str:
    return subprocess.run(
        ["osascript", "-e", expression], capture_output=True, text=True, check=True
    ).stdout.strip()


def run_js(source: str) -> str:
    handle = tempfile.NamedTemporaryFile("w", suffix=".js", delete=False)
    handle.write(source)
    handle.close()
    try:
        return osascript(
            f'tell application "Safari" to do JavaScript (read file (POSIX file "{handle.name}" as text) '
            "as «class utf8») in front document"
        )
    finally:
        pathlib.Path(handle.name).unlink()


def main() -> int:
    server, port = serve()
    url = f"http://127.0.0.1:{port}/"
    try:
        osascript(f'tell application "Safari" to set URL of front document to "{url}"')
        for _ in range(60):
            if osascript('tell application "Safari" to get URL of front document').startswith(url):
                break
            time.sleep(0.25)
        time.sleep(1.5)
        run_js(f"(source => {{ eval(source); return true }})({json.dumps(SIGNALS)})")
        run_js("globalThis.signalsStart()")
        time.sleep(3.0)
        print(run_js("globalThis.signalsRead()"))
    finally:
        server.shutdown()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

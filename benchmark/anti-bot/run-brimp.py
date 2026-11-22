"""Run the signal probe in Brimp and print its JSON."""

from __future__ import annotations

import json
import pathlib
import sys

import brimp

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from server import serve  # noqa: E402

SIGNALS = (pathlib.Path(__file__).parent / "signals.js").read_text()


def main() -> int:
    server, port = serve()
    try:
        with brimp.Session() as session:
            page = session.get(f"http://127.0.0.1:{port}/")
            page.evaluate(f"(source => {{ eval(source); return true }})({json.dumps(SIGNALS)})")
            page.evaluate("globalThis.signalsStart()")
            # A real pause, not logical time: `advance_time` needs the virtual
            # clock a timing profile installs.
            import time

            time.sleep(2.6)
            print(page.evaluate("globalThis.signalsRead()"))
    finally:
        server.shutdown()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

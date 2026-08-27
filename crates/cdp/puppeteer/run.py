#!/usr/bin/env python3
import os
import subprocess
import sys


def main():
    if len(sys.argv) != 3:
        raise SystemExit("usage: run.py BRIMP_BINARY WORKFLOW_MJS")
    server = subprocess.Popen(
        [sys.argv[1], "cdp", "--bind", "127.0.0.1:0"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    try:
        websocket_url = server.stdout.readline().strip()
        if not websocket_url.startswith("ws://"):
            stderr = server.stderr.read()
            raise RuntimeError(f"CDP server did not report an endpoint: {websocket_url}\n{stderr}")
        browser_url = "http://" + websocket_url.removeprefix("ws://").split("/", 1)[0]
        env = os.environ.copy()
        env["BRIMP_CDP_URL"] = browser_url
        result = subprocess.run(["node", sys.argv[2]], env=env, text=True, capture_output=True, check=False)
        if result.returncode:
            raise RuntimeError(f"Puppeteer workflow failed ({result.returncode}):\n{result.stderr}")
        print(result.stdout.strip())
    finally:
        server.terminate()
        try:
            server.wait(timeout=5)
        except subprocess.TimeoutExpired:
            server.kill()
            server.wait()


if __name__ == "__main__":
    main()

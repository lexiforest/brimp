#!/usr/bin/env python3

import asyncio
import json
import os
import subprocess
import sys
from urllib.parse import urlsplit

from cdp_smoke_test import RawCDPClient, get_json


async def worker_identity(websocket_url, request_id=1):
    client = await RawCDPClient.connect(websocket_url)
    try:
        result = await client.command("Browser.getVersion")
        return client, result["workerPid"], result["path"]
    except Exception:
        client.writer.close()
        await client.writer.wait_closed()
        raise


async def run_test(controller_path):
    fake_worker = os.path.join(os.path.dirname(__file__), "fake_cdp_worker.py")
    process = subprocess.Popen(
        [
            controller_path,
            "serve",
            "--worker-protocol=cdp",
            f"--worker-path={sys.executable}",
            f"--worker-arg={fake_worker}",
            "--pool-size=2",
            "--port=0",
            "--operation-timeout=5",
            "--max-worker-memory-mb=0",
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    try:
        announcement = await asyncio.wait_for(
            asyncio.to_thread(process.stdout.readline), timeout=15
        )
        prefix = "Brimp CDP proxy listening on "
        if not announcement.startswith(prefix):
            raise RuntimeError(
                f"controller failed to start: {announcement!r} {process.stderr.read()!r}"
            )
        base_url = announcement[len(prefix) :].strip()

        first_version = await get_json(base_url, "/json/version")
        first, first_pid, first_path = await worker_identity(
            first_version["webSocketDebuggerUrl"]
        )
        if first_path != "/devtools/browser/fake":
            raise RuntimeError(f"browser route was not rewritten: {first_path}")
        same_result = await first.command("Browser.getVersion")
        if same_result["workerPid"] != first_pid:
            raise RuntimeError("one frontend WebSocket changed workers")

        second_version = await get_json(base_url, "/json/version")
        second, second_pid, _ = await worker_identity(
            second_version["webSocketDebuggerUrl"], 2
        )
        if second_pid == first_pid:
            raise RuntimeError("independent leases shared one worker")

        first.writer.close()
        await first.writer.wait_closed()
        second.writer.close()
        await second.writer.wait_closed()
        await asyncio.sleep(0.25)

        targets = await get_json(base_url, "/json/list")
        if len(targets) != 1:
            raise RuntimeError(f"unexpected target list: {targets}")
        direct, _, direct_path = await worker_identity(
            targets[0]["webSocketDebuggerUrl"], 3
        )
        if direct_path != "/devtools/page/fake-page":
            raise RuntimeError(f"page route was not rewritten: {direct_path}")
        direct.writer.close()
        await direct.writer.wait_closed()
    finally:
        process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: cdp_proxy_test.py /path/to/brimp")
    asyncio.run(run_test(sys.argv[1]))


if __name__ == "__main__":
    main()

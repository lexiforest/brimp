#!/usr/bin/env python3

import argparse
import base64
import hashlib
import json
import os
import socket
import struct
import sys
import threading
import time
from pathlib import Path
from urllib.parse import urlsplit


def receive_exactly(connection, length):
    data = bytearray()
    while len(data) < length:
        chunk = connection.recv(length - len(data))
        if not chunk:
            raise EOFError
        data.extend(chunk)
    return bytes(data)


def receive_frame(connection):
    header = receive_exactly(connection, 2)
    length = header[1] & 0x7F
    if length == 126:
        length = struct.unpack("!H", receive_exactly(connection, 2))[0]
    elif length == 127:
        length = struct.unpack("!Q", receive_exactly(connection, 8))[0]
    mask = receive_exactly(connection, 4) if header[1] & 0x80 else None
    payload = bytearray(receive_exactly(connection, length))
    if mask:
        for index in range(length):
            payload[index] ^= mask[index % 4]
    return header[0] & 0x0F, bytes(payload)


def send_frame(connection, payload):
    header = bytearray([0x81])
    if len(payload) < 126:
        header.append(len(payload))
    elif len(payload) <= 0xFFFF:
        header.append(126)
        header.extend(struct.pack("!H", len(payload)))
    else:
        header.append(127)
        header.extend(struct.pack("!Q", len(payload)))
    connection.sendall(header + payload)


def websocket(connection, request, path, options):
    if options.mode == "handshake-hang":
        time.sleep(60)
        return
    headers = {}
    for line in request.split("\r\n")[1:]:
        if ":" in line:
            name, value = line.split(":", 1)
            headers[name.lower()] = value.strip()
    key = headers["sec-websocket-key"]
    accept = base64.b64encode(
        hashlib.sha1((key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").encode()).digest()
    ).decode()
    connection.sendall(
        (
            "HTTP/1.1 101 Switching Protocols\r\n"
            "Upgrade: websocket\r\n"
            "Connection: Upgrade\r\n"
            f"Sec-WebSocket-Accept: {accept}\r\n\r\n"
        ).encode()
    )
    while True:
        opcode, payload = receive_frame(connection)
        if opcode == 8:
            return
        if opcode != 1:
            continue
        message = json.loads(payload)
        if options.mode == "command-hang":
            time.sleep(60)
            return
        if options.mode == "command-crash":
            os._exit(7)
        method = message.get("method")
        if options.log:
            with open(options.log, "a") as log:
                log.write(json.dumps(message) + "\n")
        result = {"workerPid": os.getpid(), "path": path}
        if method == "Browser.getVersion":
            result["protocolVersion"] = "0.0" if options.mode == "bad-version" else "1.3"
        elif method == "Target.createBrowserContext":
            result["browserContextId"] = "context"
        elif method == "Target.createTarget":
            result["targetId"] = "target"
        elif method == "Target.attachToTarget":
            result["sessionId"] = "session"
        elif method == "Runtime.evaluate":
            result["result"] = {"type": "string", "value": "Managed"}
        elif method == "Page.navigate":
            result["frameId"] = "main"
        send_frame(connection, json.dumps({"id": message.get("id"), "result": result}).encode())
        if method == "Page.navigate":
            for event, params in [
                ("Network.responseReceived", {"type": "Document", "requestId": "request", "response": {"url": message["params"]["url"], "status": 200}}),
                ("Page.loadEventFired", {}),
            ]:
                send_frame(connection, json.dumps({"method": event, "params": params, "sessionId": message["sessionId"]}).encode())



def handle(connection, address, options):
    del address
    try:
        data = bytearray()
        while b"\r\n\r\n" not in data:
            data.extend(connection.recv(4096))
        request = data.decode("ascii")
        path = request.split(" ", 2)[1]
        if "upgrade: websocket" in request.lower():
            websocket(connection, request, path, options)
            return
        if options.mode == "discovery-hang":
            time.sleep(60)
            return
        host = connection.getsockname()
        if path == "/json/version":
            value = {
                "Browser": "FakeCDP/1.0",
                "Protocol-Version": "1.3",
                "webSocketDebuggerUrl": f"ws://127.0.0.1:{host[1]}/devtools/browser/fake",
            }
        elif path in ("/json", "/json/list"):
            value = [
                {
                    "id": "fake-page",
                    "type": "page",
                    "title": "Fake",
                    "url": "about:blank",
                    "webSocketDebuggerUrl": f"ws://127.0.0.1:{host[1]}/devtools/page/fake-page",
                }
            ]
        else:
            value = {"error": "Not found"}
        body = json.dumps(value, separators=(",", ":")).encode()
        connection.sendall(
            f"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {len(body)}\r\nConnection: close\r\n\r\n".encode()
            + body
        )
        if options.mode == "discovery-keepalive":
            time.sleep(60)
    except (EOFError, OSError, ValueError):
        pass
    finally:
        connection.close()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--remote-debugging-port", type=int, required=True)
    parser.add_argument("--user-data-dir", required=True)
    parser.add_argument("--headless", action="store_true")
    parser.add_argument("--record")
    parser.add_argument("--log")
    parser.add_argument("--mode", default="normal")
    parser.add_argument("--label")
    options = parser.parse_args()
    if options.record:
        Path(options.record).write_text(json.dumps({"pid": os.getpid(), "profile": options.user_data_dir, "port": options.remote_debugging_port, "arguments": sys.argv[1:]}))
    if options.mode == "startup-crash":
        return
    if options.mode == "startup-hang":
        time.sleep(60)
        return
    listener = socket.create_server(("127.0.0.1", options.remote_debugging_port))
    while True:
        connection, address = listener.accept()
        threading.Thread(target=handle, args=(connection, address, options), daemon=True).start()


if __name__ == "__main__":
    main()

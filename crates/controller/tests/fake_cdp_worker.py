#!/usr/bin/env python3

import argparse
import base64
import hashlib
import json
import os
import socket
import struct
import threading
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


def websocket(connection, request, path):
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
        send_frame(
            connection,
            json.dumps(
                {
                    "id": message.get("id"),
                    "result": {"workerPid": os.getpid(), "path": path},
                },
                separators=(",", ":"),
            ).encode(),
        )


def handle(connection, address):
    del address
    try:
        data = bytearray()
        while b"\r\n\r\n" not in data:
            data.extend(connection.recv(4096))
        request = data.decode("ascii")
        path = request.split(" ", 2)[1]
        if "upgrade: websocket" in request.lower():
            websocket(connection, request, path)
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
    except (EOFError, OSError, ValueError):
        pass
    finally:
        connection.close()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--remote-debugging-port", type=int, required=True)
    options = parser.parse_args()
    listener = socket.create_server(("127.0.0.1", options.remote_debugging_port))
    while True:
        connection, address = listener.accept()
        threading.Thread(target=handle, args=(connection, address), daemon=True).start()


if __name__ == "__main__":
    main()

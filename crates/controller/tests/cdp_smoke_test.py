#!/usr/bin/env python3

import argparse
import asyncio
import base64
import json
import os
import struct
from urllib.parse import urlsplit


class RawCDPClient:
    def __init__(self, reader, writer):
        self.reader = reader
        self.writer = writer
        self.next_id = 1
        self.events = []

    @classmethod
    async def connect(cls, websocket_url):
        parsed = urlsplit(websocket_url)
        reader, writer = await asyncio.open_connection(parsed.hostname, parsed.port)
        key = base64.b64encode(os.urandom(16)).decode('ascii')
        request = (
            'GET {} HTTP/1.1\r\nHost: {}:{}\r\nUpgrade: websocket\r\n'
            'Connection: Upgrade\r\nSec-WebSocket-Version: 13\r\n'
            'Sec-WebSocket-Key: {}\r\n\r\n'
        ).format(parsed.path, parsed.hostname, parsed.port, key)
        writer.write(request.encode('ascii'))
        await writer.drain()
        response = await reader.readuntil(b'\r\n\r\n')
        if not response.startswith(b'HTTP/1.1 101'):
            raise RuntimeError('WebSocket upgrade failed: {!r}'.format(response))
        return cls(reader, writer)

    async def _send(self, value):
        payload = json.dumps(value, separators=(',', ':')).encode('utf-8')
        mask = os.urandom(4)
        header = bytearray([0x81])
        if len(payload) < 126:
            header.append(0x80 | len(payload))
        elif len(payload) < 65536:
            header.append(0x80 | 126)
            header.extend(struct.pack('!H', len(payload)))
        else:
            header.append(0x80 | 127)
            header.extend(struct.pack('!Q', len(payload)))
        header.extend(mask)
        header.extend(byte ^ mask[index % 4] for index, byte in enumerate(payload))
        self.writer.write(header)
        await self.writer.drain()

    async def _receive(self):
        header = await self.reader.readexactly(2)
        length = header[1] & 0x7f
        if length == 126:
            length = struct.unpack('!H', await self.reader.readexactly(2))[0]
        elif length == 127:
            length = struct.unpack('!Q', await self.reader.readexactly(8))[0]
        return json.loads((await self.reader.readexactly(length)).decode('utf-8'))

    async def command(self, method, params=None, session_id=None, timeout=10):
        request_id = self.next_id
        self.next_id += 1
        message = {'id': request_id, 'method': method, 'params': params or {}}
        if session_id:
            message['sessionId'] = session_id
        await self._send(message)
        while True:
            response = await asyncio.wait_for(self._receive(), timeout)
            if response.get('id') == request_id:
                if 'error' in response:
                    raise RuntimeError('{} failed: {}'.format(method, response['error']))
                return response['result']
            self.events.append(response)

    async def wait_for_event(self, method, timeout=10, session_id=None):
        for index, event in enumerate(self.events):
            if event.get('method') == method and (session_id is None or event.get('sessionId') == session_id):
                return self.events.pop(index)
        while True:
            event = await asyncio.wait_for(self._receive(), timeout)
            if event.get('method') == method and (session_id is None or event.get('sessionId') == session_id):
                return event
            self.events.append(event)

    async def close(self):
        mask = os.urandom(4)
        self.writer.write(b'\x88\x80' + mask)
        await self.writer.drain()
        self.writer.close()
        await self.writer.wait_closed()


async def get_json(base_url, path):
    parsed = urlsplit(base_url)
    reader, writer = await asyncio.open_connection(parsed.hostname, parsed.port)
    writer.write(('GET {} HTTP/1.1\r\nHost: {}:{}\r\nConnection: close\r\n\r\n'.format(path, parsed.hostname, parsed.port)).encode('ascii'))
    await writer.drain()
    response = await reader.read()
    writer.close()
    await writer.wait_closed()
    status, body = response.split(b'\r\n\r\n', 1)
    if b' 200 ' not in status.split(b'\r\n', 1)[0]:
        raise RuntimeError('HTTP discovery failed: {!r}'.format(status))
    return json.loads(body)


async def start_fixture_server(body):
    async def respond(reader, writer):
        try:
            try:
                request = await asyncio.wait_for(reader.readuntil(b'\r\n\r\n'), 1)
            except (asyncio.IncompleteReadError, asyncio.TimeoutError):
                return
            path = request.split(b' ', 2)[1]
            payload = ('<!doctype html><title>child frame</title><p>child</p>' if path == b'/child' else body).encode('utf-8')
            writer.write(b'HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: ' + str(len(payload)).encode('ascii') + b'\r\nConnection: close\r\n\r\n' + payload)
            await writer.drain()
        finally:
            writer.close()
            await writer.wait_closed()

    server = await asyncio.start_server(respond, '127.0.0.1', 0)
    port = server.sockets[0].getsockname()[1]
    return server, 'http://127.0.0.1:{}/fixture'.format(port)


async def run(base_url, probe=False):
    version = await get_json(base_url, '/json/version')
    if version.get('Protocol-Version') != '1.3':
        raise RuntimeError('unexpected protocol version: {}'.format(version))
    client = await RawCDPClient.connect(version['webSocketDebuggerUrl'])
    try:
        targets = (await client.command('Target.getTargets'))['targetInfos']
        if not targets:
            raise RuntimeError('controller has no initial page target')
        attached = await client.command('Target.attachToTarget', {'targetId': targets[0]['targetId'], 'flatten': True})
        session_id = attached['sessionId']
        await client.command('Page.enable', session_id=session_id)
        await client.command('Runtime.enable', session_id=session_id)
        await client.command('Network.enable', session_id=session_id)
        fixture_server, fixture_url = await start_fixture_server('<!doctype html><title>network fixture</title><iframe src="/child"></iframe>')
        try:
            await client.command('Page.navigate', {'url': fixture_url}, session_id)
            await client.wait_for_event('Page.domContentEventFired')
            await client.wait_for_event('Page.loadEventFired')
            frame_tree = await client.command('Page.getFrameTree', session_id=session_id)
            if len(frame_tree['frameTree'].get('childFrames', [])) != 1:
                raise RuntimeError('native frame tree did not include the child frame: {}'.format(frame_tree))
            request_event = await client.wait_for_event('Network.requestWillBeSent')
            response_event = await client.wait_for_event('Network.responseReceived')
            await client.wait_for_event('Network.loadingFinished')
            if request_event['params']['request']['url'] != fixture_url or response_event['params']['response']['status'] != 200:
                raise RuntimeError('unexpected Network event data')
        finally:
            fixture_server.close()
            await fixture_server.wait_closed()
        document = '''<!doctype html>
<style>
body { margin: 0; background: rgb(12,34,56); }
#button { position: absolute; left: 20px; top: 20px; width: 120px; height: 50px; }
#input { position: absolute; left: 20px; top: 90px; width: 160px; height: 30px; }
#spacer { height: 1400px; }
</style>
<button id=button>Brimp CDP</button><input id=input><div id=spacer></div>
<script>
window.inputEvents = [];
for (const type of ['mousedown', 'mouseup', 'click', 'keydown', 'keyup'])
    document.addEventListener(type, event => inputEvents.push({ type, target: event.target.id, trusted: event.isTrusted, key: event.key }));
</script>'''
        document_server, document_url = await start_fixture_server(document)
        try:
            await client.command('Page.navigate', {'url': document_url}, session_id)
            await client.wait_for_event('Page.domContentEventFired')
            await client.wait_for_event('Page.loadEventFired')
        finally:
            document_server.close()
            await document_server.wait_closed()
        evaluated = await client.command('Runtime.evaluate', {
            'expression': '({text:button.textContent,width:innerWidth,height:innerHeight,webdriver:navigator.webdriver})',
            'returnByValue': True,
        }, session_id)
        value = evaluated['result']['value']
        if value['text'] != 'Brimp CDP' or value['webdriver'] is not False:
            raise RuntimeError('unexpected evaluation result: {}'.format(value))
        screenshot = await client.command('Page.captureScreenshot', {'format': 'png'}, session_id)
        png = base64.b64decode(screenshot['data'])
        if not png.startswith(b'\x89PNG\r\n\x1a\n') or len(png) < 100:
            raise RuntimeError('invalid screenshot')
        if probe:
            observations = await client.command('Runtime.evaluate', {
                'expression': '''(() => {
                    const canvas = document.createElement('canvas');
                    const gl = canvas.getContext('webgl');
                    return { visibilityState: document.visibilityState, hasFocus: document.hasFocus(),
                        innerWidth, innerHeight, outerWidth, outerHeight, devicePixelRatio,
                        screen: { width: screen.width, height: screen.height, colorDepth: screen.colorDepth },
                        hover: matchMedia('(hover: hover)').matches,
                        webglRenderer: gl ? gl.getParameter(gl.RENDERER) : null };
                })()''',
                'returnByValue': True,
            }, session_id)
            window_handle = await client.command('Runtime.evaluate', {'expression': 'window'}, session_id)
            cadence = await client.command('Runtime.callFunctionOn', {
                'objectId': window_handle['result']['objectId'],
                'functionDeclaration': '''async function() {
                    const cadence = async () => {
                        const timestamps = [];
                        for (let i = 0; i < 6; ++i)
                            timestamps.push(await new Promise(resolve => requestAnimationFrame(resolve)));
                        return timestamps.slice(1).map((value, index) => value - timestamps[index]);
                    };
                    return Promise.race([cadence(), new Promise(resolve => setTimeout(() => resolve(null), 1000))]);
                }''',
                'returnByValue': True,
            }, session_id)
            await client.command('Runtime.releaseObject', {'objectId': window_handle['result']['objectId']}, session_id)
            probe_value = observations['result']['value']
            probe_value['animationFrameDeltas'] = cadence['result']['value']
            print('PROBE: ' + json.dumps(probe_value, sort_keys=True))
            return
        remote = await client.command('Runtime.evaluate', {
            'expression': 'document.querySelector("#button")',
        }, session_id)
        object_id = remote['result'].get('objectId')
        if not object_id:
            raise RuntimeError('object evaluation did not return a remote handle: {}'.format(remote))
        called = await client.command('Runtime.callFunctionOn', {
            'objectId': object_id,
            'functionDeclaration': 'function() { return { tag: this.tagName, text: this.textContent }; }',
            'returnByValue': True,
        }, session_id)
        if called['result'].get('value') != {'tag': 'BUTTON', 'text': 'Brimp CDP'}:
            raise RuntimeError('unexpected remote-handle call result: {}'.format(called))
        await client.command('Runtime.releaseObject', {'objectId': object_id}, session_id)

        for event_type in ('mouseMoved', 'mousePressed', 'mouseReleased'):
            await client.command('Input.dispatchMouseEvent', {
                'type': event_type, 'x': 80, 'y': 45, 'button': 'left', 'clickCount': 1,
            }, session_id)
        for event_type in ('mousePressed', 'mouseReleased'):
            await client.command('Input.dispatchMouseEvent', {
                'type': event_type, 'x': 80, 'y': 105, 'button': 'left', 'clickCount': 1,
            }, session_id)
        for event_type in ('keyDown', 'keyUp'):
            await client.command('Input.dispatchKeyEvent', {
                'type': event_type, 'key': 'a', 'text': 'a', 'windowsVirtualKeyCode': 65,
            }, session_id)
        input_result = await client.command('Runtime.evaluate', {
            'expression': '({value:input.value,active:document.activeElement.id,events:inputEvents})', 'returnByValue': True,
        }, session_id)
        input_value = input_result['result']['value']
        if input_value['value'] != 'a' or not any(event['type'] == 'click' and event['trusted'] for event in input_value['events']):
            raise RuntimeError('native input did not produce trusted click/type events: {}'.format(input_value))

        full_screenshot = await client.command('Page.captureScreenshot', {
            'format': 'png', 'captureBeyondViewport': True,
        }, session_id, timeout=60)
        full_png = base64.b64decode(full_screenshot['data'])
        width, height = struct.unpack('!II', full_png[16:24])
        if width < 640 or height <= 480:
            raise RuntimeError('full-page screenshot has unexpected dimensions: {}x{}'.format(width, height))
        unsupported_id = client.next_id
        client.next_id += 1
        await client._send({'id': unsupported_id, 'method': 'Debugger.enable', 'sessionId': session_id, 'params': {}})
        while True:
            response = await client._receive()
            if response.get('id') == unsupported_id:
                break
        if response.get('error', {}).get('code') != -32601:
            raise RuntimeError('unsupported method did not return -32601: {}'.format(response))

        storage_server, storage_url = await start_fixture_server('<!doctype html><title>context storage</title>')
        try:
            first_context = (await client.command('Target.createBrowserContext'))['browserContextId']
            first_target = (await client.command('Target.createTarget', {
                'url': storage_url, 'browserContextId': first_context,
            }))['targetId']
            second_target = (await client.command('Target.createTarget', {
                'url': storage_url, 'browserContextId': first_context,
            }))['targetId']
            first_session = (await client.command('Target.attachToTarget', {
                'targetId': first_target, 'flatten': True,
            }))['sessionId']
            second_session = (await client.command('Target.attachToTarget', {
                'targetId': second_target, 'flatten': True,
            }))['sessionId']
            await client.command('Page.enable', session_id=first_session)
            await client.command('Page.enable', session_id=second_session)
            await client.command('Page.navigate', {'url': storage_url}, first_session)
            await client.command('Page.navigate', {'url': storage_url}, second_session)
            await client.wait_for_event('Page.loadEventFired', session_id=first_session)
            await client.wait_for_event('Page.loadEventFired', session_id=second_session)
            await client.command('Runtime.evaluate', {
                'expression': 'localStorage.setItem("context-key", "shared")', 'returnByValue': True,
            }, first_session)
            shared = await client.command('Runtime.evaluate', {
                'expression': 'localStorage.getItem("context-key")', 'returnByValue': True,
            }, second_session)
            if shared['result'].get('value') != 'shared':
                raise RuntimeError('targets in one browser context did not share storage: {}'.format(shared))

            second_context = (await client.command('Target.createBrowserContext'))['browserContextId']
            isolated_target = (await client.command('Target.createTarget', {
                'url': storage_url, 'browserContextId': second_context,
            }))['targetId']
            isolated_session = (await client.command('Target.attachToTarget', {
                'targetId': isolated_target, 'flatten': True,
            }))['sessionId']
            await client.command('Page.enable', session_id=isolated_session)
            await client.command('Page.navigate', {'url': storage_url}, isolated_session)
            await client.wait_for_event('Page.loadEventFired', session_id=isolated_session)
            isolated = await client.command('Runtime.evaluate', {
                'expression': 'localStorage.getItem("context-key")', 'returnByValue': True,
            }, isolated_session)
            if isolated['result'].get('value') is not None:
                raise RuntimeError('ephemeral browser contexts leaked storage: {}'.format(isolated))

            await client.command('Target.disposeBrowserContext', {'browserContextId': first_context})
            contexts = (await client.command('Target.getBrowserContexts'))['browserContextIds']
            targets_after_dispose = (await client.command('Target.getTargets'))['targetInfos']
            if first_context in contexts or any(target['targetId'] in (first_target, second_target) for target in targets_after_dispose):
                raise RuntimeError('disposing a browser context did not remove its targets')
            await client.command('Target.disposeBrowserContext', {'browserContextId': second_context})
        finally:
            storage_server.close()
            await storage_server.wait_closed()
        print('PASS: targets, navigation, lifecycle/network events, frame tree, evaluation/handles, trusted native input, screenshots, ephemeral context sharing/isolation, and explicit errors')
    finally:
        await client.close()


def main(argv=None):
    parser = argparse.ArgumentParser(description='Raw CDP acceptance test for brimp')
    parser.add_argument('base_url', help='controller URL, for example http://127.0.0.1:9222')
    parser.add_argument('--probe', action='store_true', help='print rendering and lifecycle observations')
    options = parser.parse_args(argv)
    asyncio.run(run(options.base_url.rstrip('/'), options.probe))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())

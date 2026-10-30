import test from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { createSession } from '../../../bindings/node/index.js';

// Frames are loaded like a navigation: fetched outside the page's script path so
// cross-origin sources work, parsed into their own document, and their scripts run
// with that document bound. postMessage works in both directions with the frame as
// the source.
const childPage = `<!doctype html><html><head><title>child</title></head><body>
<div id="marker">child doc</div>
<script>window.frameValue = 'inline ran'; parent.postMessage({ hello: 'child inline' }, '*');</script>
<script src="/child-extra.js"></script>
</body></html>`;

async function fixtures() {
  const server = createServer((request, response) => {
    const send = (type, body) => {
      response.writeHead(200, { 'content-type': type });
      response.end(body);
    };
    if (request.url === '/child') return send('text/html', childPage);
    if (request.url === '/child-extra.js')
      return send('text/javascript', `parent.postMessage({ hello: 'child external' }, '*');`);
    const target = request.url === '/cross' ? `http://127.0.0.1:${port + 1}/child` : '/child';
    send('text/html', `<!doctype html><html><body><iframe id="f" src="${target}"></iframe></body></html>`);
  });
  const port = await new Promise((resolve) => {
    server.listen(0, '127.0.0.1', () => resolve(server.address().port));
  });
  const second = createServer((request, response) => {
    if (request.url === '/child-extra.js') {
      response.writeHead(200, { 'content-type': 'text/javascript' });
      response.end(`parent.postMessage({ hello: 'child external' }, '*');`);
      return;
    }
    response.writeHead(200, { 'content-type': 'text/html' });
    response.end(childPage);
  });
  const secondPort = port + 1;
  await new Promise((resolve) => second.listen(secondPort, '127.0.0.1', resolve));
  return { port, close: () => (server.close(), second.close()) };
}

test('frames load their document, run their scripts and post messages with their origin', async () => {
  const { port, close } = await fixtures();
  const session = await createSession();
  try {
    for (const path of ['/', '/cross']) {
      const page = await session.get(`http://127.0.0.1:${port}${path}`);
      assert.deepEqual(
        await page.evaluate(`(async () => {
          const frame = document.getElementById('f');
          const messages = [];
          addEventListener('message', event => messages.push([event.data.hello, event.source === frame.contentWindow, event.origin]));
          const win = frame.contentWindow;
          let loaded = false;
          frame.addEventListener('load', () => { loaded = true; });
          for (let attempt = 0; attempt < 40 && messages.length < 2; attempt++)
            await new Promise(resolve => setTimeout(resolve, 50));
          return [frame.contentDocument.title, frame.contentDocument.getElementById('marker').textContent,
            win.frameValue, win.frameValue === 'inline ran' ? messages.length : -1, loaded,
            messages.every(([, fromFrame]) => fromFrame)];
        })()`),
        ['child', 'child doc', 'inline ran', 2, true, true],
      );
    }
  } finally {
    await session.close();
    close();
  }
});

test('a frame whose source cannot be fetched keeps an empty document and still loads', async () => {
  const { port, close } = await fixtures();
  const session = await createSession();
  try {
    const page = await session.get(`http://127.0.0.1:${port}/`);
    assert.deepEqual(
      await page.evaluate(`(async () => {
        const frame = document.getElementById('f');
        let loaded = false;
        frame.addEventListener('load', () => { loaded = true; });
        frame.src = 'http://127.0.0.1:1/missing';
        frame.contentWindow;
        await new Promise(resolve => setTimeout(resolve, 800));
        return [frame.contentDocument.title, frame.contentDocument.body.childElementCount, loaded];
      })()`),
      ['', 0, true],
    );
  } finally {
    await session.close();
    close();
  }
});

test('frame load and posted messages dispatch on the visible frame window', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    assert.deepEqual(await page.evaluate(`(async () => {
      const frame = document.createElement('iframe');
      document.body.append(frame);
      const child = frame.contentWindow;
      let loaded = false;
      child.onload = function(event) {
        loaded = this === child && event.currentTarget === child;
      };
      child.location.href = 'data:text/html,<body>new document</body>';
      const received = new Promise(resolve => {
        child.addEventListener('message', function(event) {
          resolve([event.data, this === child, event.currentTarget === child]);
        }, {once: true});
      });
      child.postMessage('round trip', '*');
      return [loaded, await received];
    })()`), [true, ['round trip', true, true]]);
  } finally {
    await session.close();
  }
});

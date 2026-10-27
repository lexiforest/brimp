import {createServer} from 'node:http';
import {readFile} from 'node:fs/promises';
import {join, extname} from 'node:path';

const types = {'.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.css': 'text/css', '.json': 'application/json', '.svg': 'image/svg+xml', '.png': 'image/png', '.ico': 'image/x-icon'};
export async function startServer(dist, workload, manifest) {
  const requests = [];
  const timers = new Set();
  const server = createServer(async (request, response) => {
    try {
      const url = new URL(request.url, 'http://localhost');
      const chunks = [];
      let length = 0;
      for await (const chunk of request) {
        length += chunk.length;
        if (length > 1024 * 1024) { response.writeHead(413).end(); return; }
        chunks.push(chunk);
      }
      const body = Buffer.concat(chunks).toString();
      if (requests.length < 256) requests.push({method: request.method, path: url.pathname, body: body.slice(0, 4096)});
      response.setHeader('cache-control', 'no-store');
      if (url.pathname === '/api/echo') {
        response.setHeader('content-type', 'application/json');
        response.end(JSON.stringify({method: request.method, body}));
      } else if (url.pathname === '/api/bytes') {
        response.setHeader('content-type', 'application/octet-stream');
        response.end(Buffer.from([0, 127, 255]));
      } else if (url.pathname === '/api/slow') {
        const timer = setTimeout(() => { timers.delete(timer); response.end('late'); }, 10000);
        timers.add(timer);
        response.on('close', () => { clearTimeout(timer); timers.delete(timer); });
      } else if (url.pathname === '/contact/1/edit' && request.method === 'GET') {
        response.setHeader('content-type', 'text/html');
        response.end('<form hx-put="/contact/1" hx-target="this" hx-swap="outerHTML"><div><label>First Name</label><input name="firstName" value="Joe"></div><div><label>Last Name</label><input name="lastName" value="Blow"></div><div><label>Email Address</label><input name="email" value="joe@blow.com"></div><button class="btn">Submit</button></form>');
      } else if (url.pathname === '/contact/1' && request.method === 'PUT') {
        const fields = new URLSearchParams(body);
        const escape = value => String(value ?? '').replace(/[&<>"']/g, char => ({'&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;'}[char]));
        response.setHeader('content-type', 'text/html');
        response.end(`<div id="saved-contact"><span>${escape(fields.get('firstName'))}</span> <span>${escape(fields.get('lastName'))}</span> <span>${escape(fields.get('email'))}</span></div>`);
      } else {
        const path = `${workload}/${decodeURIComponent(url.pathname.slice(1) || 'index.html')}`;
        if (!Object.hasOwn(manifest.files, path)) { response.writeHead(404).end('Not found'); return; }
        response.setHeader('content-type', types[extname(path)] ?? 'application/octet-stream');
        response.end(await readFile(join(dist, path)));
      }
    } catch (error) {
      if (!response.headersSent) response.writeHead(500);
      response.end(String(error));
    }
  });
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', resolve);
  });
  return {
    origin: `http://127.0.0.1:${server.address().port}`,
    requests,
    async close() {
      for (const timer of timers) clearTimeout(timer);
      server.closeAllConnections();
      await new Promise(resolve => server.close(resolve));
    },
  };
}

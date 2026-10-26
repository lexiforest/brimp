from concurrent.futures import ThreadPoolExecutor
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import json
import threading
import time
import pytest
from brimp import Session, Closed, StaleHandle, Timeout

FIXTURES = Path(__file__).resolve().parents[2] / 'fixtures'

@pytest.fixture(scope='module')
def wasm_server():
    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path == '/slow':
                self.send_response(200); self.send_header('Content-Type','application/wasm'); self.end_headers()
                time.sleep(0.5)
                return
            path = FIXTURES / ('wasm-import.wasm' if self.path == '/wasm-import.wasm' else 'xxhash-wasm/' + self.path.lstrip('/'))
            if not path.is_file(): self.send_error(404); return
            data = path.read_bytes()
            self.send_response(200)
            self.send_header('Content-Type', 'application/wasm' if path.suffix == '.wasm' else 'text/javascript')
            self.send_header('Content-Length', str(len(data))); self.end_headers()
            try:
                for offset in range(0, len(data), 127): self.wfile.write(data[offset:offset+127])
            except (BrokenPipeError, ConnectionResetError): pass
        def log_message(self, *_): pass
    server = ThreadingHTTPServer(('127.0.0.1',0), Handler)
    thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
    yield f'http://127.0.0.1:{server.server_port}'
    server.shutdown();server.server_close();thread.join()


def test_real_browser_wasm_through_fetch_and_streaming(wasm_server):
    with Session() as session:
        page=session.new_page()
        result=page.evaluate(f'({(FIXTURES / "wasm-workload.js").read_text()})({json.dumps(wasm_server)})')
        assert result == {'hash32':'fb0077f9','hash64':'26c7827d889f6da3','incremental':0x26c7827d889f6da3,
            'streamed32':0xfb0077f9,'streamed64':0x26c7827d889f6da3,'module':True,
            'imported':42,'linkError':'LinkError','memory':[1,131072],'global':2,'table':'function','again':'function'}


def test_wasm_errors_and_body_contracts():
    with Session() as session:
        page=session.new_page()
        result=page.evaluate('''(async()=>{
          const bytes=new Uint8Array([0,97,115,109,1,0,0,0]);
          const error=async fn=>{try{await fn();return 'unexpected success'}catch(e){return e.name}};
          const used=new Response(bytes,{headers:{'Content-Type':'application/wasm'}});await used.arrayBuffer();
          const locked=new Response(bytes,{headers:{'Content-Type':'application/wasm'}});locked.body.getReader();
          return await Promise.all([
            error(()=>WebAssembly.compile(new Uint8Array([1]))),
            error(()=>WebAssembly.compileStreaming(new Response(bytes))),
            error(()=>WebAssembly.compileStreaming(new Response(bytes,{headers:{'Content-Type':'application/wasm; charset=utf-8'}}))),
            error(()=>WebAssembly.compileStreaming(new Response(bytes,{status:404,headers:{'Content-Type':'application/wasm'}}))),
            error(()=>WebAssembly.compileStreaming(used)),error(()=>WebAssembly.compileStreaming(locked)),
            error(()=>WebAssembly.compileStreaming(Promise.reject(new RangeError('source')))),
            error(()=>WebAssembly.compileStreaming({})),
            WebAssembly.instantiate(bytes).then(({instance})=>instance instanceof WebAssembly.Instance),
          ]);
        })()''')
        assert result == ['CompileError'] + ['TypeError']*5 + ['RangeError','TypeError',True]


def test_wasm_pending_close_and_timeout(wasm_server):
    with Session(timeout=0.15) as session:
        page=session.new_page()
        with pytest.raises(Timeout): page.evaluate(f'WebAssembly.compileStreaming(fetch({json.dumps(wasm_server+"/slow")}))')
        assert page.evaluate('42') == 42
    with Session() as session, ThreadPoolExecutor(1) as pool:
        page=session.new_page()
        future=pool.submit(page.evaluate,f'WebAssembly.compileStreaming(fetch({json.dumps(wasm_server+"/slow")}))')
        time.sleep(.03);page.close()
        with pytest.raises(Closed): future.result(2)

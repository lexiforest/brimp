"""CLI integration tests against a local HTTP server; build brimp first."""
import base64
import json
import os
from pathlib import Path
import signal
import subprocess
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

import pytest

ROOT = Path(__file__).resolve().parents[3]
CLI = Path(os.environ.get('BRIMP_CLI', ROOT / 'target/debug/brimp'))


def run(*args, input=None, code=0):
    result = subprocess.run([str(CLI), *map(str, args)], input=input,
                            capture_output=True, timeout=12)
    assert result.returncode == code, (args, result.returncode, result.stdout, result.stderr)
    return result


@pytest.fixture(scope='module')
def site():
    requests = []
    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def do_POST(self):
            self.do_GET()

        def do_GET(self):
            body = self.rfile.read(int(self.headers.get('Content-Length', 0)))
            requests.append((self.path, self.command, dict(self.headers), body))
            headers = {}
            status, mime = 200, 'text/html'
            if self.path == '/redirect':
                status, headers, data = 302, {'Location': '/echo'}, b''
            elif self.path == '/auth' and self.headers.get('Authorization') != 'Basic ' + base64.b64encode(b'foo:bar').decode():
                status, headers, data = 401, {'WWW-Authenticate': 'Basic realm="local"'}, b'auth required'
            elif self.path == '/error':
                status, data = 404, b'not found'
            elif self.path == '/binary':
                mime, data = 'application/octet-stream', b'\x00\xff\x80hello'
            elif self.path == '/echo':
                mime, data = 'application/json', json.dumps({'method': self.command, 'body': body.decode(), 'headers': dict(self.headers)}).encode()
            elif self.path == '/slow.js':
                time.sleep(0.6)
                mime, data = 'text/javascript', b'globalThis.asyncDone = true'
            elif self.path == '/defer.js':
                mime, data = 'text/javascript', b'globalThis.deferDone = true'
            elif self.path == '/milestones':
                data = b'<script>globalThis.asyncDone=false;globalThis.deferDone=false;globalThis.loaded=false;addEventListener("load",()=>loaded=true)</script><script async src="/slow.js"></script><script defer src="/defer.js"></script>'
            elif self.path == '/stalled':
                time.sleep(2)
                data = b'<h1>late</h1>'
            elif self.path == '/loop':
                data = b'<script>while(true){}</script>'
            else:
                data = b'''<!doctype html><title>CLI fixture</title><main><h1>Useful article</h1>
                <p>This is a useful article with enough text to extract. It contains important details about the browser runtime and its command line interface.</p>
                <input id="name"><input id="file" type="file"><button id="button" onclick="this.textContent='clicked'">click</button>
                <script>document.getElementById('button').addEventListener('click',function(){this.textContent='clicked'});document.body.dataset.ran='yes'; console.log('fixture console');setTimeout(()=>{const p=document.createElement('p');p.id='later';p.textContent='ready';document.body.append(p)},20)</script></main>'''
            self.send_response(status)
            self.send_header('Content-Type', mime)
            self.send_header('Content-Length', str(len(data)))
            for name, value in headers.items():
                self.send_header(name, value)
            self.end_headers()
            try:
                self.wfile.write(data)
            except (BrokenPipeError, ConnectionResetError):
                pass

    server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    yield f'http://127.0.0.1:{server.server_port}', requests
    server.shutdown()
    server.server_close()
    thread.join()


def write(tmp_path, name, content):
    path = tmp_path / name
    path.write_text(content if isinstance(content, str) else json.dumps(content))
    return path


def test_help_doctor_catalog():
    assert b'fetch' in run('--help').stdout
    for command in ['fetch', 'run', 'doctor', 'plugins']:
        run('help', command)
    assert json.loads(run('doctor').stdout)['crypto'] == 'ok'
    catalog = json.loads(run('plugins', 'list').stdout)
    assert 'storage' not in catalog and 'host-storage' in catalog
    assert json.loads(run('plugins', 'show', 'canvas').stdout)['limitations']
    run('plugins', 'show', 'unknown', code=2)
    run('crawl', 'https://unused.test', code=2)
    run('serve', code=2)


def test_run_stdin_promise_console_and_blank_init(tmp_path):
    init = write(tmp_path, 'init.js', 'globalThis.initialized=42')
    result = run('run', '-', '--init-script', init, input=b'console.log("hello"); Promise.resolve(initialized)')
    assert json.loads(result.stdout) == 42
    assert b'hello' in result.stderr
    assert json.loads(run('run', '-', input=b'void 0').stdout) is None
    run('run', '-', input=b'throw Error("broken")', code=13)


def test_run_local_html_with_base_url(tmp_path):
    html = write(tmp_path, 'saved.html', '''<!doctype html><title>Saved</title>
        <base href="../assets/"><a id="next" href="next?x=1">next</a>
        <script>globalThis.pageScriptRan = true</script>''')
    value = json.loads(run('run', '-', '--html', html, '--html-url', 'https://example.test/dir/page.html',
                           '--eval', '''({title: document.title, url: location.href,
                               base: document.baseURI, href: document.getElementById('next').href,
                               pageScript: typeof globalThis.pageScriptRan,
                               ready: document.readyState})''', input=b'void 0').stdout)
    assert value == dict(title='Saved', url='https://example.test/dir/page.html',
                         base='https://example.test/assets/',
                         href='https://example.test/assets/next?x=1',
                         pageScript='undefined', ready='complete')
    assert json.loads(run('run', '-', '--html', html, '--eval', 'location.href',
                          input=b'void 0').stdout) == 'about:blank'
    run('run', '-', '--html', html, '--html-url', 'not a URL', input=b'void 0', code=2)
    run('run', '-', '--html', html, '--url', 'https://example.test/', input=b'void 0', code=2)
    run('fetch', 'https://example.test/', '--html', html, code=2)
    run('run', '-', '--html-url', 'https://example.test/', input=b'void 0', code=2)


def test_trace_report_success_failure_loss_and_output_paths(tmp_path):
    report = tmp_path / 'dependencies.json'
    trace = tmp_path / 'trace.jsonl'
    result = run('run', '-', '--trace-report', report, '--trace', trace,
                 '--trace-include', 'Document.prototype.querySelector', '--trace-capacity', '2',
                 input=b'for(let i=0;i<3;i++)document.querySelector("body"); 42')
    assert json.loads(result.stdout) == 42
    summary = json.loads(report.read_text())
    assert summary['retained'] == 2 and summary['dropped'] == 1
    assert summary['coverage']['enabled'] is True
    assert summary['coverage']['include'] == ['Document.prototype.querySelector']
    assert summary['dependencies'][0]['count'] == 2
    assert len(trace.read_text().splitlines()) == 3
    original = report.read_bytes()
    run('run', '-', '--trace-report', report, input=b'42', code=2)
    assert report.read_bytes() == original
    output = write(tmp_path, 'result.json', 'keep existing result')
    run('run', '-', '--trace-report', report, '--overwrite',
        '--output', output,
        '--trace-include', 'Document.prototype.querySelector',
        input=b'document.querySelector("[")', code=13)
    assert output.read_text() == 'keep existing result'
    summary = json.loads(report.read_text())
    assert summary['dependencies'][0]['throws'] == 1
    assert summary['retained'] == 1
    run('run', '-', '--trace-report', '-', input=b'42', code=2)


def test_recursive_trace_mode(tmp_path):
    report = tmp_path / 'recursive.json'
    result = run('run', '-', '--trace-mode', 'recursive', '--trace-report', report,
                 input=b'window.fixture={nested:{}};void window.fixture.nested.missing;void navigator.missing;window.fixture.nested.value=42;window.fixture.nested.value')
    assert json.loads(result.stdout) == 42
    summary = json.loads(report.read_text())
    assert summary['coverage']['mode'] == 'recursive'
    assert any(entry['layer'] == 'recursive' and entry['kind'] == 'get'
               and entry['path'] == 'window.fixture.nested.missing' for entry in summary['dependencies'])
    reads = {entry['path']: entry for entry in summary['unresolved']['reads']}
    assert reads['window.fixture.nested.missing']['present'] is False
    assert reads['window.fixture.nested.missing']['owner'] == 'unknown'
    assert reads['navigator.missing']['present'] is False
    assert reads['navigator.missing']['owner'] == 'profile'
    run('run', '-', '--trace-mode', 'recursive', input=b'42', code=2)
    run('run', '-', '--trace-mode', 'unknown', input=b'42', code=2)
    run('run', '-', '--trace-report', report, '--output', report, '--overwrite', input=b'42', code=2)
    run('run', '-', '--trace-report', report, '--trace', report, '--overwrite', input=b'42', code=2)
    run('run', '-', '--trace-include', 'Document', input=b'42', code=2)


@pytest.mark.parametrize('mode', ['api', 'recursive'])
def test_harness_example(tmp_path, mode):
    example = ROOT / 'examples/harness'
    report = tmp_path / 'dependencies.json'
    result = run('run', example / 'page.js', '--clock', 'logical', '--epoch-ms', '1000',
                 '--routes', example / 'routes.json', '--actions', example / 'actions.json',
                 '--eval', 'compute("Ada")', '--trace-mode', mode, '--trace-report', report)
    assert json.loads(result.stdout) == dict(text='Hello Ada', calls=1, clicks=1, now=1005)
    summary = json.loads(report.read_text())
    assert summary['coverage']['mode'] == mode
    assert any(entry['path'] == 'Document.prototype.querySelector' for entry in summary['dependencies'])


def test_fetch_outputs_and_waits(site, tmp_path):
    url, _ = site
    result = run('fetch', url, '--wait-selector', '#later')
    assert b'data-ran="yes"' in result.stdout and b'id="later"' in result.stdout
    assert b'fixture console' in result.stderr
    value = json.loads(run('fetch', url, '--wait', 'networkidle', '--network-idle', '20ms', '--eval', '({title:document.title})').stdout)
    assert value == {'title': 'CLI fixture'}
    output = tmp_path / 'article.html'
    run('fetch', url, '--output', output)
    original = output.read_bytes()
    run('fetch', url, '--output', output, code=2)
    assert output.read_bytes() == original
    run('fetch', url, '--output', output, '--overwrite')
    run('fetch', url, '--output', '-', code=2)
    run('fetch', url, '--format', 'png', code=2)
    result = run('fetch', url, '--format', 'markdown')
    assert b'useful article' in result.stdout.lower()
    value = json.loads(run('fetch', url, '--format', 'json', '--content', 'main', '--remove-images', '--language', 'en').stdout)
    assert 'content' in value and value['title']


def test_binary_requests_redirects_and_metadata(site, tmp_path):
    url, _ = site
    assert run('fetch', url + '/binary', '--format', 'raw').stdout == b'\x00\xff\x80hello'
    meta = tmp_path / 'response.json'
    value = json.loads(run('fetch', url + '/echo', '--format', 'raw', '--method', 'POST', '--data', 'body', '--header', 'X-Cli: yes', '--cookie', 'test=value', '--response-info', meta).stdout)
    assert value['method'] == 'POST' and value['body'] == 'body'
    headers = {name.lower(): value for name, value in value['headers'].items()}
    assert headers['x-cli'] == 'yes' and 'test=value' in headers['cookie']
    assert json.loads(meta.read_text())['status_code'] == 200
    run('fetch', url + '/redirect', '--no-redirects', '--format', 'raw')
    redirected = tmp_path / 'redirect.json'
    run('fetch', url + '/redirect', '--format', 'raw', '--response-info', redirected)
    assert len(json.loads(redirected.read_text())['history']) == 1
    run('fetch', url + '/redirect', '--max-redirects', '0', code=10)
    assert b'not found' == run('fetch', url + '/error', '--format', 'raw').stdout
    run('fetch', url + '/error', '--fail-on-http-error', code=11)
    run('fetch', url + '/auth', '--auth', 'foo:bar')


def test_streaming(site, tmp_path):
    url, _ = site
    target, metadata = tmp_path / 'body.bin', tmp_path / 'meta.json'
    run('fetch', url + '/binary', '--format', 'raw', '--stream', '--output', target, '--response-info', metadata)
    assert target.read_bytes() == b'\x00\xff\x80hello'
    assert json.loads(metadata.read_text())['status'] == 200
    target.unlink()
    run('fetch', url + '/binary', '--format', 'raw', '--stream', '--output', target, '--max-response-bytes', '3', code=10)
    assert not target.exists()
    run('fetch', url, '--stream', code=2)
    run('fetch', url, '--stream', '--format', 'raw', '--wait-selector', 'h1', code=2)


def test_plugins_and_logical_time(tmp_path, site):
    url, _ = site
    plugin = write(tmp_path, 'custom.js', '({global,options})=>{global.answer=options.answer; return ()=>{}}')
    result = run('run', '-', '--plugin', plugin, '--plugin-options', 'custom={"answer":42}', input=b'answer')
    assert json.loads(result.stdout) == 42
    program = write(tmp_path, 'timers.js', 'globalThis.result=[]; setTimeout(()=>result.push(Date.now()),50); null')
    result = run('run', program, '--clock', 'logical', '--epoch-ms', '1000', '--advance-time', '50ms', '--eval', 'result')
    assert json.loads(result.stdout) == [1050]
    result = run('run', '-', '--url', url, input=b'localStorage.setItem("key","value"); localStorage.getItem("key")')
    assert json.loads(result.stdout) == 'value'
    url, _ = site
    result = run('run', '-', '--url', url, '--plugin', 'host-storage', input=b'(async()=>{await hostStorage.set("x",42);return await hostStorage.get("x")})()')
    assert json.loads(result.stdout) == 42
    run('run', '-', '--plugin-options', 'missing={}', input=b'1', code=2)
    run('run', '-', '--plugin', plugin, '--plugin', plugin, input=b'1', code=2)


def test_actions_routes_and_logs(site, tmp_path):
    url, _ = site
    upload = write(tmp_path, 'upload.txt', 'file contents')
    actions = write(tmp_path, 'actions.json', [
        {'action': 'fill', 'selector': '#name', 'text': 'Ada'},
        {'action': 'click', 'selector': '#button'},
        {'action': 'upload', 'selector': '#file', 'files': [upload.name]},
        {'action': 'wait-selector', 'selector': '#later'},
    ])
    events, trace = tmp_path / 'events.jsonl', tmp_path / 'trace.jsonl'
    result = run('fetch', url, '--actions', actions, '--events', events, '--trace', trace, '--trace-include', 'Document.prototype', '--eval', '[document.querySelector("#name").value,document.querySelector("#button").textContent,document.querySelector("#file").files[0].name]')
    assert json.loads(result.stdout) == ['Ada', 'clicked', 'upload.txt']
    assert any(json.loads(line)['event'] == 'request' for line in events.read_text().splitlines())
    assert trace.read_text().strip()
    fixture = write(tmp_path, 'fixture.html', '<title>supplied</title>')
    routes = write(tmp_path, 'routes.json', [{'pattern': '*', 'decision': {'action': 'fulfill', 'file': fixture.name, 'headers': [['Content-Type', 'text/html']]}}])
    result = run('fetch', url, '--routes', routes, '--eval', 'document.title')
    assert json.loads(result.stdout) == 'supplied'
    run('fetch', url, '--output', events, '--format', 'html', '--events', events, '--overwrite', code=2)


def test_navigation_milestones(site):
    url, _ = site
    expression = '[document.readyState,asyncDone,deferDone,loaded]'
    dom = json.loads(run('fetch', url + '/milestones', '--wait', 'domcontentloaded', '--eval', expression).stdout)
    assert dom == ['interactive', False, True, False]
    loaded = json.loads(run('fetch', url + '/milestones', '--wait', 'load', '--eval', expression).stdout)
    assert loaded == ['complete', True, True, True]
    later = json.loads(run('fetch', url + '/milestones', '--wait', 'domcontentloaded', '--eval', 'new Promise(resolve=>{addEventListener("load",()=>resolve(asyncDone))})').stdout)
    assert later is True


@pytest.mark.parametrize('source', ['while(true){}', 'new Promise(()=>{})'])
def test_timeout_during_javascript(source):
    started = time.monotonic()
    run('run', '-', '--timeout', '200ms', input=source.encode(), code=14)
    assert time.monotonic() - started < 3


def test_timeout_navigation_and_atomic_output(site, tmp_path):
    url, _ = site
    destination = write(tmp_path, 'keep.html', 'keep')
    run('fetch', url + '/stalled', '--timeout', '200ms', '--output', destination, '--overwrite', code=14)
    assert destination.read_text() == 'keep'
    run('fetch', url + '/loop', '--timeout', '200ms', code=14)


@pytest.mark.skipif(os.name != 'posix', reason='POSIX signal integration')
@pytest.mark.parametrize('source', ['console.log("ready");while(true){}', 'console.log("ready");new Promise(()=>{})'])
def test_sigint_during_javascript(source):
    proc = subprocess.Popen([str(CLI), 'run', '-'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        proc.stdin.write(source.encode())
        proc.stdin.close()
        assert b'ready' in proc.stderr.readline()
        proc.send_signal(signal.SIGINT)
        proc.wait(timeout=3)
        assert proc.returncode == 15, proc.stderr.read()
    finally:
        if proc.poll() is None:
            proc.kill()
            proc.wait()


@pytest.mark.parametrize('args', [
    ['--timeout', 'nanms'], ['--timeout', '0s'], ['--header', 'not-a-header'],
    ['--header', 'User-Agent: invalid'], ['--max-response-bytes', '0'],
    ['--epoch-ms', '1000'], ['--profile', 'missing.json'],
    ['--eval', '1', '--format', 'html'], ['--timeout', '1s', '--timeout', '2s'],
])
def test_validation_before_network(args):
    run('fetch', 'https://unused.invalid', *args, code=2)


def test_profile_composes_with_plugins_trace_and_clock(site, tmp_path):
    url, _ = site
    profile = {'values': {'identity': {}}}
    user_agent = 'Mozilla/5.0 (Macintosh) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/27.0 Safari/605.1.15'
    profile['values']['identity']['user_agent'] = user_agent
    profile['values']['identity']['app_version'] = user_agent[len('Mozilla/'):]
    path = write(tmp_path, 'profile.json', profile)
    trace = tmp_path / 'trace.jsonl'
    value = json.loads(run('fetch', url, '--profile', path, '--plugin', 'environment',
                          '--trace', trace, '--clock', 'logical', '--epoch-ms', '1234',
                          '--eval', '[navigator.userAgent,typeof DOMPoint,Date.now()]').stdout)
    assert value == [user_agent, 'function', 1234]
    assert trace.exists()
    run('fetch', url, '--profile', path, '--user-agent', 'conflict', code=2)


def test_shared_deadline_and_plugin_installation(tmp_path):
    first = write(tmp_path, 'first.js', 'new Promise(resolve=>setTimeout(resolve,150))')
    program = write(tmp_path, 'program.js', 'new Promise(resolve=>setTimeout(resolve,150))')
    run('run', program, '--script', first, '--timeout', '250ms', code=14)
    plugin = write(tmp_path, 'pending.js', '()=>new Promise(()=>{})')
    run('run', '-', '--plugin', plugin, '--timeout', '200ms', input=b'1', code=14)
    plugin.write_text('()=>{while(true){}}')
    run('run', '-', '--plugin', plugin, '--timeout', '200ms', input=b'1', code=14)


def test_response_routes_and_precedence(site, tmp_path):
    url, _ = site
    rules = write(tmp_path, 'routes.json', [
        {'pattern': '*', 'stage': 'response', 'decision': {'action': 'fulfill', 'body': '<title>first</title>'}},
        {'pattern': '*', 'stage': 'response', 'decision': {'action': 'continue', 'body': '<title>last</title>', 'status': 201}},
    ])
    result = run('fetch', url, '--routes', rules, '--eval', 'document.title')
    assert json.loads(result.stdout) == 'last'
    rules.write_text(json.dumps([{'pattern': '*', 'decision': {'action': 'block'}}]))
    run('fetch', url, '--routes', rules, code=10)


def test_host_storage_atomic_rollback_and_plugin_cleanup(site):
    url, _ = site
    program = b'''(async()=>{
      await hostStorage.set('a',1);
      try { await hostStorage.transaction([{type:'set',key:'a',value:2},{type:'unknown'}]); } catch(e) {}
      return await hostStorage.get('a');
    })()'''
    assert json.loads(run('run', '-', '--url', url, '--plugin', 'host-storage', input=program).stdout) == 1


@pytest.mark.skipif(os.name != 'posix', reason='POSIX signal integration')
def test_sigterm_during_navigation(site):
    url, requests = site
    before = sum(path == '/stalled' for path, *_ in requests)
    process = subprocess.Popen([str(CLI), 'fetch', url + '/stalled'], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        until = time.monotonic() + 4
        while sum(path == '/stalled' for path, *_ in requests) == before:
            assert time.monotonic() < until
            time.sleep(.01)
        process.send_signal(signal.SIGTERM)
        process.wait(timeout=3)
        assert process.returncode == 15, process.stderr.read()
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()


def test_subsystem_plugins_and_options():
    result = run('run', '-', '--plugin', 'environment', '--plugin', 'permissions',
                 '--plugin', 'devices', '--plugin', 'media', '--plugin', 'workers',
                 '--plugin', 'trusted-types',
                 '--plugin-options', 'environment={"connection_rtt_ms":7}',
                 '--plugin-options', 'permissions={"notification_permission":"denied"}',
                 '--plugin-options', 'devices={"bluetooth_available":true}',
                 '--plugin-options', 'media={"media_devices_enabled":false}',
                 input=b'''Promise.all([navigator.connection.rtt, Notification.requestPermission(),
                    navigator.bluetooth.getAvailability(), typeof MediaEncryptedEvent,
                    typeof navigator.mediaDevices, new Worker('ignored').terminate() === undefined,
                    trustedTypes.createPolicy() === undefined])''')
    assert json.loads(result.stdout) == [7, 'denied', True, 'function', 'undefined', True, True]


def test_default_browser_plugins_and_core_only_selection():
    source = b'[typeof Worker,typeof AudioContext,typeof RTCPeerConnection,typeof hostStorage]'
    assert json.loads(run('run', '-', input=source).stdout) == ['function'] * 3 + ['undefined']
    assert json.loads(run('run', '-', '--no-plugins', input=source).stdout) == ['undefined'] * 4
    assert json.loads(run('run', '-', '--plugin', 'workers', input=source).stdout) == ['function'] + ['undefined'] * 3

import json
import pytest
from brimp import Session, JavaScriptError
from brimp import plugins


def test_recursive_diagnostics_nested_reads_writes_and_calls(tmp_path):
    trace = tmp_path / 'recursive.jsonl'
    with Session(time_mode='logical', diagnostics={'mode': 'recursive', 'path': str(trace)}) as session:
        page = session.new_page()
        assert page.evaluate('''window.fixture = {nested: {value: 1}};
            window.fixture.nested.value = 2; void window.fixture.nested.missing;
            new Promise(resolve => setTimeout(() => resolve(document.body.tagName), 1))''') == 'BODY'
        summary = page.summarize_diagnostics()
        assert summary['coverage']['mode'] == 'recursive'
        assert any(entry['layer'] == 'recursive' and entry['kind'] == 'set'
                   and entry['path'] == 'window.fixture.nested.value' for entry in summary['dependencies'])
        assert any(record['path'] == 'window.fixture.nested.missing' and record['outcome'] == 'undefined'
                   for record in page.peek_diagnostics()['records'])
    assert any(json.loads(line)['details'].get('layer') == 'recursive'
               for line in trace.read_text().splitlines() if json.loads(line)['details'])


def test_retained_function_diagnostic_inspection():
    with Session(time_mode='logical', time_origin=1000,
                 diagnostics={'include': ['Document.prototype.querySelector']}) as session:
        page = session.new_page()
        with page.evaluate_handle('(() => { let calls = 0; return async n => { await new Promise(r => setTimeout(r, 5)); document.querySelector("body"); return [n + ++calls, Date.now()]; }; })()') as fn:
            page.take_diagnostics()
            for expected in [[41, 1005], [42, 1010]]:
                with fn.call('(fn, n) => fn(n)', 40) as result:
                    assert result.json_value() == expected
            before = page.peek_diagnostics()
            summary = page.summarize_diagnostics()
            assert summary['coverage']['enabled'] is True
            assert summary['write_error'] is None
            assert summary['dependencies'] == [dict(layer='api', path='Document.prototype.querySelector', kind='call', target=None, property=None, count=2, throws=0)]
            assert page.take_diagnostics() == before
            assert page.summarize_diagnostics()['retained'] == 0


def test_clock_diagnostics_and_plugins(tmp_path):
    path = tmp_path / 'trace.jsonl'
    with Session(time_mode='logical', time_origin=1000,
                 diagnostics={'path': str(path), 'include': ['Document.prototype', 'Reflect'], 'capacity': 8},
                 plugins=[plugins.audio()]) as session:
        page = session.new_page()
        page.evaluate("globalThis.seen=[];setTimeout(()=>seen.push([Date.now(),performance.now()]),20);null")
        assert page.advance_time(20)['now'] == 20
        assert page.evaluate('seen') == [[1020, 20]]
        assert page.evaluate("new Promise(r=>setTimeout(()=>r(Date.now()),10000))") == 11020
        assert page.evaluate("CSS.s(2).to('ms').value") == 2000
        assert page.evaluate("new DOMMatrix().translate(3,4).transformPoint(new DOMPoint(2,1)).toJSON()") == dict(x=5,y=5,z=0,w=1)
        page.evaluate("document.querySelector('main'); Reflect.ownKeys({a:1}); null")
        records = page.take_diagnostics()
        assert any(r['path'] == 'Document.prototype.querySelector' for r in records['records'])
        assert all('page_id' in r and 'realm_id' in r for r in records['records'])
        assert page.take_diagnostics()['records'] == []
        assert session.new_page().evaluate('performance.now()') == 0
    assert len([json.loads(line) for line in path.read_text().splitlines()]) >= 2


def test_graphics_audio_host_extensions_and_core_only_selection():
    with Session(plugins=[]) as session:
        page = session.new_page()
        assert page.evaluate("[typeof DOMPoint,typeof CSSUnitValue,typeof scheduler,typeof AudioContext,typeof WebGLRenderingContext]") == ['function', 'function', 'object', 'undefined', 'undefined']
    with Session(time_mode='logical', plugins=[
        plugins.graphics(rgba=(9,8,7,6), handlers={'graphics.submit': lambda context, value: value['width']}),
        plugins.audio(handlers={'audio.process': lambda context, value: value['samples']})]) as session:
        page = session.new_page()
        assert page.evaluate("document.createElement('canvas').getContext('webgl').submitAsync([])") == 300
        assert page.evaluate("new AudioContext().processAsync(new Uint8Array([1,2])).then(v=>Array.from(v))") == [1,2]
        with pytest.raises(JavaScriptError, match='decoding unavailable'):
            page.evaluate('new AudioContext().decodeAudioData(new ArrayBuffer(1))')

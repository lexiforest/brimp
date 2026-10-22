import threading
from concurrent.futures import ThreadPoolExecutor

import pytest
from brimp import Plugin, Session, Reentry, RuntimeError

SOURCE = '''async ({global, host}) => {
    const state = await host.call('init', {binary: new Uint8Array([3, 5, 8])});
    global.initial = Array.from(state.binary);
    global.messages = [];
    host.subscribe('message', v => global.messages.push(v));
    global.send = name => host.call(name);
    return async () => { await host.call('dispose'); };
}'''


def test_async_install_binary_notifications_exceptions_and_disposal():
    disposed = []
    def init(context, value):
        assert value == {"binary": bytes([3, 5, 8])}
        return value
    def notify(context):
        for n in range(3): context.emit('message', n)
        return None
    def fail(context): raise ValueError('handler detail')
    plugin = Plugin('demo', SOURCE, {'init': init, 'notify': notify, 'fail': fail,
                                    'dispose': lambda ctx: disposed.append(ctx.realm_id)})
    with Session(plugins=[plugin]) as session:
        with session.new_page() as page:
            assert page.evaluate('initial') == [3, 5, 8]
            assert page.evaluate('send("notify").then(() => messages)') == [0, 1, 2]
            assert 'OperationError' in page.evaluate('send("fail").catch(e => String(e))')
            assert 'OperationError' in page.evaluate('send("absent").catch(e => String(e))')
    assert len(disposed) == 1


def test_reentry_rejected_and_other_pages_remain_usable():
    refs = {}
    def callback(context):
        for action in (lambda: refs['page'].evaluate('1'), refs['page'].close,
                       refs['session'].close, refs['session'].new_page,
                       refs['handle'].json_value):
            with pytest.raises(Reentry): action()
        return refs['other'].evaluate('6 * 7')
    plugin = Plugin('reentry', '({global, host}) => { global.check = () => host.call("check"); }', {'check': callback})
    with Session(plugins=[plugin]) as session:
        refs.update(session=session, page=session.new_page(), other=session.new_page())
        refs['handle'] = refs['page'].evaluate_handle('({a: 1})')
        assert refs['page'].evaluate('check()') == 42
        assert refs['page'].evaluate('1 + 1') == 2
        refs['handle'].dispose()


def test_close_cancels_pending_callback_and_discards_late_result():
    entered, release = threading.Event(), threading.Event()
    contexts = []
    def callback(context):
        contexts.append(context)
        entered.set()
        release.wait(5)
        return b'late'
    plugin = Plugin('pending', '({global, host}) => { global.wait = () => host.call("wait"); }', {'wait': callback})
    with Session(plugins=[plugin]) as session, ThreadPoolExecutor() as pool:
        page = session.new_page()
        result = pool.submit(page.evaluate, 'wait()')
        assert entered.wait(2)
        page.close()
        assert contexts[0].cancelled
        release.set()
        with pytest.raises(RuntimeError): result.result(2)


def test_installation_failure_and_asyncio_handler_rejection():
    with Session(plugins=[Plugin('broken', '() => { throw new Error("install detail"); }')]) as session:
        with pytest.raises(RuntimeError, match='broken.*install detail'):
            session.new_page()
    async def unsupported(context): return None
    with Session(plugins=[Plugin('asyncio', 'async ({host}) => { await host.call("async"); }', {'async': unsupported})]) as session:
        with pytest.raises(RuntimeError, match='asyncio is not supported'):
            session.new_page()


def test_navigation_cancels_host_call_without_contaminating_new_realm():
    entered, release = threading.Event(), threading.Event()
    contexts = []
    def callback(context):
        contexts.append(context)
        entered.set()
        release.wait(5)
        return "old"
    source = '({global,host}) => { global.wait = () => host.call("wait").then(v => global.late = v); }'
    with Session(plugins=[Plugin('navigation', source, {'wait': callback})]) as session:
        page = session.new_page()
        with ThreadPoolExecutor(1) as pool:
            pending = pool.submit(page.evaluate, 'wait()')
            try:
                assert entered.wait(2)
                page.get('data:text/html,<title>new realm</title>')
                assert contexts[0].cancelled
            finally:
                release.set()
            with pytest.raises(RuntimeError, match='cancelled'):
                pending.result(2)
        assert page.evaluate('[document.title, typeof late]') == ['new realm', 'undefined']


def test_bundled_plugins_input_and_extraction():
    from brimp import plugins
    with Session(plugins=[plugins.storage(), plugins.canvas(), plugins.services(), plugins.extraction()]) as session:
        page = session.new_page()
        page.evaluate("document.body.innerHTML='<article><h1>Native article</h1><p>This article is read from a DOM without a layout engine.</p></article><input id=edit><input id=check type=checkbox>'; null")
        page.fill('#edit', 'ab')
        page.type_text('#edit', 'c')
        page.press('#edit', 'Backspace')
        assert page.evaluate("document.querySelector('#edit').value") == 'ab'
        with page.query_selector('#check') as checkbox:
            checkbox.click()
            assert checkbox._value('node => node.checked') is True
        assert 'DOM' in page.extract(content_selector='article')['contentMarkdown']
        assert page.evaluate("localStorage.x='yes';localStorage.getItem('x')") == 'yes'
        assert page.evaluate("Array.from(document.createElement('canvas').getContext('2d').getImageData(0,0,1,1).data)") == [0, 0, 0, 0]

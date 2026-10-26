import math
from concurrent.futures import ThreadPoolExecutor
import pytest
from brimp import Session, Plugin, BigInt, undefined, JavaScriptError, StaleHandle, Closed, Reentry, Timeout


def test_rich_values_and_reference_calls():
    with Session() as session:
        page = session.new_page()
        values = page.evaluate('[undefined, null, NaN, Infinity, -Infinity, -0, 123n, new Uint8Array([0,255])]')
        assert values[:2] == [undefined, None]
        assert math.isnan(values[2]) and values[3:5] == [math.inf, -math.inf]
        assert math.copysign(1, values[5]) == -1
        assert isinstance(values[6], BigInt) and values[6] == 123
        assert values[7] == b'\x00\xff'
        with page.evaluate_handle('(function (data) { return [this.tag, data, typeof data.big]; })') as fn, page.evaluate_handle('({tag: "receiver"})') as receiver:
            with fn.invoke({'big': 2**80, 'small': BigInt(1), 'bytes': b'\x00\xff', 'missing': undefined}, this=receiver) as result:
                data = result.value()
                assert data[0] == 'receiver' and data[2] == 'bigint'
                assert data[1] == {'big': 2**80, 'small': 1, 'bytes': b'\x00\xff', 'missing': undefined}
            receiver.set_property('self', receiver)
            with receiver.call('(value, nested) => value === nested.ref && value.self === value', {'ref': receiver}) as result:
                assert result.value() is True
            assert receiver.delete_property('self')
            assert receiver.property_names() == ['tag']
        with pytest.raises(StaleHandle): receiver.value()
        with pytest.raises(JavaScriptError, match='reference'): page.evaluate('new Map()')
        cycle = []; cycle.append(cycle)
        with page.evaluate_handle('x => x') as fn:
            with pytest.raises(TypeError, match='Cyclic'): fn.invoke(cycle)
        other = session.new_page()
        with other.evaluate_handle('({})') as foreign, page.evaluate_handle('x => x') as fn:
            with pytest.raises(StaleHandle): fn.invoke(foreign)


def test_plugin_references_functions_binary_and_callback_cycle():
    held = []
    def inspect(context, obj, fn):
        held.append(obj)
        obj.set_property('bytes', b'\x00\xff')
        with fn.invoke(41) as result:
            assert result.value() == 42
        fn.dispose()
        return {'same': obj, 'increment': context.function('increment'), 'large': BigInt(2**70)}
    def cycle(context, fn):
        try:
            with pytest.raises(Reentry): fn.invoke()
        finally: fn.dispose()
        return 'rejected'
    plugin = Plugin('bridge', '''({global,host}) => {
        global.inspect = (obj, fn) => host.call('inspect', host.ref(obj), host.ref(fn));
        global.cycle = () => host.call('cycle', host.ref(() => host.call('increment', 1)));
    }''', {'inspect': inspect, 'increment': lambda context, n: n + 1, 'cycle': cycle})
    with Session(plugins=[plugin], timeout=2) as session:
        page = session.new_page()
        assert page.evaluate('''(async () => {
            const obj = {}; const result = await inspect(obj, n => n + 1);
            return [result.same === obj, obj.bytes, await result.increment(41), result.large];
        })()''') == [True, b'\x00\xff', 42, 2**70]
        assert page.evaluate('cycle()') == 'rejected'
        assert held[0].value() == {'bytes': b'\x00\xff'}
        page.close()
        with pytest.raises(Closed): held[0].value()


def test_handle_deadlines_close_and_concurrency():
    with Session(timeout=0.2) as session:
        page = session.new_page()
        with page.evaluate_handle('({count:0})') as obj, ThreadPoolExecutor(4) as pool:
            def increment(_):
                with obj.call('obj => ++obj.count') as result: return result.value()
            assert sorted(pool.map(increment, range(12))) == list(range(1,13))
        with page.evaluate_handle('() => new Promise(() => {})') as fn:
            with pytest.raises(Timeout): fn.invoke()
        assert page.evaluate('42') == 42
        with page.evaluate_handle('() => { while (true) {} }') as fn:
            with pytest.raises((Timeout, JavaScriptError)): fn.invoke()
        assert page.evaluate('42') == 42

from pathlib import Path
import pytest
from brimp import Session
from brimp.plugins import Plugin


def test_crypto_is_core():
    with Session() as session:
        page = session.new_page()
        assert (
            page.evaluate(
                "crypto.subtle.digest('SHA-256', new TextEncoder().encode('abc')).then(b => Array.from(new Uint8Array(b), n => n.toString(16).padStart(2, '0')).join(''))"
            )
            == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        )


def test_file_plugin_options_session_scope_and_disposal(tmp_path):
    source = tmp_path / "plugin.js"
    source.write_text(
        '({global,host,options}) => { global.answer=options.answer; global.next=()=>host.call("next"); }'
    )
    disposed = []

    def setup():
        counter = [0]

        def increment(context):
            assert context.origin is None
            assert context.session_id > 0
            counter[0] += 1
            return counter[0]

        return {"handlers": {"next": increment}, "dispose": lambda: disposed.append(counter[0])}

    plugin = Plugin.from_file(source, name="example", options={"answer": 42}, setup=setup)
    with Session(plugins=[plugin]) as first, Session(plugins=[plugin]) as second:
        a, b = first.new_page(), second.new_page()
        assert a.evaluate("answer") == 42
        assert a.evaluate("next()") == 1
        assert a.evaluate("next()") == 2
        assert b.evaluate("next()") == 1
    assert sorted(disposed) == [1, 2]


def test_plugin_setup_failure_disposes_preceding_scope():
    disposed = []

    def fail():
        raise ValueError("setup failed")

    with pytest.raises(ValueError, match="setup failed"):
        Session(
            plugins=[
                Plugin("one", "() => {}", setup=lambda: {"dispose": lambda: disposed.append(True)}),
                Plugin("two", "() => {}", setup=fail),
            ]
        )
    assert disposed == [True]


def test_file_plugin_paths_and_host_tracebacks_are_private(tmp_path):
    import threading

    source = tmp_path / "private-plugin.js"
    source.write_text(
        '({global,host}) => { global.fail=()=>host.call("fail"); global.localFailure=()=>{throw new Error("public detail")}; }'
    )

    def fail(context):
        raise ValueError(f"private host detail: {source}")

    plugin = Plugin.from_file(source, name="private", handlers={"fail": fail})
    observed, ready = [], threading.Event()

    def record(event):
        observed.append(event)
        ready.set()

    with Session(plugins=[plugin]) as session:
        page = session.new_page()
        page.on("error", record)
        result = page.evaluate("fail().catch(e=>({name:e.name,message:e.message,stack:e.stack}))")
        assert result["name"] == "OperationError"
        assert result["message"] == "The operation failed."
        assert str(source) not in str(result)
        assert ready.wait(2)
        assert "private host detail" in observed[0]["message"]
        assert "test_core_web.py" in observed[0]["message"]
        assert str(source) not in page.evaluate("try {localFailure()} catch(e) {e.stack}")
        assert "public detail" not in page.evaluate("localFailure.toString()")

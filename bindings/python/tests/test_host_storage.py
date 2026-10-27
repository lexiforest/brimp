import importlib.util
from pathlib import Path
import pytest
from brimp import Session, plugins, JavaScriptError


def test_catalog_and_host_storage():
    catalog = plugins.catalog()
    assert not {"storage", "geometry", "css-typed-om", "scheduling"}.intersection(catalog)
    catalog["canvas"]["options"]["rgba"]["default"][0] = 255
    assert plugins.catalog()["canvas"]["options"]["rgba"]["default"][0] == 0
    plugin = plugins.host_storage()
    with Session(plugins=[plugin]) as session:
        page = session.new_page()
        page.route("https://*.test/**", lambda r: r.fulfill(headers={"content-type": "text/html"}, body="<p>storage</p>"))
        with pytest.raises(JavaScriptError): page.evaluate('hostStorage.set("x",1)')
        page.get("https://a.test/")
        page.evaluate('hostStorage.set("x",{n:42})')
        page.get("https://a.test/reload")
        assert page.evaluate('hostStorage.get("x")') == {"n": 42}
        with pytest.raises(JavaScriptError):
            page.evaluate('hostStorage.transaction([{type:"set",key:"x",value:0},{type:"invalid"}])')
        assert page.evaluate('hostStorage.get("x")') == {"n": 42}
        page.get("https://b.test/")
        assert page.evaluate('hostStorage.get("x")') is None


def test_sqlite_backend_atomicity_isolation_and_reopen(tmp_path):
    path = Path(__file__).resolve().parents[3] / "examples/plugins/sqlite_storage.py"
    spec = importlib.util.spec_from_file_location("sqlite_storage", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    first, second = module.SQLiteStorage(tmp_path / "store.db"), module.SQLiteStorage(tmp_path / "store.db")
    try:
        first.transaction("https://a.test", [{"type": "set", "key": "x", "value": 42}])
        with pytest.raises(TypeError):
            first.transaction("https://a.test", [{"type": "set", "key": "x", "value": 0}, {"type": "set", "key": "bad", "value": b"not JSON"}])
        assert second.transaction("https://a.test", [{"type": "get", "key": "x"}]) == [42]
        assert second.transaction("https://b.test", [{"type": "get", "key": "x"}]) == [None]
    finally:
        first.close()
        second.close()
    reopened = module.SQLiteStorage(tmp_path / "store.db")
    try: assert reopened.transaction("https://a.test", [{"type": "get", "key": "x"}]) == [42]
    finally: reopened.close()


def test_core_storage_path_option_survives_session_reopen(tmp_path):
    for first in (True, False):
        with Session(storage_path=tmp_path / "storage.json") as session:
            page = session.new_page()
            page.route("https://storage.test/**", lambda r: r.fulfill(body="<body></body>", headers={"content-type": "text/html"}))
            page.get("https://storage.test/")
            if first:
                page.evaluate("localStorage.value='saved';null")
            assert page.evaluate("localStorage.value") == "saved"
            assert page.evaluate("navigator.storage.persisted()") is True

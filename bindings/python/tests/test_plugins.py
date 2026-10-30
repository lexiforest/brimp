import threading
from concurrent.futures import ThreadPoolExecutor

import pytest
from brimp import Plugin, Session, Reentry, RuntimeError

SOURCE = """async ({global, host}) => {
    const state = await host.call('init', {binary: new Uint8Array([3, 5, 8])});
    global.initial = Array.from(state.binary);
    global.messages = [];
    host.subscribe('message', v => global.messages.push(v));
    global.send = name => host.call(name);
    return async () => { await host.call('dispose'); };
}"""


def test_rtc_installer_selection_and_local_channels():
    from brimp import plugins

    with Session(plugins=[]) as session, session.new_page() as page:
        assert page.evaluate("typeof RTCPeerConnection") == "undefined"
    with Session(plugins=[plugins.rtc()]) as session, session.new_page() as page:
        assert page.evaluate("""(() => {
            const p = new RTCPeerConnection(), c = p.createDataChannel('python');
            return [c.label, c.readyState, p.sctp, c.send('ignored') === undefined];
        })()""") == ["python", "connecting", None, True]


def test_async_install_binary_notifications_exceptions_and_disposal():
    disposed = []

    def init(context, value):
        assert value == {"binary": bytes([3, 5, 8])}
        return value

    def notify(context):
        for n in range(3):
            context.emit("message", n)
        return None

    def fail(context):
        raise ValueError("handler detail")

    plugin = Plugin(
        "demo",
        SOURCE,
        {
            "init": init,
            "notify": notify,
            "fail": fail,
            "dispose": lambda ctx: disposed.append(ctx.realm_id),
        },
    )
    with Session(plugins=[plugin]) as session:
        with session.new_page() as page:
            assert page.evaluate("initial") == [3, 5, 8]
            assert page.evaluate('send("notify").then(() => messages)') == [0, 1, 2]
            assert "OperationError" in page.evaluate('send("fail").catch(e => String(e))')
            assert "OperationError" in page.evaluate('send("absent").catch(e => String(e))')
    assert len(disposed) == 1


def test_reentry_rejected_and_other_pages_remain_usable():
    refs = {}

    def callback(context):
        for action in (
            lambda: refs["page"].evaluate("1"),
            refs["page"].close,
            refs["session"].close,
            refs["session"].new_page,
        ):
            with pytest.raises(Reentry):
                action()
        assert refs["handle"].json_value() == {"a": 1}
        return refs["other"].evaluate("6 * 7")

    plugin = Plugin(
        "reentry",
        '({global, host}) => { global.check = () => host.call("check"); }',
        {"check": callback},
    )
    with Session(plugins=[plugin]) as session:
        refs.update(session=session, page=session.new_page(), other=session.new_page())
        refs["handle"] = refs["page"].evaluate_handle("({a: 1})")
        assert refs["page"].evaluate("check()") == 42
        assert refs["page"].evaluate("1 + 1") == 2
        refs["handle"].dispose()


def test_close_cancels_pending_callback_and_discards_late_result():
    entered, release = threading.Event(), threading.Event()
    contexts = []

    def callback(context):
        contexts.append(context)
        entered.set()
        release.wait(5)
        return b"late"

    plugin = Plugin(
        "pending",
        '({global, host}) => { global.wait = () => host.call("wait"); }',
        {"wait": callback},
    )
    with Session(plugins=[plugin]) as session, ThreadPoolExecutor() as pool:
        page = session.new_page()
        result = pool.submit(page.evaluate, "wait()")
        assert entered.wait(2)
        page.close()
        assert contexts[0].cancelled
        release.set()
        with pytest.raises(RuntimeError):
            result.result(2)


def test_installation_failure_and_asyncio_handler_rejection():
    with Session(
        plugins=[Plugin("broken", '() => { throw new Error("install detail"); }')]
    ) as session:
        with pytest.raises(RuntimeError, match="broken.*install detail"):
            session.new_page()

    async def unsupported(context):
        return None

    with Session(
        plugins=[
            Plugin(
                "asyncio", 'async ({host}) => { await host.call("async"); }', {"async": unsupported}
            )
        ]
    ) as session:
        with pytest.raises(RuntimeError, match="asyncio is not supported"):
            session.new_page()


def test_navigation_cancels_host_call_without_contaminating_new_realm():
    entered, release = threading.Event(), threading.Event()
    contexts = []

    def callback(context):
        contexts.append(context)
        entered.set()
        release.wait(5)
        return "old"

    source = (
        '({global,host}) => { global.wait = () => host.call("wait").then(v => global.late = v); }'
    )
    with Session(plugins=[Plugin("navigation", source, {"wait": callback})]) as session:
        page = session.new_page()
        with ThreadPoolExecutor(1) as pool:
            pending = pool.submit(page.evaluate, "wait()")
            try:
                assert entered.wait(2)
                page.get("data:text/html,<title>new realm</title>")
                assert contexts[0].cancelled
            finally:
                release.set()
            with pytest.raises(RuntimeError, match="cancelled"):
                pending.result(2)
        assert page.evaluate("[document.title, typeof late]") == ["new realm", "undefined"]


def test_bundled_plugins_input_and_extraction():
    from brimp import plugins

    with Session(
        plugins=[
            plugins.canvas(),
            plugins.environment(),
            plugins.permissions(),
            plugins.devices(),
            plugins.media(),
            plugins.workers(),
            plugins.trusted_types(),
            plugins.extraction(),
        ]
    ) as session:
        page = session.new_page()
        page.evaluate(
            "document.body.innerHTML='<article><h1>Native article</h1><p>This article is read from a DOM without a layout engine.</p></article><input id=edit><input id=check type=checkbox>'; null"
        )
        page.fill("#edit", "ab")
        page.type_text("#edit", "c")
        page.press("#edit", "Backspace")
        assert page.evaluate("document.querySelector('#edit').value") == "ab"
        with page.query_selector("#check") as checkbox:
            checkbox.click()
            assert checkbox._value("node => node.checked") is True
        assert "DOM" in page.extract(content_selector="article")["contentMarkdown"]
        assert page.evaluate(
            "Array.from(document.createElement('canvas').getContext('2d').getImageData(0,0,1,1).data)"
        ) == [0, 0, 0, 0]


def test_environment_navigator_override_options_are_preserved():
    from brimp import plugins

    names = {
        "hardware_concurrency",
        "max_touch_points",
        "vendor",
        "product_sub",
        "pdf_viewer_enabled",
        "device_memory_gb",
        "expose_global_privacy_control",
        "global_privacy_control",
    }
    assert names <= plugins.catalog()["environment"]["options"].keys()
    with Session(
        plugins=[
            plugins.environment(
                hardware_concurrency=2,
                max_touch_points=3,
                vendor="Configured",
                product_sub="custom",
                pdf_viewer_enabled=False,
                device_memory_gb=16,
                expose_global_privacy_control=True,
                global_privacy_control=True,
            )
        ]
    ) as session:
        page = session.new_page()
        assert page.evaluate(
            "[navigator.hardwareConcurrency,navigator.maxTouchPoints,navigator.vendor,navigator.productSub,navigator.pdfViewerEnabled,navigator.globalPrivacyControl,navigator.deviceMemory]"
        ) == [2, 3, "Configured", "custom", False, True, 16]
        assert page.evaluate(
            "[typeof SQLTransaction,Object.hasOwn(window,'openDatabase'),typeof openDatabase]"
        ) == ["function", True, "undefined"]


@pytest.mark.parametrize(
    "name,expression,options,expected",
    [
        ("environment", "navigator.connection.rtt", {"connection_rtt_ms": 7}, 7),
        (
            "permissions",
            "Notification.requestPermission()",
            {"notification_permission": "denied"},
            "denied",
        ),
        ("devices", "navigator.bluetooth.getAvailability()", {"bluetooth_available": True}, True),
        (
            "media",
            "[typeof MediaStream, typeof MediaEncryptedEvent, typeof navigator.mediaDevices]",
            {"media_devices_enabled": False},
            ["function", "function", "undefined"],
        ),
        ("workers", "new Worker('ignored').terminate() === undefined", {}, True),
        (
            "trusted_types",
            "trustedTypes.createPolicy('ci', {createHTML: value => value}).createHTML('safe')",
            {},
            "safe",
        ),
    ],
)
def test_subsystem_installers(name, expression, options, expected):
    from brimp import plugins

    with Session(
        user_agent="Mozilla/5.0 Chrome/130.0.0.0 Safari/537.36",
        plugins=[getattr(plugins, name)(**options)],
    ) as session:
        with session.new_page() as page:
            assert page.evaluate(expression) == expected


def test_default_browser_plugins_and_explicit_empty_selection():
    from brimp import plugins

    assert {p.name for p in plugins.browser()} == {
        name for name, info in plugins.catalog().items() if info["defaultEnabled"]
    }
    for options, expected in [
        ({}, ["function", "function", "function"]),
        ({"plugins": []}, ["undefined"] * 3),
    ]:
        with Session(**options) as session, session.new_page() as page:
            assert (
                page.evaluate("[typeof Worker,typeof AudioContext,typeof RTCPeerConnection]")
                == expected
            )

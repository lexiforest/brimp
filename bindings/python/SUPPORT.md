# Python support matrix

| API | Tested behavior |
| --- | --- |
| `launch(persona_json=None)` | Creates the shared in-process automation browser and validates an optional persona. |
| `Browser.new_page()` | Creates an owner-thread Brimp page. |
| `Page.goto(url, timeout=30.0)` | Navigates asynchronously; asyncio cancellation reaches the core token. |
| `Page.evaluate(source)` | Returns JSON-compatible Python values and rejects unsupported JavaScript values. |
| `Page.title()` / `Page.text_content()` | Returns canonical document output. |
| `Page.screenshot(full_page=False)` | Returns PNG `bytes` without text conversion. |
| `Page.close()` / `Browser.close()` | Closes children in order and is idempotent. |

Failures use `BrimpError.code`: `invalid_input`, `transport`, `http_status`,
`navigation`, `javascript`, `timeout`, `cancelled`, `unsupported`, `closed`,
`screenshot`, or `internal`. Locators, browser modes, and raw protocol dispatch
are not exposed. Task cancellation raises `BrimpCancelledError`, which remains
an `asyncio.CancelledError` and has code `cancelled`.

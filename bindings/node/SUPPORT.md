# Node support matrix

| API | Tested behavior |
| --- | --- |
| `launch({personaJson})` | Creates the shared in-process automation browser and validates an optional persona. |
| `Browser.newPage()` | Creates an owner-thread Brimp page. |
| `Page.goto(url, {timeoutMs, signal})` | Navigates asynchronously; `AbortSignal` reaches the core token. |
| `Page.evaluate(source)` | Returns JSON-compatible JavaScript values and rejects unsupported values. |
| `Page.title()` / `Page.textContent()` | Returns canonical document output. |
| `Page.screenshot({fullPage})` | Returns a PNG `Buffer` without text conversion. |
| `Page.close()` / `Browser.close()` | Closes children in order and is idempotent. |

Failures use `BrimpError.code`: `invalid_input`, `transport`, `http_status`,
`navigation`, `javascript`, `timeout`, `cancelled`, `unsupported`, `closed`,
`screenshot`, or `internal`. Locators, browser modes, and raw protocol dispatch
are not exposed.

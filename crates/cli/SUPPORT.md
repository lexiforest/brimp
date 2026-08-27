# CLI support matrix

| Command | Tested behavior |
| --- | --- |
| `doctor` | Validates JavaScriptCore, libcurl-impersonate, and the selected curl profile. |
| `cdp` | Serves the supported Chrome DevTools Protocol subset over HTTP and WebSocket. |
| `eval URL --js SOURCE` | Navigates to `Complete` and prints one structured JSON value on stdout. |
| `screenshot URL --output PATH` | Writes viewport or full-page PNG bytes and refuses overwrite unless requested. |

All commands support stable categorized exit codes. Navigation commands accept
timeouts and translate Ctrl-C into the shared cancellation token. Interactive
browsing, browser modes beyond the bounded CDP server, batch, fetch, extract, and
render are unsupported and are not accepted as flags or commands.

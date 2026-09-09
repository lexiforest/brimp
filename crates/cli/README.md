# brimp-cli

Command-line frontend for separate browser worker processes. The CLI does not link JavaScriptCore, curl-impersonate, or `brimp-runtime`.

Tagged releases publish relocatable archives for manylinux 2.28 x86-64/ARM64,
macOS 11+ ARM64, and Windows x86-64. Each archive includes the required native
runtimes, their licenses, and a separate SHA-256 checksum.

Browser commands currently require macOS. Select a worker explicitly with
`--worker-path PATH` or `BRIMP_WORKER_PATH`; no worker is discovered automatically.
Build both executables with `cargo build -p brimp-cli -p brimp-lite-worker`.
The CLI alone builds without native SDKs using `cargo build -p brimp-cli`.

```sh
export BRIMP_WORKER_PATH="$PWD/target/debug/lite-worker"
brimp serve --port=9222
brimp doctor
brimp get URL
brimp get URL --output article.md
brimp get URL --format json --output article.json
brimp get URL --eval 'document.title'
brimp get URL --output page.png [--full-page]
brimp crawl URL --output-dir ./reference --depth 2 --workers 2
```

Results are written to stdout or `--output`, and diagnostics stay on stderr.
Rendered HTML, raw response bytes, Defuddle Markdown/JSON, evaluation JSON, and
PNG are supported. Existing files are refused unless
`--overwrite` is explicit. Exit categories are stable: 2 input, 10 transport,
11 HTTP status, 12 navigation, 13 JavaScript, 14 timeout, 15 cancellation, 16
unsupported result, 17 closed object, and 18 extraction/screenshot/runtime failure.

`crawl` writes Markdown by default and records one terminal JSON object per URL
in `manifest.jsonl`. It obeys robots policy and remains on the final start
origin unless explicitly expanded with `--allow-origin`.

See `SUPPORT.md` for the exhaustive tested command surface.

`get` launches one worker and one page. `crawl` launches one worker and uses
`--workers` concurrent pages sharing a browser context. Scheduling, robots
policy, and output files belong to the CLI; all network and page operations
run in the child. Startup counts toward the command deadline. Cancellation,
timeout, and completion terminate and reap the child; requests are not replayed
after a crash.

A WebKit worker executable or `.app` bundle can be selected with the same
option. Its current implementation supports rendered HTML, extraction,
evaluation, and screenshots. It does not support raw response retrieval,
robots.txt retrieval, custom headers/cookies, personas, proxy/trust-root
configuration, or lite subsystem/storage options; these produce explicit
errors. WebKit crawling currently requires `--ignore-robots`.

`serve` also supports managed CDP workers via `--worker-protocol=cdp`.
See [controller documentation](../controller/README.md).

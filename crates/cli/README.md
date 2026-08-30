# brimp-cli

Thin command-line interface over `web-runtime`'s canonical automation page.

Tagged releases publish relocatable archives for manylinux 2.28 x86-64/ARM64,
macOS 11+ ARM64, and Windows x86-64. Each archive includes the required native
runtimes, their licenses, and a separate SHA-256 checksum.

```text
brimp doctor
brimp cdp [--bind HOST:PORT] [--allow-non-loopback]
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

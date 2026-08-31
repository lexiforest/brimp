# Brimp Python binding

Brimp exposes a synchronous, Requests-style API backed by the shared native
browser runtime. It runs in-process and does not start a CDP server.

```python
import brimp

response = brimp.get("https://example.com")
print(response.status_code)
print(response.text)  # original HTTP response
print(response.html)  # DOM after JavaScript
```

Sessions own shared cookies while pages own documents, connections, and an
optional immutable proxy:

```python
with brimp.Session() as session:
    page = session.new_page(proxy="socks5h://127.0.0.1:1080")
    response = page.get("https://example.com", timeout=30)
    response.raise_for_status()
    print(page.evaluate("document.title"))
    article = page.extract(content_selector="main")
    print(article["contentMarkdown"])
    page.hover("#menu")
    page.type("#name", "agent")
    page.click("#submit")
    page.screenshot("page.png", full_page=True)
```

`Page.extract()` runs the vendored Defuddle browser bundle against the live,
post-JavaScript document. It does not create a jsdom document or make another
network request.

Private test and enterprise roots can be trusted without disabling certificate
or hostname verification:

```python
with brimp.Session(ca_bundle="path/to/cacert.pem") as session:
    response = session.new_page().get("https://internal.example")
```

Persona JSON uses the versioned schema in [`../../persona/`](../../persona/README.md):

```python
persona_json = open("persona/example.json").read()
with brimp.Session(persona_json=persona_json) as session:
    print(session.new_page().get("https://example.com").status_code)
```

Heavy browser subsystems are page-scoped and disabled by default:

```python
with brimp.Session(
    enable_worker=True,
    enable_streaming_networking=True,
    enable_canvas=True,
    enable_webgl=True,
    enable_webgpu=True,
    enable_webaudio=True,
    enable_webaudio_output=True,
    storage_path="profile/storage",
) as session:
    session.new_page().get("https://example.com")
```

`enable_webaudio=True` keeps real-time graphs device-free.
`enable_webaudio_output=True` also enables WebAudio and authorizes the system
audio output device.

See `SUPPORT.md` for the exact tested surface.

## Binary wheels

The release workflow produces self-contained CPython 3.10+ ABI3 wheels for:

- manylinux 2.28 x86-64 and ARM64;
- macOS 11 or newer on Apple silicon; and
- Windows x86-64.

Run the `Python wheels` workflow manually to exercise all builds without
publishing. Pushing a `v<project-version>` tag publishes the validated four-wheel
set through the `pypi` environment and PyPI Trusted Publishing. Source
distributions are intentionally omitted because their native dependency build
is not self-contained.

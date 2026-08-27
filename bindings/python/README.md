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

Persistent cookies, connection reuse, JavaScript evaluation, and screenshots
are available through a context-managed Session:

```python
with brimp.Session() as session:
    response = session.get("https://example.com", timeout=30)
    response.raise_for_status()
    print(session.evaluate("document.title"))
    session.screenshot("page.png", full_page=True)
```

Private test and enterprise roots can be trusted without disabling certificate
or hostname verification:

```python
with brimp.Session(ca_bundle="path/to/cacert.pem") as session:
    response = session.get("https://internal.example")
```

Persona JSON uses the versioned schema in [`../../persona/`](../../persona/README.md):

```python
persona_json = open("persona/example.json").read()
with brimp.Session(persona_json=persona_json) as session:
    print(session.get("https://example.com").status_code)
```

See `SUPPORT.md` for the exact tested surface.

## Binary wheels

The release workflow produces self-contained CPython 3.10+ ABI3 wheels for:

- manylinux 2.28 x86-64 and ARM64;
- macOS 11 or newer on Apple silicon; and
- Windows x86-64.

Run the `Python wheels` workflow manually to exercise all builds without
publishing. A non-prerelease GitHub release tagged `v<project-version>` publishes
the validated four-wheel set through the `pypi` environment and PyPI Trusted
Publishing. Source distributions are intentionally omitted because their native
dependency build is not self-contained.

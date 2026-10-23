# Brimp documentation

The public documentation site is built with Astro Starlight and configured for
`https://docs.brimp.ai`.

## Build locally

Use Python 3.11+, Node.js 24+ and Rust/Cargo. No JavaScriptCore or curl SDK is
needed for the documentation build. From the repository root:

```sh
python3 -m venv /tmp/brimp-docs-env
. /tmp/brimp-docs-env/bin/activate
python -m pip install -r docs/requirements.txt
npm ci --prefix docs
npm run dev --prefix docs
```

For a production build and link/reference checks:

```sh
npm run build --prefix docs
python -m pytest docs/tests/test_site.py
```

`PYTHON=/path/to/python` selects a different interpreter for generation.
The `generate` script runs automatically before `dev` and `build`:

| Output | Source / generator |
| --- | --- |
| `public/reference/python/` | Python source and native exception stubs; [mkdocstrings](https://mkdocstrings.github.io/python/usage/) / Griffe |
| `public/reference/node/` | `bindings/node/index.d.ts`; [TypeDoc](https://typedoc.org/) |
| `src/content/docs/api/cli.md` | `crates/cli/src/args.rs`; SDK-free Clap helper |
| `public/diagrams/*.png` | Editable SVG files in `diagrams/`; Sharp |

HTML reference output is ignored by Git and regenerated on each build. The CLI
page and PNG diagrams are checked-in artifacts; regenerate them after source
changes. Keep signature descriptions in source docstrings/comments. Pin generator
dependencies in `requirements.txt`, `package-lock.json` and the helper's Cargo lockfile.

## Verify tutorials

`fixtures/server.py` serves the example site for readers. `tests/test_examples.py`
starts it on an ephemeral loopback port and runs the actual fenced tutorial code,
checking output, files, cookies, cancellation and plugin behavior. Build the
native CLI/bindings first, then run from the repository root:

```sh
cargo build -p brimp-cli -p brimp-python -p brimp-node
python3 bindings/prepare_local.py
PYTHONPATH=bindings/python/python python3 -m pytest docs/tests/test_examples.py
```

Set the SDK environment for your platform as in the development guide. The native
CI job runs tutorial checks; the SDK-free docs job generates references and checks
links, anchors, signatures and PNG assets. `HUMAN.md` is maintainer-owned.

## GitHub Pages

The `docs-pages.yml` workflow builds and deploys this directory. In the GitHub
repository, select **Settings → Pages → GitHub Actions**, set the custom domain
to `docs.brimp.ai`, and enable HTTPS. At the DNS provider, create:

```text
Type:   CNAME
Name:   docs
Target: lexiforest.github.io
```

GitHub Actions deployments do not use a checked-in `CNAME` file; the custom
domain is stored in the repository's Pages settings.

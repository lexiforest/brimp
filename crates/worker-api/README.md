# brimp-worker-api

Native-independent configuration, errors, and extraction assets shared by the
CLI/controller and lite runtime. This crate has no JavaScriptCore, curl, or DOM
dependency. `WorkerConfig` is the payload of the lite worker's `Brimp.configure`
CDP extension; it is applied before creating contexts or pages.

Extraction types and the pinned Defuddle bundle are shared by the runtime and worker-backed CLI. Verify the bundle and licenses with
`./crates/worker-api/defuddle/verify.sh` from the workspace root.

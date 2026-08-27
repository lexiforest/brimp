# Brimp Node binding

napi-rs addon plus a small idiomatic asynchronous JavaScript adapter over the
shared `web-runtime` automation surface. It supports launch, page lifetime,
navigation, structured evaluation, document output, PNG `Buffer` values, and
idempotent close. `Page.goto` accepts an `AbortSignal`, which cancels the core
operation and network request.

This is an in-process native extension, not a CDP client. See `SUPPORT.md` for
the exhaustive tested API and error-code surface.

`launch({ personaJson })` accepts JSON text using the versioned schema and
runtime-support matrix in [`../../persona/`](../../persona/README.md).

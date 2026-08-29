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

Worker, streaming-networking, persistent-storage, Canvas 2D, WebGL, WebGPU, and
WebAudio APIs are page-scoped and disabled by default. Enable only the required
ones through `browser.newPage({ enableWorker, enableStreamingNetworking,
enableCanvas, enableWebGL, enableWebGPU, enableWebAudio, enableWebAudioOutput, storagePath,
storageQuotaBytes })`.
`enableWebAudio` keeps real-time graphs device-free; `enableWebAudioOutput`
also enables WebAudio and authorizes the system audio output device.

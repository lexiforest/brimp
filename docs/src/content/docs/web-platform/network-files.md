---
title: Network, files and streams
description: Requests, response bodies, file reads, streams and structured cloning.
---

Use Fetch and XMLHttpRequest for asynchronous page requests. Blob, File,
FileReader and Web Streams provide body handling. The host request/stream API is
described separately in [Sessions, pages and requests](/guides/automation/).

| Surface | Main limit |
| --- | --- |
| Fetch/XHR | Supported request/body behavior does not imply full browser CORS or Fetch policy conformance |
| Request.formData | URL-encoded bodies work; multipart and other unsupported formats return `undefined` |
| Streaming bodies | Unread native bytes are bounded; an unread clone/tee branch can accumulate data |
| structuredClone | Supported JS data preserves cycles; functions, DOM nodes and CryptoKeys reject |

## Requests and bodies

XHR uses the Fetch pipeline for response state, progress, cancellation and
timeouts. Headers normalize names and expose sorted iteration.

<details>
<summary>Detailed behavior and edge cases</summary>

XMLHttpRequest uses the fetch pipeline for asynchronous requests, response state,
headers, body conversion, progress events, cancellation and timeout handling.
These are runtime request contracts, not a claim of complete browser XHR/CORS
conformance. Request bodies support text, JSON, byte buffers and blobs, cloning
before consumption and propagated abort-signal state. Headers normalize names
and expose sorted iteration. Request.formData parses URL-encoded bodies through
the existing URLSearchParams/FormData primitives; other formats, including
multipart, retain the legacy undefined-returning placeholder without consuming
the body. This does not add multipart parsing or full Fetch policy enforcement.

</details>

## Files and streams

FileReader reads actual Blob bytes asynchronously. Web Streams provide readers,
writers, piping and backpressure; the native Fetch queue is bounded at 256 KiB.

<details>
<summary>Detailed behavior and edge cases</summary>

FileReader supports asynchronous text, ArrayBuffer, binary-string and data-URL
reads, progress/lifecycle events, abort and reentrant reads. Data-URL reads encode
actual Blob bytes; Canvas PNG output remains explicitly separate.

Web Streams use pinned `web-streams-polyfill`. The library owns readers/writers,
closure, errors, locking, piping,
backpressure and BYOB behavior. Fetch's byte stream releases native buffer credit
when a reader takes a chunk; unread network bytes stay bounded by the 256 KiB
queue. Response cloning tees the stream, so an unread clone branch can accumulate
buffered data as with the Streams tee algorithm. This is not a claim of complete
Fetch/body conformance.

</details>

## Structured cloning and message channels

`structuredClone` preserves cycles and shared references for supported JavaScript
values. This differs from the copied-value host bridge. Message channels use
the same page-side clone implementation.

<details>
<summary>Detailed behavior and edge cases</summary>

`structuredClone` uses `core-js-pure` with the engine's ArrayBuffer transfer. It
preserves cycles and shared references in supported JavaScript values, including
Maps/Sets, dates and binary views. Functions, DOM nodes, CryptoKeys and unsupported
platform objects reject with DataCloneError. Message channels use this shared
clone implementation too.

</details>


## Verification

The [runtime tests](https://github.com/lexiforest/brimp/tree/main/tests/runtime/environment/)
record the behavior behind this reference. Safari comparisons cover the recorded
cases; they do not establish complete browser conformance. The
[field ledger](https://github.com/lexiforest/brimp/blob/main/COVERAGE.yaml) tracks
individual decisions and evidence.

# brimp-protocol

The Rust definitions of the wire boundary between controller and worker.

- CDP `Request`, `Response`, `Event`, and `ProtocolError` envelopes.
- `MAX_FRAME_SIZE` for the four-byte big-endian framed JSON transport.
- `WorkerConfig` for `Brimp.configure`; persona is JSON, validated by the worker.
- `ExtractionOptions` and `ExtractedDocument` for `Brimp.extract`.
- `CommandError`, the structured error data used by Brimp extensions.

This package depends only on serialization/error libraries. Engine code,
cancellation, process management, persona resolution, and executable extraction
assets belong outside the protocol crate.

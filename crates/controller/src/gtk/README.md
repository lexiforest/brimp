# Linux host adapter

The message transports compile on Linux, including socketpair and WebSocket.
Browser sessions require owned worker processes; Linux worker launch and
supervision are not implemented. Their process, memory-accounting, and executable
entry adapter belongs here. WebKitGTK-specific browser integration remains in
its worker. WPE can reuse the same OS adapter.

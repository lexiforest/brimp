# Windows host adapter

`transport/named_pipe.rs` implements framed messages over a duplex byte-mode
named pipe. WebSocket message transport also compiles on Windows.

Browser sessions require owned worker processes. Local worker launch, Job Object
supervision, and memory accounting are not implemented. Those process adapters
belong here; there is no CLI option for attaching to an existing pipe or browser.

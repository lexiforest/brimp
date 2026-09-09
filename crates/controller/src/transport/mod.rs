//! Message-oriented, nonblocking transports. CDP routing belongs to `connection`.
mod framed;
#[cfg(windows)]
mod named_pipe;
#[cfg(unix)]
mod socketpair;
mod websocket;

#[cfg(windows)]
pub use named_pipe::NamedPipe;
#[cfg(unix)]
pub use socketpair::SocketPair;
pub use websocket::WebSocketTransport;

use std::io;

/// A connected message channel. Methods may be called from different threads.
/// `receive` returns `None` while no complete message is available. Implementations
/// retain partial reads/writes, bound buffers, and never block on peer activity.
/// `close` is idempotent and releases the underlying connection.
pub trait Transport: Send + Sync {
    fn send(&self, message: &[u8]) -> io::Result<()>;
    fn receive(&self) -> io::Result<Option<Vec<u8>>>;
    fn close(&self);
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn closed() -> io::Error {
    io::Error::new(io::ErrorKind::NotConnected, "transport closed")
}

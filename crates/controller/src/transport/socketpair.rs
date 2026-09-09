use super::framed::Framed;
use std::io;

pub type SocketPair = Framed<std::os::unix::net::UnixStream>;
impl SocketPair {
    pub fn from_stream(stream: std::os::unix::net::UnixStream) -> io::Result<Self> {
        stream.set_nonblocking(true)?;
        Ok(Self::new(stream))
    }
}

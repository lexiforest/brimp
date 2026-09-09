use super::{Transport, closed, invalid};
use brimp_protocol::MAX_FRAME_SIZE;
use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::sync::Mutex;

/// Length-prefixed CDP messages over a nonblocking connected byte stream.
pub struct Framed<S> {
    state: Mutex<State<S>>,
}
struct State<S> {
    stream: Option<S>,
    input: Vec<u8>,
    output: VecDeque<u8>,
}
impl<S: Read + Write> Framed<S> {
    pub(super) fn new(stream: S) -> Self {
        Self {
            state: Mutex::new(State {
                stream: Some(stream),
                input: Vec::new(),
                output: VecDeque::new(),
            }),
        }
    }
}
impl<S: Read + Write> State<S> {
    fn flush(&mut self) -> io::Result<()> {
        let stream = self.stream.as_mut().ok_or_else(closed)?;
        while !self.output.is_empty() {
            match stream.write(self.output.as_slices().0) {
                Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
                Ok(count) => {
                    self.output.drain(..count);
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
}
impl<S: Read + Write + Send> Transport for Framed<S> {
    fn send(&self, message: &[u8]) -> io::Result<()> {
        if message.len() > MAX_FRAME_SIZE {
            return Err(invalid("frame exceeds 64 MiB"));
        }
        let mut state = self.state.lock().unwrap();
        state.flush()?;
        if state.output.len() + message.len() + 4 > MAX_FRAME_SIZE + 4 {
            return Err(invalid("transport write buffer full"));
        }
        state.output.extend((message.len() as u32).to_be_bytes());
        state.output.extend(message);
        state.flush()
    }
    fn receive(&self) -> io::Result<Option<Vec<u8>>> {
        let mut state = self.state.lock().unwrap();
        state.flush()?;
        loop {
            let needed = if state.input.len() < 4 {
                4
            } else {
                let length = u32::from_be_bytes(state.input[..4].try_into().unwrap()) as usize;
                if length > MAX_FRAME_SIZE {
                    return Err(invalid("frame exceeds 64 MiB"));
                }
                if state.input.len() == length + 4 {
                    let message = state.input.split_off(4);
                    state.input.clear();
                    return Ok(Some(message));
                }
                length + 4
            };
            let mut buffer = [0; 8192];
            let count = (needed - state.input.len()).min(buffer.len());
            match state
                .stream
                .as_mut()
                .ok_or_else(closed)?
                .read(&mut buffer[..count])
            {
                Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
                Ok(count) => state.input.extend_from_slice(&buffer[..count]),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(None),
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
        }
    }
    fn close(&self) {
        let mut state = self.state.lock().unwrap();
        state.stream.take();
        state.input.clear();
        state.output.clear();
    }
}

use super::framed::Framed;
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::windows::io::AsRawHandle;
use windows_sys::Win32::Foundation::ERROR_NO_DATA;
use windows_sys::Win32::System::Pipes::{PIPE_NOWAIT, PIPE_READMODE_BYTE, SetNamedPipeHandleState};

pub struct PipeStream(File);
impl Read for PipeStream {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        match self.0.read(bytes) {
            Ok(0) => Err(io::ErrorKind::WouldBlock.into()),
            Err(error) if error.raw_os_error() == Some(ERROR_NO_DATA as i32) => {
                Err(io::ErrorKind::WouldBlock.into())
            }
            result => result,
        }
    }
}
impl Write for PipeStream {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match self.0.write(bytes) {
            Ok(0) => Err(io::ErrorKind::WouldBlock.into()),
            result => result,
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
/// Framed transport over an owned duplex byte-mode pipe handle.
pub type NamedPipe = Framed<PipeStream>;
impl NamedPipe {
    pub fn from_file(file: File) -> io::Result<Self> {
        let mode = PIPE_READMODE_BYTE | PIPE_NOWAIT;
        if unsafe {
            SetNamedPipeHandleState(
                file.as_raw_handle(),
                &mode,
                std::ptr::null(),
                std::ptr::null(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(Self::new(PipeStream(file)))
    }
}

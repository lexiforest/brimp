mod png;
mod renderer;

use std::{error::Error, fmt, io, path::Path};

pub use renderer::render_png;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScreenshotOptions {
    pub width: u32,
    pub height: u32,
    pub full_page: bool,
}

impl ScreenshotOptions {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            full_page: false,
        }
    }
}

#[derive(Debug)]
pub enum ScreenshotError {
    InvalidDimensions,
    Encode(::png::EncodingError),
    Io(io::Error),
}

impl fmt::Display for ScreenshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => formatter
                .write_str("screenshot dimensions must be between 1 and 65535 physical pixels"),
            Self::Encode(error) => write!(formatter, "could not encode PNG: {error}"),
            Self::Io(error) => write!(formatter, "could not write screenshot: {error}"),
        }
    }
}

impl Error for ScreenshotError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Encode(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::InvalidDimensions => None,
        }
    }
}

impl From<::png::EncodingError> for ScreenshotError {
    fn from(error: ::png::EncodingError) -> Self {
        Self::Encode(error)
    }
}

impl From<io::Error> for ScreenshotError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub fn save_png(path: impl AsRef<Path>, bytes: &[u8]) -> Result<(), ScreenshotError> {
    std::fs::write(path, bytes).map_err(ScreenshotError::from)
}

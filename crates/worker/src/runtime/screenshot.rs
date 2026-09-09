use anyrender::render_to_buffer;
use anyrender_vello_cpu::VelloCpuImageRenderer;
use blitz_dom::BaseDocument;

use std::{error::Error, fmt, io, path::Path};

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
    Render(String),
    Encode(::png::EncodingError),
    Io(io::Error),
}

impl fmt::Display for ScreenshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => formatter
                .write_str("screenshot dimensions must be between 1 and 65535 physical pixels"),
            Self::Render(error) => formatter.write_str(error),
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
            Self::Render(_) => None,
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

pub struct RenderedRgba {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub fn render_png(
    document: &mut BaseDocument,
    options: ScreenshotOptions,
    device_pixel_ratio: f64,
) -> Result<Vec<u8>, ScreenshotError> {
    let rendered = render_rgba(document, options, device_pixel_ratio)?;
    encode_rgba(&rendered.pixels, rendered.width, rendered.height)
}

pub fn render_rgba(
    document: &mut BaseDocument,
    options: ScreenshotOptions,
    device_pixel_ratio: f64,
) -> Result<RenderedRgba, ScreenshotError> {
    document.resolve(0.0);

    let physical_width = physical_dimension(options.width as f64, device_pixel_ratio)?;
    let css_height = if options.full_page {
        let root = document.root_element();
        let layout_height = f64::from(
            root.unrounded_layout
                .size
                .height
                .max(root.unrounded_layout.content_size.height),
        );
        let overflow_height = root.scrollable_overflow.height() / device_pixel_ratio;
        layout_height
            .max(overflow_height)
            .max(options.height as f64)
    } else {
        options.height as f64
    };
    let physical_height = physical_dimension(css_height, device_pixel_ratio)?;

    let pixels = render_to_buffer::<VelloCpuImageRenderer, _>(
        |scene| {
            blitz_paint::paint_scene(
                scene,
                document,
                device_pixel_ratio,
                physical_width,
                physical_height,
                0,
                0,
            );
        },
        physical_width,
        physical_height,
    );
    Ok(RenderedRgba {
        pixels,
        width: physical_width,
        height: physical_height,
    })
}

fn physical_dimension(css: f64, scale: f64) -> Result<u32, ScreenshotError> {
    let physical = (css * scale).ceil();
    if !physical.is_finite() || !(1.0..=f64::from(u16::MAX)).contains(&physical) {
        return Err(ScreenshotError::InvalidDimensions);
    }
    Ok(physical as u32)
}

pub fn encode_rgba(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, ScreenshotError> {
    let mut output = Vec::new();
    {
        let mut encoder = ::png::Encoder::new(&mut output, width, height);
        encoder.set_color(::png::ColorType::Rgba);
        encoder.set_depth(::png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(rgba)?;
    }
    Ok(output)
}

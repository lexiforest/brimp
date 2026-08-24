use crate::ScreenshotError;

pub(crate) fn encode_rgba(
    rgba: &[u8],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, ScreenshotError> {
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

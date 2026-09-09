#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub width: f64,
    pub height: f64,
    pub device_pixel_ratio: f64,
    pub scroll_x: f64,
    pub scroll_y: f64,
}

impl Viewport {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            device_pixel_ratio: 1.0,
            scroll_x: 0.0,
            scroll_y: 0.0,
        }
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new(800.0, 600.0)
    }
}

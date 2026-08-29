use std::{
    collections::{HashMap, hash_map::Entry},
    sync::OnceLock,
};

use browser_dom::NodeId;
use rustybuzz::{Direction, Face, UnicodeBuffer};
use skia_safe::{
    AlphaType, BlendMode, Color, Color4f, ColorChannel, ColorSpace, ColorType, Data,
    EncodedImageFormat, FilterMode, Font, FontArguments, FontMgr, GlyphId, IPoint, ISize, Image,
    ImageInfo, Matrix, Paint, Path, PathBuilder, PathFillType, Point, Point3, RRect, Rect,
    SamplingOptions, Surface, TileMode, Typeface, Vector,
    canvas::SrcRectConstraint,
    color_filters, dash_path_effect,
    gradient::{Colors as GradientColors, Gradient, Interpolation, shaders as gradient_shaders},
    image::CachingHint,
    image_filters, named_primaries, named_transfer_fn, paint, path, path_utils,
    shaders as skia_shaders, surfaces,
};
use unicode_bidi::{BidiInfo, Level};
use unicode_segmentation::UnicodeSegmentation;

const MAX_CANVAS_DIMENSION: u32 = 32_767;
const MAX_CANVAS_PIXELS: u64 = 64 * 1024 * 1024;
const WENQUANYI_FONT: &[u8] = include_bytes!("../../browser-dom/assets/fonts/wqy-microhei.ttc");
const NOTO_EMOJI_FONT: &[u8] =
    include_bytes!("../../browser-dom/assets/fonts/noto-color-emoji.ttf");
static PROPORTIONAL_TYPEFACE: OnceLock<Option<Typeface>> = OnceLock::new();
static MONOSPACE_TYPEFACE: OnceLock<Option<Typeface>> = OnceLock::new();
static EMOJI_TYPEFACE: OnceLock<Option<Typeface>> = OnceLock::new();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CanvasFontFace {
    Proportional,
    Monospace,
    Emoji,
}

struct ShapedRun {
    font: Font,
    glyphs: Vec<GlyphId>,
    positions: Vec<Point>,
}

struct ShapedText {
    runs: Vec<ShapedRun>,
    advance: f32,
    bounds: Rect,
}

pub struct CanvasRaster {
    pub node: NodeId,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ContextKind {
    TwoDimensional,
    WebGl1,
    WebGl2,
    WebGpu,
}

struct CanvasBitmap {
    width: u32,
    height: u32,
    alpha: bool,
    color_space: CanvasColorSpace,
    color_type: CanvasColorType,
    context: Option<ContextKind>,
    surface: Option<Surface>,
    path: PathBuilder,
    origin_clean: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CanvasColorSpace {
    Srgb,
    DisplayP3,
}

impl CanvasColorSpace {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "srgb" => Ok(Self::Srgb),
            "display-p3" => Ok(Self::DisplayP3),
            _ => Err(format!("unsupported Canvas color space: {value}")),
        }
    }

    fn skia(self) -> Result<ColorSpace, String> {
        match self {
            Self::Srgb => Ok(ColorSpace::new_srgb()),
            Self::DisplayP3 => ColorSpace::new_cicp(
                named_primaries::CicpId::SMPTE_EG_432_1,
                named_transfer_fn::CicpId::SRGB,
            )
            .ok_or_else(|| "Skia could not create the Display-P3 color space".to_owned()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CanvasColorType {
    Unorm8,
    Float16,
}

impl CanvasColorType {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "unorm8" | "rgba-unorm8" => Ok(Self::Unorm8),
            "float16" | "rgba-float16" => Ok(Self::Float16),
            _ => Err(format!("unsupported Canvas color type: {value}")),
        }
    }

    const fn bytes_per_pixel(self) -> usize {
        match self {
            Self::Unorm8 => 4,
            Self::Float16 => 8,
        }
    }

    const fn skia(self) -> ColorType {
        match self {
            Self::Unorm8 => ColorType::RGBA8888,
            Self::Float16 => ColorType::RGBAF16,
        }
    }
}

enum GradientKind {
    Linear([f32; 4]),
    Radial([f32; 6]),
    Conic([f32; 3]),
}

struct CanvasGradientData {
    kind: GradientKind,
    transform: [f32; 6],
    stops: Vec<(f32, Color4f)>,
}

struct CanvasPatternData {
    image: Image,
    tile_modes: (TileMode, TileMode),
    transform: [f32; 6],
    origin_clean: bool,
}

struct ImageBitmapData {
    image: Image,
    origin_clean: bool,
}

#[derive(Clone, Copy)]
pub(crate) enum CanvasPaintStyle {
    Color([f32; 4]),
    Gradient { id: u64, alpha: f32 },
    Pattern { id: u64, alpha: f32 },
}

pub(crate) struct CanvasStrokeStyle {
    pub width: f32,
    pub cap: String,
    pub join: String,
    pub miter_limit: f32,
    pub dash: Vec<f32>,
    pub dash_offset: f32,
}

pub(crate) struct CanvasShadowStyle {
    pub color: [f32; 4],
    pub blur: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

#[derive(Clone, Copy)]
pub(crate) enum CanvasFilterInput {
    SourceGraphic,
    Operation(usize),
}

pub(crate) enum CanvasLightSource {
    Distant {
        azimuth: f32,
        elevation: f32,
    },
    Point {
        position: [f32; 3],
    },
    Spot {
        position: [f32; 3],
        target: [f32; 3],
        falloff_exponent: f32,
        cutoff_angle: f32,
    },
}

pub(crate) enum CanvasFilterOperation {
    Blur {
        sigma_x: f32,
        sigma_y: f32,
        input: CanvasFilterInput,
    },
    Offset {
        x: f32,
        y: f32,
        input: CanvasFilterInput,
    },
    ColorMatrix {
        matrix: [f32; 20],
        input: CanvasFilterInput,
    },
    ComponentTransfer {
        tables: Box<[[u8; 256]; 4]>,
        input: CanvasFilterInput,
    },
    Morphology {
        dilate: bool,
        radius_x: f32,
        radius_y: f32,
        input: CanvasFilterInput,
    },
    Flood {
        color: [f32; 4],
    },
    ConvolveMatrix {
        width: i32,
        height: i32,
        kernel: Vec<f32>,
        gain: f32,
        bias: f32,
        target_x: i32,
        target_y: i32,
        edge_mode: String,
        convolve_alpha: bool,
        input: CanvasFilterInput,
    },
    DisplacementMap {
        scale: f32,
        x_channel: String,
        y_channel: String,
        input: CanvasFilterInput,
        input2: CanvasFilterInput,
    },
    Lighting {
        specular: bool,
        color: [f32; 3],
        surface_scale: f32,
        constant: f32,
        exponent: f32,
        light: CanvasLightSource,
        input: CanvasFilterInput,
    },
    DropShadow {
        shadow: CanvasShadowStyle,
        input: CanvasFilterInput,
    },
    Blend {
        mode: String,
        input: CanvasFilterInput,
        input2: CanvasFilterInput,
    },
    Composite {
        operator: String,
        coefficients: [f32; 4],
        input: CanvasFilterInput,
        input2: CanvasFilterInput,
    },
    Merge(Vec<CanvasFilterInput>),
}

pub(crate) struct CanvasDrawEffects {
    pub shadow: CanvasShadowStyle,
    pub filters: Vec<CanvasFilterOperation>,
}

impl CanvasBitmap {
    fn new(width: u32, height: u32) -> Result<Self, String> {
        validate_dimensions(width, height)?;
        Ok(Self {
            width,
            height,
            alpha: true,
            color_space: CanvasColorSpace::Srgb,
            color_type: CanvasColorType::Unorm8,
            context: None,
            surface: new_canvas_surface(
                width,
                height,
                true,
                CanvasColorSpace::Srgb,
                CanvasColorType::Unorm8,
            )?,
            path: PathBuilder::new(),
            origin_clean: true,
        })
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        if self.width == width && self.height == height {
            return Ok(());
        }
        validate_dimensions(width, height)?;
        self.width = width;
        self.height = height;
        self.surface =
            new_canvas_surface(width, height, self.alpha, self.color_space, self.color_type)?;
        self.path.reset();
        self.origin_clean = true;
        Ok(())
    }

    fn reset(&mut self, width: u32, height: u32) -> Result<(), String> {
        validate_dimensions(width, height)?;
        self.width = width;
        self.height = height;
        self.surface =
            new_canvas_surface(width, height, self.alpha, self.color_space, self.color_type)?;
        self.path.reset();
        self.origin_clean = true;
        Ok(())
    }

    fn force_opaque(&mut self) {
        if self.alpha {
            return;
        }
        if let Some(surface) = self.surface.as_mut() {
            surface
                .canvas()
                .draw_color(Color4f::new(0.0, 0.0, 0.0, 1.0), BlendMode::DstOver);
        }
    }
}

#[derive(Default)]
pub(crate) struct CanvasStore {
    entries: HashMap<NodeId, CanvasBitmap>,
    gradients: HashMap<u64, CanvasGradientData>,
    patterns: HashMap<u64, CanvasPatternData>,
    image_bitmaps: HashMap<u64, ImageBitmapData>,
    paths: HashMap<u64, PathBuilder>,
    next_gradient: u64,
    next_pattern: u64,
    next_image_bitmap: u64,
    next_path: u64,
}

impl CanvasStore {
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        self.gradients.clear();
        self.patterns.clear();
        self.image_bitmaps.clear();
        self.paths.clear();
    }

    pub(crate) fn create_path(&mut self) -> Result<u64, String> {
        self.insert_path(PathBuilder::new())
    }

    pub(crate) fn copy_path(&mut self, source: u64) -> Result<u64, String> {
        let path = self.paths.get(&source).ok_or("unknown Path2D")?.clone();
        self.insert_path(path)
    }

    pub(crate) fn create_svg_path(&mut self, source: &str) -> Result<u64, String> {
        let path = Path::from_svg(source).ok_or("invalid SVG path data")?;
        let mut builder = PathBuilder::new();
        builder.add_path(&path, None);
        self.insert_path(builder)
    }

    pub(crate) fn add_path(
        &mut self,
        target: u64,
        source: u64,
        transform: [f32; 6],
    ) -> Result<(), String> {
        let source = self
            .paths
            .get(&source)
            .ok_or("unknown source Path2D")?
            .snapshot();
        self.paths
            .get_mut(&target)
            .ok_or("unknown target Path2D")?
            .add_path_with_transform(
                &source,
                &Matrix::from_affine(&transform),
                path::AddPathMode::Append,
            );
        Ok(())
    }

    pub(crate) fn create_image_bitmap(
        &mut self,
        source: NodeId,
        width: u32,
        height: u32,
    ) -> Result<(u64, u32, u32), String> {
        let (image, origin_clean) = {
            let source = self.bitmap(source, width, height)?;
            let image = source
                .surface
                .as_mut()
                .ok_or("ImageBitmap source has no bitmap")?
                .image_snapshot();
            (image, source.origin_clean)
        };
        let id = self.insert_image_bitmap(image, width, height, origin_clean)?;
        Ok((id, width, height))
    }

    pub(crate) fn decode_image_bitmap(&mut self, bytes: &[u8]) -> Result<(u64, u32, u32), String> {
        let image = Image::from_encoded(Data::new_copy(bytes))
            .ok_or("Blob does not contain a supported raster image")?;
        let width = u32::try_from(image.width()).map_err(|_| "decoded image width is invalid")?;
        let height =
            u32::try_from(image.height()).map_err(|_| "decoded image height is invalid")?;
        validate_dimensions(width, height)?;
        let id = self.insert_image_bitmap(image, width, height, true)?;
        Ok((id, width, height))
    }

    fn insert_image_bitmap(
        &mut self,
        image: Image,
        width: u32,
        height: u32,
        origin_clean: bool,
    ) -> Result<u64, String> {
        validate_dimensions(width, height)?;
        self.next_image_bitmap = self
            .next_image_bitmap
            .checked_add(1)
            .ok_or("ImageBitmap id overflow")?;
        let id = self.next_image_bitmap;
        self.image_bitmaps.insert(
            id,
            ImageBitmapData {
                image,
                origin_clean,
            },
        );
        Ok(id)
    }

    pub(crate) fn destroy_image_bitmap(&mut self, bitmap: u64) -> Result<(), String> {
        self.image_bitmaps
            .remove(&bitmap)
            .ok_or("unknown ImageBitmap")?;
        Ok(())
    }

    pub(crate) fn image_bitmap_rgba(
        &self,
        bitmap: u64,
    ) -> Result<(u32, u32, Vec<u8>, bool), String> {
        self.image_bitmap_rgba_in_color_space(bitmap, CanvasColorSpace::Srgb)
    }

    pub(crate) fn image_bitmap_rgba_in_color_space(
        &self,
        bitmap: u64,
        color_space: CanvasColorSpace,
    ) -> Result<(u32, u32, Vec<u8>, bool), String> {
        let bitmap = self
            .image_bitmaps
            .get(&bitmap)
            .ok_or("unknown ImageBitmap")?;
        let width = u32::try_from(bitmap.image.width()).map_err(|_| "invalid ImageBitmap width")?;
        let height =
            u32::try_from(bitmap.image.height()).map_err(|_| "invalid ImageBitmap height")?;
        let row_bytes = usize::try_from(width)
            .ok()
            .and_then(|width| width.checked_mul(4))
            .ok_or("ImageBitmap row is too large")?;
        let mut pixels = vec![0; pixel_byte_len(width, height)?];
        if !bitmap.image.read_pixels(
            &image_data_info(width, height, color_space, CanvasColorType::Unorm8)?,
            &mut pixels,
            row_bytes,
            (0, 0),
            CachingHint::Disallow,
        ) {
            return Err("Skia could not read ImageBitmap pixels".to_owned());
        }
        Ok((width, height, pixels, bitmap.origin_clean))
    }

    pub(crate) fn convert_image_data_to_unorm8(
        width: u32,
        height: u32,
        pixels: &[u8],
        source_color_space: CanvasColorSpace,
        source_color_type: CanvasColorType,
        destination_color_space: CanvasColorSpace,
    ) -> Result<Vec<u8>, String> {
        if pixels.len() != pixel_byte_len_for(width, height, source_color_type)? {
            return Err("image data byte length does not match its dimensions".to_owned());
        }
        let mut surface =
            new_canvas_surface(width, height, true, source_color_space, source_color_type)?
                .ok_or("image data has no pixels")?;
        let source_row_bytes = width as usize * source_color_type.bytes_per_pixel();
        if !surface.canvas().write_pixels(
            &image_data_info(width, height, source_color_space, source_color_type)?,
            pixels,
            source_row_bytes,
            (0, 0),
        ) {
            return Err("Skia could not import image data pixels".to_owned());
        }
        let mut converted = vec![0; pixel_byte_len(width, height)?];
        if !surface.read_pixels(
            &image_data_info(
                width,
                height,
                destination_color_space,
                CanvasColorType::Unorm8,
            )?,
            &mut converted,
            width as usize * 4,
            (0, 0),
        ) {
            return Err("Skia could not convert image data pixels".to_owned());
        }
        Ok(converted)
    }

    pub(crate) fn create_linear_gradient(
        &mut self,
        coordinates: [f32; 4],
        transform: [f32; 6],
    ) -> Result<u64, String> {
        if !coordinates.iter().all(|value| value.is_finite()) {
            return Err("Canvas gradient coordinates must be finite".to_owned());
        }
        self.insert_gradient(GradientKind::Linear(coordinates), transform)
    }

    pub(crate) fn create_radial_gradient(
        &mut self,
        coordinates: [f32; 6],
        transform: [f32; 6],
    ) -> Result<u64, String> {
        if !coordinates.iter().all(|value| value.is_finite()) {
            return Err("Canvas gradient coordinates must be finite".to_owned());
        }
        if coordinates[2] < 0.0 || coordinates[5] < 0.0 {
            return Err("Canvas gradient radius must not be negative".to_owned());
        }
        self.insert_gradient(GradientKind::Radial(coordinates), transform)
    }

    pub(crate) fn create_conic_gradient(
        &mut self,
        coordinates: [f32; 3],
        transform: [f32; 6],
    ) -> Result<u64, String> {
        if !coordinates.iter().all(|value| value.is_finite()) {
            return Err("Canvas gradient coordinates must be finite".to_owned());
        }
        self.insert_gradient(GradientKind::Conic(coordinates), transform)
    }

    pub(crate) fn add_color_stop(
        &mut self,
        gradient: u64,
        offset: f32,
        color: [f32; 4],
    ) -> Result<(), String> {
        let gradient = self
            .gradients
            .get_mut(&gradient)
            .ok_or("unknown CanvasGradient")?;
        gradient
            .stops
            .push((offset, Color4f::new(color[0], color[1], color[2], color[3])));
        gradient
            .stops
            .sort_by(|left, right| left.0.total_cmp(&right.0));
        Ok(())
    }

    pub(crate) fn create_pattern(
        &mut self,
        source: NodeId,
        width: u32,
        height: u32,
        repetition: &str,
    ) -> Result<u64, String> {
        let bitmap = self.bitmap(source, width, height)?;
        let origin_clean = bitmap.origin_clean;
        let image = bitmap
            .surface
            .as_mut()
            .ok_or("Canvas pattern source has no bitmap")?
            .image_snapshot();
        self.insert_pattern(image, repetition, origin_clean)
    }

    pub(crate) fn create_rgba_pattern(
        &mut self,
        width: u32,
        height: u32,
        pixels: &[u8],
        repetition: &str,
        origin_clean: bool,
    ) -> Result<u64, String> {
        if pixels.len() != pixel_byte_len(width, height)? {
            return Err("decoded image byte length does not match its dimensions".to_owned());
        }
        let mut source = new_surface(width, height)?.ok_or("decoded image has no bitmap")?;
        let _ = source.canvas().write_pixels(
            &rgba_info(width, height),
            pixels,
            width as usize * 4,
            (0, 0),
        );
        self.insert_pattern(source.image_snapshot(), repetition, origin_clean)
    }

    pub(crate) fn create_image_bitmap_pattern(
        &mut self,
        bitmap: u64,
        repetition: &str,
    ) -> Result<u64, String> {
        let (image, origin_clean) = {
            let bitmap = self
                .image_bitmaps
                .get(&bitmap)
                .ok_or("unknown ImageBitmap")?;
            (bitmap.image.clone(), bitmap.origin_clean)
        };
        self.insert_pattern(image, repetition, origin_clean)
    }

    fn insert_pattern(
        &mut self,
        image: Image,
        repetition: &str,
        origin_clean: bool,
    ) -> Result<u64, String> {
        let tile_modes = match repetition {
            "repeat" => (TileMode::Repeat, TileMode::Repeat),
            "repeat-x" => (TileMode::Repeat, TileMode::Decal),
            "repeat-y" => (TileMode::Decal, TileMode::Repeat),
            "no-repeat" => (TileMode::Decal, TileMode::Decal),
            _ => return Err("invalid Canvas pattern repetition".to_owned()),
        };
        self.next_pattern = self
            .next_pattern
            .checked_add(1)
            .ok_or("Canvas pattern id overflow")?;
        let id = self.next_pattern;
        self.patterns.insert(
            id,
            CanvasPatternData {
                image,
                tile_modes,
                transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                origin_clean,
            },
        );
        Ok(id)
    }

    pub(crate) fn set_pattern_transform(
        &mut self,
        pattern: u64,
        transform: [f32; 6],
    ) -> Result<(), String> {
        if !transform.iter().all(|value| value.is_finite()) {
            return Err("Canvas pattern transform must be finite".to_owned());
        }
        self.patterns
            .get_mut(&pattern)
            .ok_or("unknown CanvasPattern")?
            .transform = transform;
        Ok(())
    }

    pub(crate) fn rasters(&mut self) -> Vec<CanvasRaster> {
        let ids = self.entries.keys().copied().collect::<Vec<_>>();
        ids.into_iter()
            .filter_map(|id| {
                let bitmap = self.entries.get_mut(&id)?;
                let surface = bitmap.surface.as_mut()?;
                let row_bytes = bitmap.width as usize * 4;
                let mut pixels = vec![0; row_bytes * bitmap.height as usize];
                surface
                    .read_pixels(
                        &rgba_info(bitmap.width, bitmap.height),
                        &mut pixels,
                        row_bytes,
                        (0, 0),
                    )
                    .then_some(CanvasRaster {
                        node: id,
                        width: bitmap.width,
                        height: bitmap.height,
                        pixels,
                    })
            })
            .collect()
    }

    pub(crate) fn acquire_2d(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        alpha: bool,
        color_space: CanvasColorSpace,
        color_type: CanvasColorType,
    ) -> Result<bool, String> {
        let bitmap = self.bitmap(id, width, height)?;
        match bitmap.context {
            None => {
                bitmap.alpha = alpha;
                bitmap.color_space = color_space;
                bitmap.color_type = color_type;
                bitmap.surface = new_canvas_surface(width, height, alpha, color_space, color_type)?;
                bitmap.context = Some(ContextKind::TwoDimensional);
                Ok(true)
            }
            Some(ContextKind::TwoDimensional) => Ok(true),
            Some(ContextKind::WebGl1 | ContextKind::WebGl2 | ContextKind::WebGpu) => Ok(false),
        }
    }

    pub(crate) fn can_acquire_webgl(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        version: u8,
    ) -> Result<bool, String> {
        let bitmap = self.bitmap(id, width, height)?;
        Ok(matches!(
            (bitmap.context, version),
            (None, _) | (Some(ContextKind::WebGl1), 1) | (Some(ContextKind::WebGl2), 2)
        ))
    }

    pub(crate) fn acquire_webgpu(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
    ) -> Result<bool, String> {
        let bitmap = self.bitmap(id, width, height)?;
        match bitmap.context {
            None => {
                bitmap.context = Some(ContextKind::WebGpu);
                Ok(true)
            }
            Some(ContextKind::WebGpu) => Ok(true),
            Some(ContextKind::TwoDimensional | ContextKind::WebGl1 | ContextKind::WebGl2) => {
                Ok(false)
            }
        }
    }

    pub(crate) fn commit_webgl(&mut self, id: NodeId, version: u8) -> Result<(), String> {
        let bitmap = self.entries.get_mut(&id).ok_or("unknown canvas")?;
        bitmap.context = Some(if version == 2 {
            ContextKind::WebGl2
        } else {
            ContextKind::WebGl1
        });
        Ok(())
    }

    pub(crate) fn is_webgl(&self, id: NodeId) -> bool {
        self.entries.get(&id).is_some_and(|bitmap| {
            matches!(
                bitmap.context,
                Some(ContextKind::WebGl1 | ContextKind::WebGl2)
            )
        })
    }

    pub(crate) fn origin_clean(&self, id: NodeId) -> bool {
        self.entries
            .get(&id)
            .is_none_or(|bitmap| bitmap.origin_clean)
    }

    pub(crate) fn webgl_dimensions(&self) -> Vec<(NodeId, u32, u32)> {
        self.entries
            .iter()
            .filter_map(|(&id, bitmap)| {
                matches!(
                    bitmap.context,
                    Some(ContextKind::WebGl1 | ContextKind::WebGl2)
                )
                .then_some((id, bitmap.width, bitmap.height))
            })
            .collect()
    }

    pub(crate) fn reset(&mut self, id: NodeId, width: u32, height: u32) -> Result<(), String> {
        let bitmap = match self.entries.entry(id) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(CanvasBitmap::new(width, height)?),
        };
        bitmap.reset(width, height)
    }

    pub(crate) fn begin_path(&mut self, id: NodeId, width: u32, height: u32) -> Result<(), String> {
        self.bitmap(id, width, height)?.path.reset();
        Ok(())
    }

    pub(crate) fn close_path(&mut self, id: NodeId, width: u32, height: u32) -> Result<(), String> {
        self.bitmap(id, width, height)?.path.close();
        Ok(())
    }

    pub(crate) fn save(&mut self, id: NodeId, width: u32, height: u32) -> Result<(), String> {
        if let Some(surface) = self.bitmap(id, width, height)?.surface.as_mut() {
            surface.canvas().save();
        }
        Ok(())
    }

    pub(crate) fn restore(&mut self, id: NodeId, width: u32, height: u32) -> Result<(), String> {
        if let Some(surface) = self.bitmap(id, width, height)?.surface.as_mut() {
            surface.canvas().restore();
        }
        Ok(())
    }

    pub(crate) fn clip_path(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        path_id: Option<u64>,
        even_odd: bool,
    ) -> Result<(), String> {
        let selected = path_id
            .map(|path| {
                self.paths
                    .get(&path)
                    .ok_or("unknown Path2D")
                    .map(PathBuilder::snapshot)
            })
            .transpose()?;
        let bitmap = self.bitmap(id, width, height)?;
        let mut path = selected.unwrap_or_else(|| bitmap.path.snapshot());
        path.set_fill_type(if even_odd {
            PathFillType::EvenOdd
        } else {
            PathFillType::Winding
        });
        if let Some(surface) = bitmap.surface.as_mut() {
            surface.canvas().clip_path(&path, None, true);
        }
        Ok(())
    }

    pub(crate) fn path_points(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        operation: &str,
        points: &[f64],
        transform: [f32; 6],
    ) -> Result<(), String> {
        let path = &mut self.bitmap(id, width, height)?.path;
        append_path_points(path, operation, points, transform)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn path_arc_to(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        first: [f64; 2],
        second: [f64; 2],
        radius: f64,
        transform: [f32; 6],
    ) -> Result<(), String> {
        let path = &mut self.bitmap(id, width, height)?.path;
        append_path_arc_to(path, first, second, radius, transform)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn path_rect(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        x: f64,
        y: f64,
        rect_width: f64,
        rect_height: f64,
        transform: [f32; 6],
    ) -> Result<(), String> {
        let path = &mut self.bitmap(id, width, height)?.path;
        append_path_rect(path, [x, y, rect_width, rect_height], transform)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn path_round_rect(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        rect: [f64; 4],
        radii: [f64; 8],
        transform: [f32; 6],
    ) -> Result<(), String> {
        let path = &mut self.bitmap(id, width, height)?.path;
        append_path_round_rect(path, rect, radii, transform)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn path_arc(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        x: f64,
        y: f64,
        radius: f64,
        start: f64,
        sweep: f64,
        transform: [f32; 6],
    ) -> Result<(), String> {
        let path = &mut self.bitmap(id, width, height)?.path;
        append_path_arc(path, [x, y], radius, start, sweep, transform)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn path_ellipse(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        center: [f64; 2],
        radii: [f64; 2],
        rotation: f64,
        start: f64,
        sweep: f64,
        transform: [f32; 6],
    ) -> Result<(), String> {
        let path = &mut self.bitmap(id, width, height)?.path;
        append_path_ellipse(path, center, radii, rotation, start, sweep, transform)
    }

    pub(crate) fn path2d_close(&mut self, path: u64) -> Result<(), String> {
        self.paths.get_mut(&path).ok_or("unknown Path2D")?.close();
        Ok(())
    }

    pub(crate) fn path2d_points(
        &mut self,
        path: u64,
        operation: &str,
        points: &[f64],
    ) -> Result<(), String> {
        append_path_points(
            self.paths.get_mut(&path).ok_or("unknown Path2D")?,
            operation,
            points,
            identity_matrix(),
        )
    }

    pub(crate) fn path2d_arc_to(
        &mut self,
        path: u64,
        first: [f64; 2],
        second: [f64; 2],
        radius: f64,
    ) -> Result<(), String> {
        append_path_arc_to(
            self.paths.get_mut(&path).ok_or("unknown Path2D")?,
            first,
            second,
            radius,
            identity_matrix(),
        )
    }

    pub(crate) fn path2d_rect(&mut self, path: u64, rect: [f64; 4]) -> Result<(), String> {
        append_path_rect(
            self.paths.get_mut(&path).ok_or("unknown Path2D")?,
            rect,
            identity_matrix(),
        )
    }

    pub(crate) fn path2d_round_rect(
        &mut self,
        path: u64,
        rect: [f64; 4],
        radii: [f64; 8],
    ) -> Result<(), String> {
        append_path_round_rect(
            self.paths.get_mut(&path).ok_or("unknown Path2D")?,
            rect,
            radii,
            identity_matrix(),
        )
    }

    pub(crate) fn path2d_arc(
        &mut self,
        path: u64,
        center: [f64; 2],
        radius: f64,
        start: f64,
        sweep: f64,
    ) -> Result<(), String> {
        append_path_arc(
            self.paths.get_mut(&path).ok_or("unknown Path2D")?,
            center,
            radius,
            start,
            sweep,
            identity_matrix(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn path2d_ellipse(
        &mut self,
        path: u64,
        center: [f64; 2],
        radii: [f64; 2],
        rotation: f64,
        start: f64,
        sweep: f64,
    ) -> Result<(), String> {
        append_path_ellipse(
            self.paths.get_mut(&path).ok_or("unknown Path2D")?,
            center,
            radii,
            rotation,
            start,
            sweep,
            identity_matrix(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_path(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        path_id: Option<u64>,
        stroke: bool,
        even_odd: bool,
        style: CanvasPaintStyle,
        stroke_style: &CanvasStrokeStyle,
        effects: &CanvasDrawEffects,
        composite: &str,
    ) -> Result<(), String> {
        let selected = path_id
            .map(|path| {
                self.paths
                    .get(&path)
                    .ok_or("unknown Path2D")
                    .map(PathBuilder::snapshot)
            })
            .transpose()?;
        let origin_clean = self.paint_origin_clean(style)?;
        let mut paint =
            self.paint_for(style, composite, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0], effects)?;
        let bitmap = self.bitmap(id, width, height)?;
        bitmap.origin_clean &= origin_clean;
        let Some(surface) = bitmap.surface.as_mut() else {
            return Ok(());
        };
        let mut path = selected.unwrap_or_else(|| bitmap.path.snapshot());
        path.set_fill_type(if even_odd {
            PathFillType::EvenOdd
        } else {
            PathFillType::Winding
        });
        if stroke {
            apply_stroke(&mut paint, stroke_style)?;
        }
        surface.canvas().draw_path(&path, &paint);
        bitmap.force_opaque();
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn point_in_path(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        path_id: Option<u64>,
        x: f64,
        y: f64,
        even_odd: bool,
    ) -> Result<bool, String> {
        if !all_finite(&[x, y]) {
            return Ok(false);
        }
        let mut path = match path_id {
            Some(path) => self.paths.get(&path).ok_or("unknown Path2D")?.snapshot(),
            None => self.bitmap(id, width, height)?.path.snapshot(),
        };
        path.set_fill_type(if even_odd {
            PathFillType::EvenOdd
        } else {
            PathFillType::Winding
        });
        Ok(path.contains((x as f32, y as f32)))
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn point_in_stroke(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        path_id: Option<u64>,
        x: f64,
        y: f64,
        stroke_style: &CanvasStrokeStyle,
    ) -> Result<bool, String> {
        if !all_finite(&[x, y]) {
            return Ok(false);
        }
        let path = match path_id {
            Some(path) => self.paths.get(&path).ok_or("unknown Path2D")?.snapshot(),
            None => self.bitmap(id, width, height)?.path.snapshot(),
        };
        let mut paint = Paint::default();
        apply_stroke(&mut paint, stroke_style)?;
        let mut stroked = PathBuilder::new();
        if !path_utils::fill_path_with_paint(&path, &paint, &mut stroked, None, None) {
            return Ok(false);
        }
        Ok(stroked.snapshot().contains((x as f32, y as f32)))
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn fill_rect(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        x: f64,
        y: f64,
        rect_width: f64,
        rect_height: f64,
        style: CanvasPaintStyle,
        transform: [f32; 6],
        effects: &CanvasDrawEffects,
        composite: &str,
    ) -> Result<(), String> {
        if !all_finite(&[x, y, rect_width, rect_height]) {
            return Ok(());
        }
        let origin_clean = self.paint_origin_clean(style)?;
        let paint = self.paint_for(style, composite, transform, effects)?;
        let bitmap = self.bitmap(id, width, height)?;
        bitmap.origin_clean &= origin_clean;
        let Some(surface) = bitmap.surface.as_mut() else {
            return Ok(());
        };
        let canvas = surface.canvas();
        canvas.save();
        canvas.concat(&Matrix::from_affine(&transform));
        canvas.draw_rect(
            Rect::from_xywh(x as f32, y as f32, rect_width as f32, rect_height as f32),
            &paint,
        );
        canvas.restore();
        bitmap.force_opaque();
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn stroke_rect(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        x: f64,
        y: f64,
        rect_width: f64,
        rect_height: f64,
        style: CanvasPaintStyle,
        transform: [f32; 6],
        stroke_style: &CanvasStrokeStyle,
        effects: &CanvasDrawEffects,
        composite: &str,
    ) -> Result<(), String> {
        if !all_finite(&[x, y, rect_width, rect_height]) {
            return Ok(());
        }
        let origin_clean = self.paint_origin_clean(style)?;
        let mut paint = self.paint_for(style, composite, transform, effects)?;
        let bitmap = self.bitmap(id, width, height)?;
        bitmap.origin_clean &= origin_clean;
        let Some(surface) = bitmap.surface.as_mut() else {
            return Ok(());
        };
        apply_stroke(&mut paint, stroke_style)?;
        let canvas = surface.canvas();
        canvas.save();
        canvas.concat(&Matrix::from_affine(&transform));
        canvas.draw_rect(
            Rect::from_xywh(x as f32, y as f32, rect_width as f32, rect_height as f32),
            &paint,
        );
        canvas.restore();
        bitmap.force_opaque();
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_canvas(
        &mut self,
        target: NodeId,
        target_width: u32,
        target_height: u32,
        source: NodeId,
        source_width: u32,
        source_height: u32,
        source_rect: [f32; 4],
        destination_rect: [f32; 4],
        alpha: f32,
        transform: [f32; 6],
        smoothing: bool,
        effects: &CanvasDrawEffects,
        composite: &str,
    ) -> Result<(), String> {
        if !source_rect
            .iter()
            .chain(destination_rect.iter())
            .all(|value| value.is_finite())
        {
            return Ok(());
        }
        let source = self.bitmap(source, source_width, source_height)?;
        let origin_clean = source.origin_clean;
        let image = source
            .surface
            .as_mut()
            .map(|surface| surface.image_snapshot());
        let Some(image) = image else {
            return Ok(());
        };
        self.draw_image(
            target,
            target_width,
            target_height,
            image,
            source_rect,
            destination_rect,
            alpha,
            transform,
            smoothing,
            effects,
            composite,
            origin_clean,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_rgba_image(
        &mut self,
        target: NodeId,
        target_width: u32,
        target_height: u32,
        image_width: u32,
        image_height: u32,
        pixels: &[u8],
        source_rect: [f32; 4],
        destination_rect: [f32; 4],
        alpha: f32,
        transform: [f32; 6],
        smoothing: bool,
        effects: &CanvasDrawEffects,
        composite: &str,
        origin_clean: bool,
    ) -> Result<(), String> {
        if pixels.len() != pixel_byte_len(image_width, image_height)? {
            return Err("decoded image byte length does not match its dimensions".to_owned());
        }
        let mut source =
            new_surface(image_width, image_height)?.ok_or("decoded image has no bitmap")?;
        let _ = source.canvas().write_pixels(
            &rgba_info(image_width, image_height),
            pixels,
            image_width as usize * 4,
            (0, 0),
        );
        self.draw_image(
            target,
            target_width,
            target_height,
            source.image_snapshot(),
            source_rect,
            destination_rect,
            alpha,
            transform,
            smoothing,
            effects,
            composite,
            origin_clean,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_image_bitmap(
        &mut self,
        target: NodeId,
        target_width: u32,
        target_height: u32,
        bitmap: u64,
        source_rect: [f32; 4],
        destination_rect: [f32; 4],
        alpha: f32,
        transform: [f32; 6],
        smoothing: bool,
        effects: &CanvasDrawEffects,
        composite: &str,
    ) -> Result<(), String> {
        let (image, origin_clean) = {
            let bitmap = self
                .image_bitmaps
                .get(&bitmap)
                .ok_or("unknown ImageBitmap")?;
            (bitmap.image.clone(), bitmap.origin_clean)
        };
        self.draw_image(
            target,
            target_width,
            target_height,
            image,
            source_rect,
            destination_rect,
            alpha,
            transform,
            smoothing,
            effects,
            composite,
            origin_clean,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_image(
        &mut self,
        target: NodeId,
        target_width: u32,
        target_height: u32,
        image: Image,
        source_rect: [f32; 4],
        destination_rect: [f32; 4],
        alpha: f32,
        transform: [f32; 6],
        smoothing: bool,
        effects: &CanvasDrawEffects,
        composite: &str,
        origin_clean: bool,
    ) -> Result<(), String> {
        let bitmap = self.bitmap(target, target_width, target_height)?;
        bitmap.origin_clean &= origin_clean;
        let Some(surface) = bitmap.surface.as_mut() else {
            return Ok(());
        };
        let source_rect = Rect::from_xywh(
            source_rect[0],
            source_rect[1],
            source_rect[2],
            source_rect[3],
        );
        let destination_rect = Rect::from_xywh(
            destination_rect[0],
            destination_rect[1],
            destination_rect[2],
            destination_rect[3],
        );
        let mut paint = Paint::default();
        paint
            .set_anti_alias(true)
            .set_alpha_f(alpha)
            .set_blend_mode(blend_mode(composite)?);
        apply_effects(&mut paint, effects)?;
        let canvas = surface.canvas();
        canvas.save();
        canvas.concat(&Matrix::from_affine(&transform));
        canvas.draw_image_rect_with_sampling_options(
            image,
            Some((&source_rect, SrcRectConstraint::Strict)),
            destination_rect,
            SamplingOptions::from(if smoothing {
                FilterMode::Linear
            } else {
                FilterMode::Nearest
            }),
            &paint,
        );
        canvas.restore();
        bitmap.force_opaque();
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_text(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        text: &str,
        x: f32,
        y: f32,
        font_size: f32,
        font_family: &str,
        direction: &str,
        max_width: Option<f32>,
        stroke: bool,
        stroke_style: &CanvasStrokeStyle,
        style: CanvasPaintStyle,
        transform: [f32; 6],
        effects: &CanvasDrawEffects,
        composite: &str,
    ) -> Result<(), String> {
        if ![x, y, font_size].iter().all(|value| value.is_finite()) {
            return Ok(());
        }
        let origin_clean = self.paint_origin_clean(style)?;
        let mut paint = self.paint_for(style, composite, transform, effects)?;
        if stroke {
            apply_stroke(&mut paint, stroke_style)?;
        }
        let family = CanvasFontFace::parse(font_family)?;
        let shaped = shape_text(text, font_size, direction, family)?;
        let scale = max_width
            .filter(|max_width| {
                max_width.is_finite() && *max_width > 0.0 && shaped.advance > *max_width
            })
            .map_or(1.0, |max_width| max_width / shaped.advance);
        let bitmap = self.bitmap(id, width, height)?;
        bitmap.origin_clean &= origin_clean;
        let Some(surface) = bitmap.surface.as_mut() else {
            return Ok(());
        };
        let canvas = surface.canvas();
        canvas.save();
        canvas.concat(&Matrix::from_affine(&transform));
        if scale < 1.0 {
            canvas.translate((x, y));
            canvas.scale((scale, 1.0));
            for run in &shaped.runs {
                canvas.draw_glyphs_at(
                    &run.glyphs,
                    run.positions.as_slice(),
                    (0.0, 0.0),
                    &run.font,
                    &paint,
                );
            }
        } else {
            for run in &shaped.runs {
                canvas.draw_glyphs_at(
                    &run.glyphs,
                    run.positions.as_slice(),
                    (x, y),
                    &run.font,
                    &paint,
                );
            }
        }
        canvas.restore();
        bitmap.force_opaque();
        Ok(())
    }

    pub(crate) fn measure_text(
        &self,
        text: &str,
        font_size: f32,
        font_family: &str,
        direction: &str,
    ) -> Result<[f32; 8], String> {
        let family = CanvasFontFace::parse(font_family)?;
        let font = canvas_font(family, font_size)?;
        let shaped = shape_text(text, font_size, direction, family)?;
        let (_, metrics) = font.metrics();
        Ok([
            shaped.advance,
            -shaped.bounds.left,
            shaped.bounds.right,
            -shaped.bounds.top,
            shaped.bounds.bottom,
            -metrics.ascent,
            metrics.descent,
            metrics.leading,
        ])
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn clear_rect(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        x: f64,
        y: f64,
        rect_width: f64,
        rect_height: f64,
        transform: [f32; 6],
    ) -> Result<(), String> {
        if !all_finite(&[x, y, rect_width, rect_height]) {
            return Ok(());
        }
        let Some(surface) = self.bitmap(id, width, height)?.surface.as_mut() else {
            return Ok(());
        };
        let mut paint = Paint::new(Color4f::new(0.0, 0.0, 0.0, 0.0), None);
        paint.set_anti_alias(true).set_blend_mode(BlendMode::Clear);
        let canvas = surface.canvas();
        canvas.save();
        canvas.concat(&Matrix::from_affine(&transform));
        canvas.draw_rect(
            Rect::from_xywh(x as f32, y as f32, rect_width as f32, rect_height as f32),
            &paint,
        );
        canvas.restore();
        self.bitmap(id, width, height)?.force_opaque();
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn read_rgba(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        source_x: i32,
        source_y: i32,
        read_width: u32,
        read_height: u32,
    ) -> Result<Vec<u8>, String> {
        self.read_image_data(
            id,
            width,
            height,
            source_x,
            source_y,
            read_width,
            read_height,
            CanvasColorSpace::Srgb,
            CanvasColorType::Unorm8,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn read_image_data(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        source_x: i32,
        source_y: i32,
        read_width: u32,
        read_height: u32,
        color_space: CanvasColorSpace,
        color_type: CanvasColorType,
    ) -> Result<Vec<u8>, String> {
        let byte_len = pixel_byte_len_for(read_width, read_height, color_type)?;
        let mut pixels = vec![0; byte_len];
        let bitmap = self.bitmap(id, width, height)?;
        if !bitmap.origin_clean {
            return Err("Canvas bitmap is not origin-clean".to_owned());
        }
        let Some(surface) = bitmap.surface.as_mut() else {
            return Ok(pixels);
        };
        let info = image_data_info(read_width, read_height, color_space, color_type)?;
        let row_bytes = usize::try_from(read_width)
            .ok()
            .and_then(|width| width.checked_mul(color_type.bytes_per_pixel()))
            .ok_or_else(|| "canvas image data is too large".to_owned())?;
        surface.read_pixels(&info, &mut pixels, row_bytes, (source_x, source_y));
        Ok(pixels)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn write_rgba(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        destination_x: i32,
        destination_y: i32,
        image_width: u32,
        image_height: u32,
        pixels: &[u8],
    ) -> Result<(), String> {
        self.write_image_data(
            id,
            width,
            height,
            destination_x,
            destination_y,
            image_width,
            image_height,
            pixels,
            CanvasColorSpace::Srgb,
            CanvasColorType::Unorm8,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn write_image_data(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        destination_x: i32,
        destination_y: i32,
        image_width: u32,
        image_height: u32,
        pixels: &[u8],
        color_space: CanvasColorSpace,
        color_type: CanvasColorType,
    ) -> Result<(), String> {
        if pixels.len() != pixel_byte_len_for(image_width, image_height, color_type)? {
            return Err("ImageData byte length does not match its dimensions".to_owned());
        }
        let Some(surface) = self.bitmap(id, width, height)?.surface.as_mut() else {
            return Ok(());
        };
        let row_bytes = usize::try_from(image_width)
            .ok()
            .and_then(|width| width.checked_mul(color_type.bytes_per_pixel()))
            .ok_or_else(|| "canvas image data is too large".to_owned())?;
        let _ = surface.canvas().write_pixels(
            &image_data_info(image_width, image_height, color_space, color_type)?,
            pixels,
            row_bytes,
            (destination_x, destination_y),
        );
        self.bitmap(id, width, height)?.force_opaque();
        Ok(())
    }

    pub(crate) fn encode(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        mime_type: &str,
        quality: u8,
    ) -> Result<Option<(String, Vec<u8>)>, String> {
        let bitmap = self.bitmap(id, width, height)?;
        if !bitmap.origin_clean {
            return Err("Canvas bitmap is not origin-clean".to_owned());
        }
        let Some(surface) = bitmap.surface.as_mut() else {
            return Ok(None);
        };
        let (mime_type, format) = match mime_type {
            "image/jpeg" => ("image/jpeg", EncodedImageFormat::JPEG),
            "image/webp" => ("image/webp", EncodedImageFormat::WEBP),
            _ => ("image/png", EncodedImageFormat::PNG),
        };
        let image = surface.image_snapshot();
        #[allow(deprecated)]
        let data = image
            .encode_to_data_with_quality(format, u32::from(quality))
            .ok_or_else(|| format!("Skia could not encode {mime_type}"))?;
        Ok(Some((mime_type.to_owned(), data.as_bytes().to_vec())))
    }

    fn bitmap(&mut self, id: NodeId, width: u32, height: u32) -> Result<&mut CanvasBitmap, String> {
        let bitmap = match self.entries.entry(id) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(CanvasBitmap::new(width, height)?),
        };
        bitmap.resize(width, height)?;
        Ok(bitmap)
    }

    fn insert_gradient(&mut self, kind: GradientKind, transform: [f32; 6]) -> Result<u64, String> {
        self.next_gradient = self
            .next_gradient
            .checked_add(1)
            .ok_or("Canvas gradient id overflow")?;
        let id = self.next_gradient;
        self.gradients.insert(
            id,
            CanvasGradientData {
                kind,
                transform,
                stops: Vec::new(),
            },
        );
        Ok(id)
    }

    fn insert_path(&mut self, path: PathBuilder) -> Result<u64, String> {
        self.next_path = self.next_path.checked_add(1).ok_or("Path2D id overflow")?;
        let id = self.next_path;
        self.paths.insert(id, path);
        Ok(id)
    }

    fn paint_origin_clean(&self, style: CanvasPaintStyle) -> Result<bool, String> {
        match style {
            CanvasPaintStyle::Pattern { id, .. } => self
                .patterns
                .get(&id)
                .map(|pattern| pattern.origin_clean)
                .ok_or_else(|| "unknown CanvasPattern".to_owned()),
            CanvasPaintStyle::Color(_) | CanvasPaintStyle::Gradient { .. } => Ok(true),
        }
    }

    fn paint_for(
        &self,
        style: CanvasPaintStyle,
        composite: &str,
        drawing_transform: [f32; 6],
        effects: &CanvasDrawEffects,
    ) -> Result<Paint, String> {
        let mut paint = Paint::default();
        paint
            .set_anti_alias(true)
            .set_blend_mode(blend_mode(composite)?);
        match style {
            CanvasPaintStyle::Color(rgba) => {
                paint.set_color4f(Color4f::new(rgba[0], rgba[1], rgba[2], rgba[3]), None);
            }
            CanvasPaintStyle::Gradient { id, alpha } => {
                let gradient = self.gradients.get(&id).ok_or("unknown CanvasGradient")?;
                if gradient.stops.is_empty() {
                    paint.set_color4f(Color4f::new(0.0, 0.0, 0.0, 0.0), None);
                    return Ok(paint);
                }
                let positions = gradient
                    .stops
                    .iter()
                    .map(|(offset, _)| *offset)
                    .collect::<Vec<_>>();
                let colors = gradient
                    .stops
                    .iter()
                    .map(|(_, color)| Color4f::new(color.r, color.g, color.b, color.a * alpha))
                    .collect::<Vec<_>>();
                let drawing = Matrix::from_affine(&drawing_transform);
                let local = drawing
                    .invert()
                    .map(|inverse| {
                        Matrix::concat(&inverse, &Matrix::from_affine(&gradient.transform))
                    })
                    .ok_or("Canvas drawing transform is not invertible")?;
                let shader_gradient = Gradient::new(
                    GradientColors::new(
                        colors.as_slice(),
                        Some(positions.as_slice()),
                        TileMode::Clamp,
                        ColorSpace::new_srgb(),
                    ),
                    Interpolation::default(),
                );
                let shader = match gradient.kind {
                    GradientKind::Linear(coordinates) => gradient_shaders::linear_gradient(
                        (
                            (coordinates[0], coordinates[1]),
                            (coordinates[2], coordinates[3]),
                        ),
                        &shader_gradient,
                        Some(&local),
                    ),
                    GradientKind::Radial(coordinates) => {
                        gradient_shaders::two_point_conical_gradient(
                            ((coordinates[0], coordinates[1]), coordinates[2]),
                            ((coordinates[3], coordinates[4]), coordinates[5]),
                            &shader_gradient,
                            Some(&local),
                        )
                    }
                    GradientKind::Conic(coordinates) => gradient_shaders::sweep_gradient(
                        (coordinates[1], coordinates[2]),
                        (
                            coordinates[0].to_degrees(),
                            coordinates[0].to_degrees() + 360.0,
                        ),
                        &shader_gradient,
                        Some(&local),
                    ),
                }
                .ok_or("Skia could not create the Canvas gradient shader")?;
                paint.set_shader(shader);
            }
            CanvasPaintStyle::Pattern { id, alpha } => {
                let pattern = self.patterns.get(&id).ok_or("unknown CanvasPattern")?;
                let matrix = Matrix::from_affine(&pattern.transform);
                let shader = pattern
                    .image
                    .to_shader(
                        Some(pattern.tile_modes),
                        SamplingOptions::default(),
                        Some(&matrix),
                    )
                    .ok_or("Skia could not create the Canvas pattern shader")?;
                paint.set_shader(shader).set_alpha_f(alpha);
            }
        }
        apply_effects(&mut paint, effects)?;
        Ok(paint)
    }
}

fn apply_stroke(paint: &mut Paint, stroke: &CanvasStrokeStyle) -> Result<(), String> {
    let cap = match stroke.cap.as_str() {
        "butt" => paint::Cap::Butt,
        "round" => paint::Cap::Round,
        "square" => paint::Cap::Square,
        _ => return Err("invalid Canvas lineCap".to_owned()),
    };
    let join = match stroke.join.as_str() {
        "miter" => paint::Join::Miter,
        "round" => paint::Join::Round,
        "bevel" => paint::Join::Bevel,
        _ => return Err("invalid Canvas lineJoin".to_owned()),
    };
    paint
        .set_style(paint::Style::Stroke)
        .set_stroke_width(stroke.width)
        .set_stroke_cap(cap)
        .set_stroke_join(join)
        .set_stroke_miter(stroke.miter_limit);
    if !stroke.dash.is_empty() {
        paint.set_path_effect(dash_path_effect::new(&stroke.dash, stroke.dash_offset));
    }
    Ok(())
}

fn apply_effects(paint: &mut Paint, effects: &CanvasDrawEffects) -> Result<(), String> {
    let mut filters = Vec::with_capacity(effects.filters.len());
    for operation in &effects.filters {
        let filter = match operation {
            CanvasFilterOperation::Blur {
                sigma_x,
                sigma_y,
                input,
            } => image_filters::blur(
                (*sigma_x, *sigma_y),
                Some(TileMode::Decal),
                canvas_filter_input(&filters, *input),
                None,
            ),
            CanvasFilterOperation::Offset { x, y, input } => {
                image_filters::offset((*x, *y), canvas_filter_input(&filters, *input), None)
            }
            CanvasFilterOperation::ColorMatrix { matrix, input } => image_filters::color_filter(
                color_filters::matrix_row_major(matrix, None),
                canvas_filter_input(&filters, *input),
                None,
            ),
            CanvasFilterOperation::ComponentTransfer { tables, input } => {
                image_filters::color_filter(
                    color_filters::table_argb(
                        Some(&tables[3]),
                        Some(&tables[0]),
                        Some(&tables[1]),
                        Some(&tables[2]),
                    )
                    .ok_or("Skia could not create the Canvas component-transfer table")?,
                    canvas_filter_input(&filters, *input),
                    None,
                )
            }
            CanvasFilterOperation::Morphology {
                dilate,
                radius_x,
                radius_y,
                input,
            } => {
                let input = canvas_filter_input(&filters, *input);
                if *dilate {
                    image_filters::dilate((*radius_x, *radius_y), input, None)
                } else {
                    image_filters::erode((*radius_x, *radius_y), input, None)
                }
            }
            CanvasFilterOperation::Flood { color } => image_filters::shader(
                skia_shaders::color(Color::from_argb(
                    canvas_color_byte(color[3]),
                    canvas_color_byte(color[0]),
                    canvas_color_byte(color[1]),
                    canvas_color_byte(color[2]),
                )),
                None,
            ),
            CanvasFilterOperation::ConvolveMatrix {
                width,
                height,
                kernel,
                gain,
                bias,
                target_x,
                target_y,
                edge_mode,
                convolve_alpha,
                input,
            } => image_filters::matrix_convolution(
                ISize::new(*width, *height),
                kernel,
                *gain,
                *bias * 255.0,
                IPoint::new(*target_x, *target_y),
                svg_edge_mode(edge_mode)?,
                *convolve_alpha,
                canvas_filter_input(&filters, *input),
                None,
            ),
            CanvasFilterOperation::DisplacementMap {
                scale,
                x_channel,
                y_channel,
                input,
                input2,
            } => image_filters::displacement_map(
                (svg_color_channel(x_channel)?, svg_color_channel(y_channel)?),
                *scale,
                canvas_filter_input(&filters, *input2),
                canvas_filter_input(&filters, *input),
                None,
            ),
            CanvasFilterOperation::Lighting {
                specular,
                color,
                surface_scale,
                constant,
                exponent,
                light,
                input,
            } => {
                let color = Color::from_rgb(
                    canvas_color_byte(color[0]),
                    canvas_color_byte(color[1]),
                    canvas_color_byte(color[2]),
                );
                let input = canvas_filter_input(&filters, *input);
                match (specular, light) {
                    (false, CanvasLightSource::Distant { azimuth, elevation }) => {
                        image_filters::distant_lit_diffuse(
                            svg_distant_light(*azimuth, *elevation),
                            color,
                            *surface_scale,
                            *constant,
                            input,
                            None,
                        )
                    }
                    (false, CanvasLightSource::Point { position }) => {
                        image_filters::point_lit_diffuse(
                            Point3::new(position[0], position[1], position[2]),
                            color,
                            *surface_scale,
                            *constant,
                            input,
                            None,
                        )
                    }
                    (
                        false,
                        CanvasLightSource::Spot {
                            position,
                            target,
                            falloff_exponent,
                            cutoff_angle,
                        },
                    ) => image_filters::spot_lit_diffuse(
                        Point3::new(position[0], position[1], position[2]),
                        Point3::new(target[0], target[1], target[2]),
                        *falloff_exponent,
                        *cutoff_angle,
                        color,
                        *surface_scale,
                        *constant,
                        input,
                        None,
                    ),
                    (true, CanvasLightSource::Distant { azimuth, elevation }) => {
                        image_filters::distant_lit_specular(
                            svg_distant_light(*azimuth, *elevation),
                            color,
                            *surface_scale,
                            *constant,
                            *exponent,
                            input,
                            None,
                        )
                    }
                    (true, CanvasLightSource::Point { position }) => {
                        image_filters::point_lit_specular(
                            Point3::new(position[0], position[1], position[2]),
                            color,
                            *surface_scale,
                            *constant,
                            *exponent,
                            input,
                            None,
                        )
                    }
                    (
                        true,
                        CanvasLightSource::Spot {
                            position,
                            target,
                            falloff_exponent,
                            cutoff_angle,
                        },
                    ) => image_filters::spot_lit_specular(
                        Point3::new(position[0], position[1], position[2]),
                        Point3::new(target[0], target[1], target[2]),
                        *falloff_exponent,
                        *cutoff_angle,
                        color,
                        *surface_scale,
                        *constant,
                        *exponent,
                        input,
                        None,
                    ),
                }
            }
            CanvasFilterOperation::DropShadow { shadow, input } => image_filters::drop_shadow(
                (shadow.offset_x, shadow.offset_y),
                (shadow.blur * 0.5, shadow.blur * 0.5),
                Color4f::new(
                    shadow.color[0],
                    shadow.color[1],
                    shadow.color[2],
                    shadow.color[3],
                ),
                None::<ColorSpace>,
                canvas_filter_input(&filters, *input),
                None,
            ),
            CanvasFilterOperation::Blend {
                mode,
                input,
                input2,
            } => image_filters::blend(
                svg_blend_mode(mode)?,
                canvas_filter_input(&filters, *input2),
                canvas_filter_input(&filters, *input),
                None,
            ),
            CanvasFilterOperation::Composite {
                operator,
                coefficients,
                input,
                input2,
            } => {
                let background = canvas_filter_input(&filters, *input2);
                let foreground = canvas_filter_input(&filters, *input);
                if operator == "arithmetic" {
                    image_filters::arithmetic(
                        coefficients[0],
                        coefficients[1],
                        coefficients[2],
                        coefficients[3],
                        true,
                        background,
                        foreground,
                        None,
                    )
                } else {
                    image_filters::blend(
                        svg_composite_mode(operator)?,
                        background,
                        foreground,
                        None,
                    )
                }
            }
            CanvasFilterOperation::Merge(inputs) => image_filters::merge(
                inputs
                    .iter()
                    .map(|input| canvas_filter_input(&filters, *input)),
                None,
            ),
        };
        filters.push(filter.ok_or_else(|| "Skia could not create the Canvas filter".to_owned())?);
    }
    let mut filter = filters.last().cloned();
    let shadow = &effects.shadow;
    if shadow.color[3] > 0.0 {
        let sigma = shadow.blur * 0.5;
        filter = image_filters::drop_shadow(
            (shadow.offset_x, shadow.offset_y),
            (sigma, sigma),
            Color4f::new(
                shadow.color[0],
                shadow.color[1],
                shadow.color[2],
                shadow.color[3],
            ),
            None::<ColorSpace>,
            filter.take(),
            None,
        );
        if filter.is_none() {
            return Err("Skia could not create the Canvas shadow filter".to_owned());
        }
    }
    if let Some(filter) = filter {
        paint.set_image_filter(filter);
    }
    Ok(())
}

fn canvas_filter_input(
    filters: &[skia_safe::ImageFilter],
    input: CanvasFilterInput,
) -> Option<skia_safe::ImageFilter> {
    match input {
        CanvasFilterInput::SourceGraphic => None,
        CanvasFilterInput::Operation(index) => Some(filters[index].clone()),
    }
}

fn svg_blend_mode(value: &str) -> Result<BlendMode, String> {
    if value == "normal" {
        Ok(BlendMode::SrcOver)
    } else {
        blend_mode(value)
    }
}

fn svg_composite_mode(value: &str) -> Result<BlendMode, String> {
    match value {
        "over" => Ok(BlendMode::SrcOver),
        "in" => Ok(BlendMode::SrcIn),
        "out" => Ok(BlendMode::SrcOut),
        "atop" => Ok(BlendMode::SrcATop),
        "xor" => Ok(BlendMode::Xor),
        _ => Err("invalid SVG composite operator".to_owned()),
    }
}

fn svg_edge_mode(value: &str) -> Result<TileMode, String> {
    match value {
        "duplicate" => Ok(TileMode::Clamp),
        "wrap" => Ok(TileMode::Repeat),
        "none" => Ok(TileMode::Decal),
        _ => Err("invalid SVG convolution edge mode".to_owned()),
    }
}

fn svg_color_channel(value: &str) -> Result<ColorChannel, String> {
    match value {
        "R" => Ok(ColorChannel::R),
        "G" => Ok(ColorChannel::G),
        "B" => Ok(ColorChannel::B),
        "A" => Ok(ColorChannel::A),
        _ => Err("invalid SVG displacement-map channel".to_owned()),
    }
}

fn canvas_color_byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn svg_distant_light(azimuth: f32, elevation: f32) -> Point3 {
    let azimuth = azimuth.to_radians();
    let elevation = elevation.to_radians();
    Point3::new(
        azimuth.cos() * elevation.cos(),
        azimuth.sin() * elevation.cos(),
        elevation.sin(),
    )
}

fn new_surface(width: u32, height: u32) -> Result<Option<Surface>, String> {
    new_canvas_surface(
        width,
        height,
        true,
        CanvasColorSpace::Srgb,
        CanvasColorType::Unorm8,
    )
}

fn new_canvas_surface(
    width: u32,
    height: u32,
    alpha: bool,
    color_space: CanvasColorSpace,
    color_type: CanvasColorType,
) -> Result<Option<Surface>, String> {
    if width == 0 || height == 0 {
        return Ok(None);
    }
    let info = ImageInfo::new(
        (width as i32, height as i32),
        color_type.skia(),
        if alpha {
            AlphaType::Premul
        } else {
            AlphaType::Opaque
        },
        color_space.skia()?,
    );
    let mut surface = surfaces::raster(&info, None, None)
        .ok_or_else(|| "Skia could not allocate the Canvas backing store".to_owned())?;
    surface
        .canvas()
        .clear(Color4f::new(0.0, 0.0, 0.0, if alpha { 0.0 } else { 1.0 }));
    Ok(Some(surface))
}

fn validate_dimensions(width: u32, height: u32) -> Result<(), String> {
    if width > MAX_CANVAS_DIMENSION || height > MAX_CANVAS_DIMENSION {
        return Err("Canvas dimensions exceed the supported maximum".to_owned());
    }
    let pixels = u64::from(width) * u64::from(height);
    if pixels > MAX_CANVAS_PIXELS {
        return Err("Canvas backing store exceeds the supported maximum".to_owned());
    }
    Ok(())
}

fn pixel_byte_len(width: u32, height: u32) -> Result<usize, String> {
    pixel_byte_len_for(width, height, CanvasColorType::Unorm8)
}

fn pixel_byte_len_for(
    width: u32,
    height: u32,
    color_type: CanvasColorType,
) -> Result<usize, String> {
    validate_dimensions(width, height)?;
    usize::try_from(u64::from(width) * u64::from(height) * color_type.bytes_per_pixel() as u64)
        .map_err(|_| "canvas image data is too large".to_owned())
}

fn rgba_info(width: u32, height: u32) -> ImageInfo {
    ImageInfo::new(
        (width as i32, height as i32),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        ColorSpace::new_srgb(),
    )
}

fn image_data_info(
    width: u32,
    height: u32,
    color_space: CanvasColorSpace,
    color_type: CanvasColorType,
) -> Result<ImageInfo, String> {
    Ok(ImageInfo::new(
        (width as i32, height as i32),
        color_type.skia(),
        AlphaType::Unpremul,
        color_space.skia()?,
    ))
}

const fn identity_matrix() -> [f32; 6] {
    [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
}

fn append_path_points(
    path: &mut PathBuilder,
    operation: &str,
    points: &[f64],
    transform: [f32; 6],
) -> Result<(), String> {
    if !all_finite(points) {
        return Ok(());
    }
    let matrix = Matrix::from_affine(&transform);
    let mapped = points
        .chunks_exact(2)
        .map(|point| matrix.map_point((point[0] as f32, point[1] as f32)))
        .collect::<Vec<_>>();
    match (operation, mapped.as_slice()) {
        ("move", [point]) => path.move_to(*point),
        ("line", [point]) => path.line_to(*point),
        ("quadratic", [control, end]) => path.quad_to(*control, *end),
        ("bezier", [first, second, end]) => path.cubic_to(*first, *second, *end),
        _ => return Err("invalid Canvas path operation".to_owned()),
    };
    Ok(())
}

fn append_path_arc_to(
    path: &mut PathBuilder,
    first: [f64; 2],
    second: [f64; 2],
    radius: f64,
    transform: [f32; 6],
) -> Result<(), String> {
    if !all_finite(&[first[0], first[1], second[0], second[1], radius]) {
        return Ok(());
    }
    let matrix = Matrix::from_affine(&transform);
    let mut addition = PathBuilder::new();
    if let Some(current) = path.get_last_pt() {
        let Some(inverse) = matrix.invert() else {
            return Ok(());
        };
        addition.move_to(inverse.map_point(current));
        addition.arc_to_tangent(
            (first[0] as f32, first[1] as f32),
            (second[0] as f32, second[1] as f32),
            radius as f32,
        );
    } else {
        addition.move_to((first[0] as f32, first[1] as f32));
    }
    path.add_path_with_transform(&addition.snapshot(), &matrix, path::AddPathMode::Extend);
    Ok(())
}

fn append_path_rect(
    path: &mut PathBuilder,
    rect: [f64; 4],
    transform: [f32; 6],
) -> Result<(), String> {
    if !all_finite(&rect) {
        return Ok(());
    }
    let mut addition = PathBuilder::new();
    addition.add_rect(
        Rect::from_xywh(
            rect[0] as f32,
            rect[1] as f32,
            rect[2] as f32,
            rect[3] as f32,
        ),
        None,
        None,
    );
    path.add_path_with_transform(
        &addition.snapshot(),
        &Matrix::from_affine(&transform),
        path::AddPathMode::Append,
    );
    Ok(())
}

fn append_path_round_rect(
    path: &mut PathBuilder,
    rect: [f64; 4],
    radii: [f64; 8],
    transform: [f32; 6],
) -> Result<(), String> {
    if !all_finite(&rect) || !all_finite(&radii) {
        return Ok(());
    }
    let rect = Rect::from_xywh(
        rect[0] as f32,
        rect[1] as f32,
        rect[2] as f32,
        rect[3] as f32,
    );
    let radii = [
        Vector::new(radii[0] as f32, radii[1] as f32),
        Vector::new(radii[2] as f32, radii[3] as f32),
        Vector::new(radii[4] as f32, radii[5] as f32),
        Vector::new(radii[6] as f32, radii[7] as f32),
    ];
    let mut addition = PathBuilder::new();
    addition.add_rrect(RRect::new_rect_radii(rect, &radii), None, None);
    path.add_path_with_transform(
        &addition.snapshot(),
        &Matrix::from_affine(&transform),
        path::AddPathMode::Append,
    );
    Ok(())
}

fn append_path_arc(
    path: &mut PathBuilder,
    center: [f64; 2],
    radius: f64,
    start: f64,
    sweep: f64,
    transform: [f32; 6],
) -> Result<(), String> {
    if !all_finite(&[center[0], center[1], radius, start, sweep]) {
        return Ok(());
    }
    let mut addition = PathBuilder::new();
    addition.add_arc(
        Rect::from_xywh(
            (center[0] - radius) as f32,
            (center[1] - radius) as f32,
            (radius * 2.0) as f32,
            (radius * 2.0) as f32,
        ),
        start.to_degrees() as f32,
        sweep.to_degrees() as f32,
    );
    path.add_path_with_transform(
        &addition.snapshot(),
        &Matrix::from_affine(&transform),
        path::AddPathMode::Extend,
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn append_path_ellipse(
    path: &mut PathBuilder,
    center: [f64; 2],
    radii: [f64; 2],
    rotation: f64,
    start: f64,
    sweep: f64,
    transform: [f32; 6],
) -> Result<(), String> {
    if !all_finite(&[
        center[0], center[1], radii[0], radii[1], rotation, start, sweep,
    ]) {
        return Ok(());
    }
    let mut addition = PathBuilder::new();
    addition.add_arc(
        Rect::from_xywh(
            (center[0] - radii[0]) as f32,
            (center[1] - radii[1]) as f32,
            (radii[0] * 2.0) as f32,
            (radii[1] * 2.0) as f32,
        ),
        start.to_degrees() as f32,
        sweep.to_degrees() as f32,
    );
    let rotation = Matrix::rotate_deg_pivot(
        rotation.to_degrees() as f32,
        (center[0] as f32, center[1] as f32),
    );
    let transform = Matrix::concat(&Matrix::from_affine(&transform), &rotation);
    path.add_path_with_transform(&addition.snapshot(), &transform, path::AddPathMode::Extend);
    Ok(())
}

fn all_finite(values: &[f64]) -> bool {
    values.iter().all(|value| value.is_finite())
}

impl CanvasFontFace {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "proportional" => Ok(Self::Proportional),
            "monospace" => Ok(Self::Monospace),
            "emoji" => Ok(Self::Emoji),
            _ => Err("invalid Canvas font family".to_owned()),
        }
    }

    fn data(self) -> &'static [u8] {
        match self {
            Self::Proportional | Self::Monospace => WENQUANYI_FONT,
            Self::Emoji => NOTO_EMOJI_FONT,
        }
    }

    fn index(self) -> u32 {
        match self {
            Self::Proportional => 0,
            Self::Monospace => 1,
            Self::Emoji => 0,
        }
    }

    fn typeface(self) -> &'static OnceLock<Option<Typeface>> {
        match self {
            Self::Proportional => &PROPORTIONAL_TYPEFACE,
            Self::Monospace => &MONOSPACE_TYPEFACE,
            Self::Emoji => &EMOJI_TYPEFACE,
        }
    }
}

fn canvas_font(face: CanvasFontFace, size: f32) -> Result<Font, String> {
    if !size.is_finite() || size <= 0.0 {
        return Err("Canvas font size must be positive and finite".to_owned());
    }
    let typeface = face
        .typeface()
        .get_or_init(|| {
            FontMgr::new()
                .new_from_data(face.data(), 0)
                .and_then(|typeface| {
                    if face == CanvasFontFace::Monospace {
                        let mut arguments = FontArguments::new();
                        arguments.set_collection_index(1);
                        typeface.clone_with_arguments(&arguments)
                    } else {
                        Some(typeface)
                    }
                })
        })
        .clone()
        .ok_or("Skia could not load the bundled Canvas font")?;
    let mut font = Font::new(typeface, size);
    font.set_subpixel(true).set_linear_metrics(true);
    Ok(font)
}

fn fallback_face(
    grapheme: &str,
    primary: CanvasFontFace,
    previous: CanvasFontFace,
    primary_face: &Face<'_>,
    proportional_face: &Face<'_>,
    emoji_face: &Face<'_>,
) -> CanvasFontFace {
    let mut has_visible_character = false;
    let mut primary_supports_grapheme = true;
    let mut proportional_supports_grapheme = true;
    let mut emoji_supports_grapheme = true;
    let mut requests_emoji_presentation = false;
    for character in grapheme.chars() {
        if matches!(character, '\u{200d}' | '\u{fe0e}' | '\u{fe0f}') {
            requests_emoji_presentation |= matches!(character, '\u{200d}' | '\u{fe0f}');
            continue;
        }
        has_visible_character = true;
        primary_supports_grapheme &= primary_face.glyph_index(character).is_some();
        proportional_supports_grapheme &= proportional_face.glyph_index(character).is_some();
        emoji_supports_grapheme &= emoji_face.glyph_index(character).is_some();
    }
    if !has_visible_character {
        return previous;
    }
    if emoji_supports_grapheme && (primary == CanvasFontFace::Emoji || requests_emoji_presentation)
    {
        CanvasFontFace::Emoji
    } else if primary_supports_grapheme {
        primary
    } else if emoji_supports_grapheme {
        CanvasFontFace::Emoji
    } else if proportional_supports_grapheme {
        CanvasFontFace::Proportional
    } else {
        primary
    }
}

fn bidi_text_runs<'a>(text: &'a str, direction: &str) -> Result<Vec<(&'a str, Direction)>, String> {
    let paragraph_level = match direction {
        "ltr" => Level::ltr(),
        "rtl" => Level::rtl(),
        _ => return Err("invalid Canvas text direction".to_owned()),
    };
    let bidi = BidiInfo::new(text, Some(paragraph_level));
    let mut result = Vec::new();
    for paragraph in &bidi.paragraphs {
        let (levels, runs) = bidi.visual_runs(paragraph, paragraph.range.clone());
        for run in runs {
            let run_direction = if levels[run.start].is_rtl() {
                Direction::RightToLeft
            } else {
                Direction::LeftToRight
            };
            result.push((&text[run], run_direction));
        }
    }
    Ok(result)
}

fn shape_text(
    text: &str,
    size: f32,
    direction: &str,
    primary: CanvasFontFace,
) -> Result<ShapedText, String> {
    let bidi_runs = bidi_text_runs(text, direction)?;
    let mut source_runs = Vec::<(CanvasFontFace, String, Direction)>::new();
    let primary_face = Face::from_slice(primary.data(), primary.index())
        .ok_or("Rustybuzz could not load the selected bundled Canvas font")?;
    let proportional_face = Face::from_slice(
        CanvasFontFace::Proportional.data(),
        CanvasFontFace::Proportional.index(),
    )
    .ok_or("Rustybuzz could not load the proportional bundled Canvas font")?;
    let emoji_face = Face::from_slice(CanvasFontFace::Emoji.data(), CanvasFontFace::Emoji.index())
        .ok_or("Rustybuzz could not load the emoji bundled Canvas font")?;
    for (bidi_text, run_direction) in bidi_runs {
        let mut font_runs = Vec::<(CanvasFontFace, String, Direction)>::new();
        let mut previous = primary;
        for grapheme in bidi_text.graphemes(true) {
            let face = fallback_face(
                grapheme,
                primary,
                previous,
                &primary_face,
                &proportional_face,
                &emoji_face,
            );
            if let Some((run_face, run, _)) = font_runs.last_mut()
                && *run_face == face
            {
                run.push_str(grapheme);
            } else {
                font_runs.push((face, grapheme.to_owned(), run_direction));
            }
            previous = face;
        }
        if run_direction == Direction::RightToLeft {
            font_runs.reverse();
        }
        source_runs.extend(font_runs);
    }

    let mut runs = Vec::with_capacity(source_runs.len());
    let mut pen = Point::new(0.0, 0.0);
    let mut bounds = None::<Rect>;
    for (face_kind, run, run_direction) in source_runs {
        let face = Face::from_slice(face_kind.data(), face_kind.index())
            .ok_or("Rustybuzz could not load a bundled Canvas font")?;
        let font = canvas_font(face_kind, size)?;
        let scale = size / face.units_per_em() as f32;
        let mut buffer = UnicodeBuffer::new();
        buffer.push_str(&run);
        buffer.guess_segment_properties();
        buffer.set_direction(run_direction);
        let output = rustybuzz::shape(&face, &[], buffer);
        let glyphs = output
            .glyph_infos()
            .iter()
            .map(|info| {
                GlyphId::try_from(info.glyph_id)
                    .map_err(|_| "Canvas font returned an invalid glyph identifier".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut positions = Vec::with_capacity(glyphs.len());
        for position in output.glyph_positions() {
            positions.push(Point::new(
                pen.x + position.x_offset as f32 * scale,
                pen.y - position.y_offset as f32 * scale,
            ));
            pen.x += position.x_advance as f32 * scale;
            pen.y -= position.y_advance as f32 * scale;
        }

        let mut glyph_bounds = vec![Rect::new_empty(); glyphs.len()];
        font.get_bounds(&glyphs, &mut glyph_bounds, None);
        for (mut glyph_bounds, position) in glyph_bounds.into_iter().zip(&positions) {
            if glyph_bounds.is_empty() {
                continue;
            }
            glyph_bounds.offset((position.x, position.y));
            if let Some(bounds) = &mut bounds {
                bounds.join_non_empty_arg(glyph_bounds);
            } else {
                bounds = Some(glyph_bounds);
            }
        }
        runs.push(ShapedRun {
            font,
            glyphs,
            positions,
        });
    }

    Ok(ShapedText {
        runs,
        advance: pen.x.abs(),
        bounds: bounds.unwrap_or_else(Rect::new_empty),
    })
}

fn blend_mode(value: &str) -> Result<BlendMode, String> {
    Ok(match value {
        "source-over" => BlendMode::SrcOver,
        "source-in" => BlendMode::SrcIn,
        "source-out" => BlendMode::SrcOut,
        "source-atop" => BlendMode::SrcATop,
        "destination-over" => BlendMode::DstOver,
        "destination-in" => BlendMode::DstIn,
        "destination-out" => BlendMode::DstOut,
        "destination-atop" => BlendMode::DstATop,
        "lighter" => BlendMode::Plus,
        "copy" => BlendMode::Src,
        "xor" => BlendMode::Xor,
        "multiply" => BlendMode::Multiply,
        "screen" => BlendMode::Screen,
        "overlay" => BlendMode::Overlay,
        "darken" => BlendMode::Darken,
        "lighten" => BlendMode::Lighten,
        "color-dodge" => BlendMode::ColorDodge,
        "color-burn" => BlendMode::ColorBurn,
        "hard-light" => BlendMode::HardLight,
        "soft-light" => BlendMode::SoftLight,
        "difference" => BlendMode::Difference,
        "exclusion" => BlendMode::Exclusion,
        "hue" => BlendMode::Hue,
        "saturation" => BlendMode::Saturation,
        "color" => BlendMode::Color,
        "luminosity" => BlendMode::Luminosity,
        _ => return Err("invalid Canvas composite operation".to_owned()),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        CanvasColorSpace, CanvasColorType, CanvasDrawEffects, CanvasFontFace, CanvasPaintStyle,
        CanvasShadowStyle, CanvasStore, Direction, bidi_text_runs, shape_text,
    };

    #[test]
    fn raster_surface_round_trips_unpremultiplied_rgba() {
        let mut store = CanvasStore::default();
        store
            .acquire_2d(
                7,
                4,
                3,
                true,
                CanvasColorSpace::Srgb,
                CanvasColorType::Unorm8,
            )
            .unwrap();
        store
            .fill_rect(
                7,
                4,
                3,
                1.0,
                1.0,
                2.0,
                1.0,
                CanvasPaintStyle::Color([1.0, 0.0, 0.0, 0.5]),
                [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                &CanvasDrawEffects {
                    shadow: CanvasShadowStyle {
                        color: [0.0; 4],
                        blur: 0.0,
                        offset_x: 0.0,
                        offset_y: 0.0,
                    },
                    filters: Vec::new(),
                },
                "source-over",
            )
            .unwrap();

        let pixels = store.read_rgba(7, 4, 3, 0, 0, 4, 3).unwrap();
        assert_eq!(&pixels[20..][..4], &[255, 0, 0, 128]);
        assert_eq!(&pixels[..4], &[0, 0, 0, 0]);
    }

    #[test]
    fn display_p3_float16_surface_preserves_extended_range_pixels() {
        let mut store = CanvasStore::default();
        store
            .acquire_2d(
                9,
                1,
                1,
                true,
                CanvasColorSpace::DisplayP3,
                CanvasColorType::Float16,
            )
            .unwrap();
        let pixels = [0x3d00_u16, 0x3400, 0x3800, 0x3c00]
            .into_iter()
            .flat_map(u16::to_ne_bytes)
            .collect::<Vec<_>>();
        store
            .write_image_data(
                9,
                1,
                1,
                0,
                0,
                1,
                1,
                &pixels,
                CanvasColorSpace::DisplayP3,
                CanvasColorType::Float16,
            )
            .unwrap();

        let readback = store
            .read_image_data(
                9,
                1,
                1,
                0,
                0,
                1,
                1,
                CanvasColorSpace::DisplayP3,
                CanvasColorType::Float16,
            )
            .unwrap();
        assert_eq!(readback, pixels);
        assert_eq!(store.read_rgba(9, 1, 1, 0, 0, 1, 1).unwrap()[3], 255);
    }

    #[test]
    fn image_bitmap_snapshots_expose_unpremultiplied_pixels() {
        let mut store = CanvasStore::default();
        store
            .acquire_2d(
                8,
                2,
                1,
                true,
                CanvasColorSpace::Srgb,
                CanvasColorType::Unorm8,
            )
            .unwrap();
        let expected = [255, 10, 20, 128, 5, 40, 90, 255];
        store.write_rgba(8, 2, 1, 0, 0, 2, 1, &expected).unwrap();
        let (bitmap, _, _) = store.create_image_bitmap(8, 2, 1).unwrap();

        let (width, height, pixels, origin_clean) = store.image_bitmap_rgba(bitmap).unwrap();
        assert_eq!((width, height), (2, 1));
        assert_eq!(pixels, expected);
        assert!(origin_clean);
    }

    #[test]
    fn bundled_font_shaping_combines_marks_and_honors_direction() {
        let decomposed = shape_text("A\u{301}", 20.0, "ltr", CanvasFontFace::Proportional).unwrap();
        let composed = shape_text("\u{c1}", 20.0, "ltr", CanvasFontFace::Proportional).unwrap();
        assert_eq!(decomposed.runs[0].glyphs, composed.runs[0].glyphs);
        assert_eq!(decomposed.runs[0].positions, composed.runs[0].positions);
        assert!((decomposed.advance - composed.advance).abs() < 0.001);

        let ltr = shape_text("abc", 20.0, "ltr", CanvasFontFace::Proportional).unwrap();
        let rtl = shape_text("abc", 20.0, "rtl", CanvasFontFace::Proportional).unwrap();
        assert_eq!(ltr.runs[0].glyphs, rtl.runs[0].glyphs);
        assert!((ltr.advance - rtl.advance).abs() < 0.001);
    }

    #[test]
    fn unicode_bidi_splits_mixed_text_into_visual_directional_runs() {
        let ltr = bidi_text_runs("abc אבג", "ltr").unwrap();
        assert_eq!(
            ltr,
            [
                ("abc ", Direction::LeftToRight),
                ("אבג", Direction::RightToLeft),
            ]
        );

        let rtl = bidi_text_runs("abc אבג", "rtl").unwrap();
        assert_eq!(
            rtl,
            [
                (" אבג", Direction::RightToLeft),
                ("abc", Direction::LeftToRight),
            ]
        );
    }

    #[test]
    fn bundled_font_selection_uses_the_monospace_ttc_face() {
        let proportional = shape_text("ii", 20.0, "ltr", CanvasFontFace::Proportional).unwrap();
        let monospace_narrow = shape_text("ii", 20.0, "ltr", CanvasFontFace::Monospace).unwrap();
        let monospace_wide = shape_text("WW", 20.0, "ltr", CanvasFontFace::Monospace).unwrap();
        assert_ne!(proportional.advance, monospace_narrow.advance);
        assert!((monospace_narrow.advance - monospace_wide.advance).abs() < 0.001);
    }

    #[test]
    fn bundled_font_fallback_keeps_extended_graphemes_in_one_face() {
        // The proportional face contains U+2008 while the monospace face does
        // not. The combining acute accent is present in both, so scalar-based
        // fallback would incorrectly split this grapheme across two faces.
        let shaped = shape_text("\u{2008}\u{301}", 20.0, "ltr", CanvasFontFace::Monospace).unwrap();
        assert_eq!(shaped.runs.len(), 1);
    }

    #[test]
    fn bundled_font_fallback_keeps_emoji_sequences_in_the_emoji_face() {
        let shaped = shape_text("👩‍💻", 24.0, "ltr", CanvasFontFace::Proportional).unwrap();
        assert_eq!(shaped.runs.len(), 1);
        assert!(!shaped.runs[0].glyphs.is_empty());
    }
}

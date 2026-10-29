//! Skia-backed Canvas2D operations.
//!
//! The JavaScript canvas layer keeps the browser-facing state machine (styles,
//! transforms, paths, text metrics) and forwards the drawing commands here. A
//! canvas surface is identified by a small integer supplied by the JS layer.
use super::plugins::{BridgeValue, HostHandler, HostInvocation};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use skia_safe::{
    AlphaType, BlendMode, ClipOp, Color, ColorType, EncodedImageFormat, Font, FontMgr, FontStyle,
    ImageInfo, Matrix, Paint, PaintStyle, Path, PathBuilder, PathFillType, Rect, Surface,
};
use std::{cell::RefCell, collections::HashMap, sync::Arc};

struct CanvasState {
    surface: Surface,
    path: PathBuilder,
    alpha: bool,
}

thread_local! {
    static CANVASES: RefCell<HashMap<u64, CanvasState>> = RefCell::new(HashMap::new());
    static PATHS: RefCell<HashMap<u64, Path>> = RefCell::new(HashMap::new());
}

pub(crate) fn handler() -> HostHandler {
    Arc::new(|invocation: HostInvocation| {
        let result = dispatch(&invocation.arguments);
        match result {
            Ok(value) => invocation.completion.resolve(value),
            Err(error) => invocation.completion.reject(error),
        }
    })
}

pub(crate) fn dispatch(args: &[BridgeValue]) -> Result<BridgeValue, String> {
    let operation = string_arg(args, 0)?;
    let id = number_arg(args, 1)? as u64;
    match operation.as_str() {
        "canvas2dAcquire" => acquire(id, args),
        "canvas2dRelease" => {
            CANVASES.with(|canvases| canvases.borrow_mut().remove(&id));
            Ok(BridgeValue::Undefined)
        }
        "canvas2dReset" => reset(id, args),
        "canvas2dFillRect" => fill_rect(id, args),
        "canvas2dStrokeRect" => stroke_rect(id, args),
        "canvas2dClearRect" => clear_rect(id, args),
        "canvas2dBeginPath" => {
            with_canvas(id, |canvas| {
                canvas.path = PathBuilder::new();
                Ok(())
            })?;
            Ok(BridgeValue::Undefined)
        }
        "canvas2dClosePath" => {
            with_canvas(id, |canvas| {
                canvas.path.close();
                Ok(())
            })?;
            Ok(BridgeValue::Undefined)
        }
        "canvas2dPathPoint" => path_point(id, args),
        "canvas2dPathArc" => path_arc(id, args, 4),
        "canvas2dPathEllipse" => path_ellipse(id, args, 4),
        "canvas2dPathRect" => path_rect(id, args, 4),
        "canvas2dPathRoundRect" => path_round_rect(id, args, 4),
        "canvas2dPathArcTo" => path_arc_to(id, args, 4),
        "canvas2dDrawPath" => draw_path(id, args),
        "canvas2dClip" => clip(id, args),
        "canvas2dDrawText" => draw_text(id, args),
        "canvas2dPutImageData" => put_image_data(id, args),
        "canvas2dGetImageData" => get_image_data(id, args),
        "canvasEncode" => encode(id, args),
        "canvas2dSave" => {
            with_canvas(id, |canvas| {
                canvas.surface.canvas().save();
                Ok(())
            })?;
            Ok(BridgeValue::Undefined)
        }
        "canvas2dRestore" => {
            with_canvas(id, |canvas| {
                canvas.surface.canvas().restore();
                Ok(())
            })?;
            Ok(BridgeValue::Undefined)
        }
        _ => Err(format!("unknown canvas operation: {operation}")),
    }
}

fn acquire(id: u64, args: &[BridgeValue]) -> Result<BridgeValue, String> {
    let width = number_arg(args, 2)? as i32;
    let height = number_arg(args, 3)? as i32;
    let alpha = boolean_arg(args, 4)?;
    let mut surface = Surface::new_raster_n32_premul((width.max(1), height.max(1)))
        .ok_or_else(|| "unable to create canvas surface".to_string())?;
    surface.canvas().clear(if alpha {
        Color::TRANSPARENT
    } else {
        Color::BLACK
    });
    CANVASES.with(|canvases| {
        canvases.borrow_mut().insert(
            id,
            CanvasState {
                surface,
                path: PathBuilder::new(),
                alpha,
            },
        );
    });
    Ok(BridgeValue::Boolean(true))
}

fn reset(id: u64, args: &[BridgeValue]) -> Result<BridgeValue, String> {
    let width = number_arg(args, 2)? as i32;
    let height = number_arg(args, 3)? as i32;
    let alpha = boolean_arg(args, 4).unwrap_or(true);
    with_canvas(id, |canvas| {
        let mut surface = Surface::new_raster_n32_premul((width.max(1), height.max(1)))
            .ok_or_else(|| "unable to create canvas surface".to_string())?;
        surface.canvas().clear(if alpha {
            Color::TRANSPARENT
        } else {
            Color::BLACK
        });
        canvas.surface = surface;
        canvas.path = PathBuilder::new();
        canvas.alpha = alpha;
        Ok(())
    })?;
    Ok(BridgeValue::Undefined)
}

fn with_canvas<T>(
    id: u64,
    f: impl FnOnce(&mut CanvasState) -> Result<T, String>,
) -> Result<T, String> {
    CANVASES.with(|canvases| {
        let mut canvases = canvases.borrow_mut();
        let canvas = canvases
            .get_mut(&id)
            .ok_or_else(|| "unknown canvas surface".to_string())?;
        f(canvas)
    })
}

fn fill_rect(id: u64, args: &[BridgeValue]) -> Result<BridgeValue, String> {
    let x = number_arg(args, 4)? as f32;
    let y = number_arg(args, 5)? as f32;
    let width = number_arg(args, 6)? as f32;
    let height = number_arg(args, 7)? as f32;
    let style = style_arg(args, 8)?;
    let transform = matrix_from(args, 9)?;
    let composite = string_arg(args, 16).unwrap_or_else(|_| "source-over".into());
    with_canvas(id, |canvas| {
        let (x, width) = normalize(x, width);
        let (y, height) = normalize(y, height);
        let canvas2d = canvas.surface.canvas();
        canvas2d.save();
        canvas2d.concat(&transform);
        canvas2d.draw_rect(
            Rect::new(x, y, x + width, y + height),
            &paint(&style, &composite, PaintStyle::Fill),
        );
        canvas2d.restore();
        Ok(())
    })?;
    Ok(BridgeValue::Undefined)
}

fn stroke_rect(id: u64, args: &[BridgeValue]) -> Result<BridgeValue, String> {
    let x = number_arg(args, 4)? as f32;
    let y = number_arg(args, 5)? as f32;
    let width = number_arg(args, 6)? as f32;
    let height = number_arg(args, 7)? as f32;
    let style = style_arg(args, 8)?;
    let transform = matrix_from(args, 9)?;
    let stroke = stroke_arg(args, 15)?;
    let composite = string_arg(args, 17).unwrap_or_else(|_| "source-over".into());
    with_canvas(id, |canvas| {
        let (x, width) = normalize(x, width);
        let (y, height) = normalize(y, height);
        let canvas2d = canvas.surface.canvas();
        canvas2d.save();
        canvas2d.concat(&transform);
        let mut paint = paint(&style, &composite, PaintStyle::Stroke);
        paint.set_stroke_width(stroke.width);
        paint.set_stroke_cap(stroke.cap);
        paint.set_stroke_join(stroke.join);
        paint.set_stroke_miter(stroke.miter);
        canvas2d.draw_rect(Rect::new(x, y, x + width, y + height), &paint);
        canvas2d.restore();
        Ok(())
    })?;
    Ok(BridgeValue::Undefined)
}

fn clear_rect(id: u64, args: &[BridgeValue]) -> Result<BridgeValue, String> {
    let x = number_arg(args, 4)? as f32;
    let y = number_arg(args, 5)? as f32;
    let width = number_arg(args, 6)? as f32;
    let height = number_arg(args, 7)? as f32;
    let transform = matrix_from(args, 8)?;
    with_canvas(id, |canvas| {
        let (x, width) = normalize(x, width);
        let (y, height) = normalize(y, height);
        let alpha = canvas.alpha;
        let canvas2d = canvas.surface.canvas();
        canvas2d.save();
        canvas2d.concat(&transform);
        canvas2d.clip_rect(
            Rect::new(x, y, x + width, y + height),
            ClipOp::Intersect,
            true,
        );
        canvas2d.clear(if alpha {
            Color::TRANSPARENT
        } else {
            Color::BLACK
        });
        canvas2d.restore();
        Ok(())
    })?;
    Ok(BridgeValue::Undefined)
}

fn path_point(id: u64, args: &[BridgeValue]) -> Result<BridgeValue, String> {
    let kind = string_arg(args, 4)?;
    let transform = matrix_at_end(args, 8)?;
    let mut command = PathBuilder::new();
    match kind.as_str() {
        "move" => {
            command.move_to((number_arg(args, 5)? as f32, number_arg(args, 6)? as f32));
        }
        "line" => {
            command.line_to((number_arg(args, 5)? as f32, number_arg(args, 6)? as f32));
        }
        "quadratic" => {
            command.quad_to(
                (number_arg(args, 5)? as f32, number_arg(args, 6)? as f32),
                (number_arg(args, 7)? as f32, number_arg(args, 8)? as f32),
            );
        }
        "bezier" => {
            command.cubic_to(
                (number_arg(args, 5)? as f32, number_arg(args, 6)? as f32),
                (number_arg(args, 7)? as f32, number_arg(args, 8)? as f32),
                (number_arg(args, 9)? as f32, number_arg(args, 10)? as f32),
            );
        }
        _ => return Err(format!("unknown path point kind: {kind}")),
    }
    let command = command.detach();
    with_canvas(id, |canvas| {
        canvas
            .path
            .add_path_with_transform(&command, &transform, None);
        Ok(())
    })?;
    Ok(BridgeValue::Undefined)
}

fn path_arc(id: u64, args: &[BridgeValue], offset: usize) -> Result<BridgeValue, String> {
    let (x, y, radius, start, sweep) = (
        number_arg(args, offset)? as f32,
        number_arg(args, offset + 1)? as f32,
        number_arg(args, offset + 2)? as f32,
        number_arg(args, offset + 3)? as f32,
        number_arg(args, offset + 4)? as f32,
    );
    let transform = matrix_at_end(args, offset + 5)?;
    let mut command = PathBuilder::new();
    let oval = Rect::new(x - radius, y - radius, x + radius, y + radius);
    if sweep.abs() >= std::f32::consts::TAU - 1e-5 {
        command.add_oval(oval, None, 0);
    } else {
        command.add_arc(oval, start.to_degrees(), sweep.to_degrees());
    }
    let command = command.detach();
    with_canvas(id, |canvas| {
        canvas
            .path
            .add_path_with_transform(&command, &transform, None);
        Ok(())
    })?;
    Ok(BridgeValue::Undefined)
}

fn path_ellipse(id: u64, args: &[BridgeValue], offset: usize) -> Result<BridgeValue, String> {
    let (x, y, rx, ry, rotation, start, sweep) = (
        number_arg(args, offset)? as f32,
        number_arg(args, offset + 1)? as f32,
        number_arg(args, offset + 2)? as f32,
        number_arg(args, offset + 3)? as f32,
        number_arg(args, offset + 4)? as f32,
        number_arg(args, offset + 5)? as f32,
        number_arg(args, offset + 6)? as f32,
    );
    let transform = matrix_at_end(args, offset + 7)?;
    let mut command = PathBuilder::new();
    let oval = Rect::new(-rx, -ry, rx, ry);
    command.add_arc(oval, start.to_degrees(), sweep.to_degrees());
    let local = Matrix::new_all(
        rotation.cos(),
        -rotation.sin(),
        x,
        rotation.sin(),
        rotation.cos(),
        y,
        0.0,
        0.0,
        1.0,
    );
    let command = command.detach().with_transform(&local);
    with_canvas(id, |canvas| {
        canvas
            .path
            .add_path_with_transform(&command, &transform, None);
        Ok(())
    })?;
    Ok(BridgeValue::Undefined)
}

fn path_rect(id: u64, args: &[BridgeValue], offset: usize) -> Result<BridgeValue, String> {
    let x = number_arg(args, offset)? as f32;
    let y = number_arg(args, offset + 1)? as f32;
    let width = number_arg(args, offset + 2)? as f32;
    let height = number_arg(args, offset + 3)? as f32;
    let transform = matrix_at_end(args, offset + 4)?;
    let (x, width) = normalize(x, width);
    let (y, height) = normalize(y, height);
    let mut command = PathBuilder::new();
    command.add_rect(Rect::new(x, y, x + width, y + height), None, 0);
    let command = command.detach();
    with_canvas(id, |canvas| {
        canvas
            .path
            .add_path_with_transform(&command, &transform, None);
        Ok(())
    })?;
    Ok(BridgeValue::Undefined)
}

fn path_round_rect(id: u64, args: &[BridgeValue], offset: usize) -> Result<BridgeValue, String> {
    let x = number_arg(args, offset)? as f32;
    let y = number_arg(args, offset + 1)? as f32;
    let width = number_arg(args, offset + 2)? as f32;
    let height = number_arg(args, offset + 3)? as f32;
    let mut radii = [0.0f32; 8];
    for (index, radius) in radii.iter_mut().enumerate() {
        *radius = number_arg(args, offset + 4 + index)? as f32;
    }
    let transform = matrix_at_end(args, offset + 12)?;
    let (x, width) = normalize(x, width);
    let (y, height) = normalize(y, height);
    let mut command = PathBuilder::new();
    let rect = Rect::new(x, y, x + width, y + height);
    let rrect = skia_safe::RRect::new_rect_radii(
        rect,
        &[
            skia_safe::Point::new(radii[0], radii[1]),
            skia_safe::Point::new(radii[2], radii[3]),
            skia_safe::Point::new(radii[4], radii[5]),
            skia_safe::Point::new(radii[6], radii[7]),
        ],
    );
    command.add_rrect(rrect, None, 0);
    let command = command.detach();
    with_canvas(id, |canvas| {
        canvas
            .path
            .add_path_with_transform(&command, &transform, None);
        Ok(())
    })?;
    Ok(BridgeValue::Undefined)
}

fn path_arc_to(id: u64, args: &[BridgeValue], offset: usize) -> Result<BridgeValue, String> {
    let first = (
        number_arg(args, offset)? as f32,
        number_arg(args, offset + 1)? as f32,
    );
    let second = (
        number_arg(args, offset + 2)? as f32,
        number_arg(args, offset + 3)? as f32,
    );
    let radius = number_arg(args, offset + 4)? as f32;
    let transform = matrix_at_end(args, offset + 5)?;
    let mut command = PathBuilder::new();
    command.arc_to_tangent(first, second, radius);
    let command = command.detach();
    with_canvas(id, |canvas| {
        canvas
            .path
            .add_path_with_transform(&command, &transform, None);
        Ok(())
    })?;
    Ok(BridgeValue::Undefined)
}

fn draw_path(id: u64, args: &[BridgeValue]) -> Result<BridgeValue, String> {
    let paint_kind = string_arg(args, 4)?;
    let fill_rule = string_arg(args, 5)?;
    let style = style_arg(args, 6)?;
    let stroke = stroke_arg(args, 7)?;
    let composite = string_arg(args, 9).unwrap_or_else(|_| "source-over".into());
    let path_id = number_arg(args, 10).unwrap_or(0.0) as u64;
    with_canvas(id, |canvas| {
        let mut path = if path_id == 0 {
            canvas.path.snapshot()
        } else {
            PATHS.with(|paths| paths.borrow().get(&path_id).cloned().unwrap_or_default())
        };
        path.set_fill_type(if fill_rule == "evenodd" {
            PathFillType::EvenOdd
        } else {
            PathFillType::Winding
        });
        let canvas2d = canvas.surface.canvas();
        if paint_kind == "fill" {
            canvas2d.draw_path(&path, &paint(&style, &composite, PaintStyle::Fill));
        } else {
            let mut paint = paint(&style, &composite, PaintStyle::Stroke);
            paint.set_stroke_width(stroke.width);
            paint.set_stroke_cap(stroke.cap);
            paint.set_stroke_join(stroke.join);
            paint.set_stroke_miter(stroke.miter);
            canvas2d.draw_path(&path, &paint);
        }
        Ok(())
    })?;
    Ok(BridgeValue::Undefined)
}

fn clip(id: u64, args: &[BridgeValue]) -> Result<BridgeValue, String> {
    let fill_rule = string_arg(args, 4)?;
    let path_id = number_arg(args, 5).unwrap_or(0.0) as u64;
    with_canvas(id, |canvas| {
        let mut path = if path_id == 0 {
            canvas.path.snapshot()
        } else {
            PATHS.with(|paths| paths.borrow().get(&path_id).cloned().unwrap_or_default())
        };
        path.set_fill_type(if fill_rule == "evenodd" {
            PathFillType::EvenOdd
        } else {
            PathFillType::Winding
        });
        canvas
            .surface
            .canvas()
            .clip_path(&path, ClipOp::Intersect, true);
        Ok(())
    })?;
    Ok(BridgeValue::Undefined)
}

fn draw_text(id: u64, args: &[BridgeValue]) -> Result<BridgeValue, String> {
    let text = string_arg(args, 4)?;
    let x = number_arg(args, 5)? as f32;
    let y = number_arg(args, 6)? as f32;
    let size = number_arg(args, 7)? as f32;
    let family = string_arg(args, 8)?;
    let stroke = boolean_arg(args, 10).unwrap_or(false);
    let stroke_options = stroke_arg(args, 11)?;
    let style = style_arg(args, 12)?;
    let transform = matrix_from(args, 13)?;
    let composite = string_arg(args, 20).unwrap_or_else(|_| "source-over".into());
    with_canvas(id, |canvas| {
        let family = match family.as_str() {
            "proportional" | "sans-serif" | "system-ui" => "Helvetica",
            "monospace" | "ui-monospace" => "Menlo",
            "emoji" => "Apple Color Emoji",
            other => other,
        };
        let manager = FontMgr::new();
        let typeface = manager
            .match_family_style(family, FontStyle::normal())
            .or_else(|| manager.match_family_style("Helvetica", FontStyle::normal()))
            .or_else(|| manager.legacy_make_typeface(None, FontStyle::normal()))
            .ok_or_else(|| "no system typeface".to_string())?;
        let font = Font::new(typeface, size.max(0.1));
        let canvas2d = canvas.surface.canvas();
        let mut paint = paint(
            &style,
            &composite,
            if stroke {
                PaintStyle::Stroke
            } else {
                PaintStyle::Fill
            },
        );
        if stroke {
            paint.set_stroke_width(stroke_options.width);
        }
        canvas2d.save();
        canvas2d.concat(&transform);
        canvas2d.draw_str(text, (x, y), &font, &paint);
        canvas2d.restore();
        Ok(())
    })?;
    Ok(BridgeValue::Undefined)
}

fn put_image_data(id: u64, args: &[BridgeValue]) -> Result<BridgeValue, String> {
    let x = number_arg(args, 4)? as i32;
    let y = number_arg(args, 5)? as i32;
    let width = number_arg(args, 6)? as i32;
    let height = number_arg(args, 7)? as i32;
    let source = bytes_arg(args, 8)?;
    let alpha = CANVASES.with(|canvases| {
        canvases
            .borrow()
            .get(&id)
            .map(|canvas| canvas.alpha)
            .unwrap_or(true)
    });
    if source.len() < width as usize * height as usize * 4 {
        return Err("image data is too short".into());
    }
    let info = ImageInfo::new(
        (width, height),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let mut pixels = source.to_vec();
    if !alpha {
        for index in (3..pixels.len()).step_by(4) {
            pixels[index] = 255;
        }
    }
    let mut bitmap = skia_safe::Bitmap::new();
    bitmap.alloc_pixels_info(&info, None);
    let length = width.max(0) as usize * height.max(0) as usize * 4;
    unsafe {
        std::ptr::copy_nonoverlapping(pixels.as_ptr(), bitmap.pixels() as *mut u8, length);
    }
    bitmap.notify_pixels_changed();
    with_canvas(id, |canvas| {
        // putImageData ignores the current clip and transform.
        canvas.surface.write_pixels_from_bitmap(&bitmap, (x, y));
        Ok(())
    })?;
    Ok(BridgeValue::Undefined)
}

fn get_image_data(id: u64, args: &[BridgeValue]) -> Result<BridgeValue, String> {
    let x = number_arg(args, 4)? as i32;
    let y = number_arg(args, 5)? as i32;
    let width = number_arg(args, 6)? as i32;
    let height = number_arg(args, 7)? as i32;
    let mut pixels = vec![0u8; width.max(0) as usize * height.max(0) as usize * 4];
    let info = ImageInfo::new(
        (width.max(0), height.max(0)),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let alpha = with_canvas(id, |canvas| {
        canvas
            .surface
            .read_pixels(&info, &mut pixels, width.max(0) as usize * 4, (x, y));
        Ok(canvas.alpha)
    })?;
    if !alpha {
        for index in (3..pixels.len()).step_by(4) {
            pixels[index] = 255;
        }
    }
    Ok(BridgeValue::Binary(pixels))
}

fn encode(id: u64, args: &[BridgeValue]) -> Result<BridgeValue, String> {
    let kind = string_arg(args, 4).unwrap_or_else(|_| "image/png".into());
    if kind != "image/png" {
        return Ok(BridgeValue::String("data:,".into()));
    }
    let data = with_canvas(id, |canvas| {
        Ok(canvas
            .surface
            .image_snapshot()
            .encode_to_data(EncodedImageFormat::PNG))
    })?;
    match data {
        Some(data) => Ok(BridgeValue::String(format!(
            "data:image/png;base64,{}",
            BASE64.encode(data.as_bytes())
        ))),
        None => Ok(BridgeValue::String("data:,".into())),
    }
}

struct Style {
    color: Color,
}

fn style_arg(args: &[BridgeValue], index: usize) -> Result<Style, String> {
    let value = string_arg(args, index)?;
    let value: serde_json::Value =
        serde_json::from_str(&value).map_err(|error| error.to_string())?;
    let color = value
        .get("color")
        .and_then(|value| value.as_array())
        .and_then(|values| {
            Some((
                values.first()?.as_f64()?,
                values.get(1)?.as_f64()?,
                values.get(2)?.as_f64()?,
                values.get(3)?.as_f64()?,
            ))
        });
    let (red, green, blue, alpha) = color.unwrap_or((0.0, 0.0, 0.0, 0.0));
    Ok(Style {
        color: Color::from_argb(
            (alpha.clamp(0.0, 1.0) * 255.0) as u8,
            (red.clamp(0.0, 1.0) * 255.0) as u8,
            (green.clamp(0.0, 1.0) * 255.0) as u8,
            (blue.clamp(0.0, 1.0) * 255.0) as u8,
        ),
    })
}

fn paint(style: &Style, composite: &str, paint_style: PaintStyle) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(style.color);
    paint.set_style(paint_style);
    paint.set_blend_mode(blend_mode(composite));
    paint
}

fn blend_mode(name: &str) -> BlendMode {
    match name {
        "copy" => BlendMode::Src,
        "source-in" => BlendMode::SrcIn,
        "source-out" => BlendMode::SrcOut,
        "source-atop" => BlendMode::SrcATop,
        "destination-over" => BlendMode::DstOver,
        "destination-in" => BlendMode::DstIn,
        "destination-out" => BlendMode::DstOut,
        "destination-atop" => BlendMode::DstATop,
        "lighter" => BlendMode::Plus,
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
        _ => BlendMode::SrcOver,
    }
}

struct Stroke {
    width: f32,
    cap: skia_safe::PaintCap,
    join: skia_safe::PaintJoin,
    miter: f32,
}

fn stroke_arg(args: &[BridgeValue], index: usize) -> Result<Stroke, String> {
    let value = string_arg(args, index)?;
    let value: serde_json::Value =
        serde_json::from_str(&value).map_err(|error| error.to_string())?;
    Ok(Stroke {
        width: value
            .get("width")
            .and_then(|value| value.as_f64())
            .unwrap_or(1.0) as f32,
        cap: match value.get("cap").and_then(|value| value.as_str()) {
            Some("round") => skia_safe::PaintCap::Round,
            Some("square") => skia_safe::PaintCap::Square,
            _ => skia_safe::PaintCap::Butt,
        },
        join: match value.get("join").and_then(|value| value.as_str()) {
            Some("round") => skia_safe::PaintJoin::Round,
            Some("bevel") => skia_safe::PaintJoin::Bevel,
            _ => skia_safe::PaintJoin::Miter,
        },
        miter: value
            .get("miterLimit")
            .and_then(|value| value.as_f64())
            .unwrap_or(10.0) as f32,
    })
}

fn normalize(origin: f32, extent: f32) -> (f32, f32) {
    if extent < 0.0 {
        (origin + extent, -extent)
    } else {
        (origin, extent)
    }
}

fn matrix_from(args: &[BridgeValue], index: usize) -> Result<Matrix, String> {
    if args.len() < index + 6 {
        return Err("missing transform".into());
    }
    let a = number_arg(args, index)? as f32;
    let b = number_arg(args, index + 1)? as f32;
    let c = number_arg(args, index + 2)? as f32;
    let d = number_arg(args, index + 3)? as f32;
    let e = number_arg(args, index + 4)? as f32;
    let f = number_arg(args, index + 5)? as f32;
    Ok(Matrix::new_all(a, c, e, b, d, f, 0.0, 0.0, 1.0))
}

fn matrix_at_end(args: &[BridgeValue], command_length: usize) -> Result<Matrix, String> {
    let index = args
        .len()
        .checked_sub(6)
        .ok_or_else(|| "missing transform".to_string())?;
    let _ = command_length;
    matrix_from(args, index)
}

fn string_arg(args: &[BridgeValue], index: usize) -> Result<String, String> {
    match args.get(index) {
        Some(BridgeValue::String(value)) => Ok(value.clone()),
        _ => Err(format!("expected string at argument {index}")),
    }
}

fn number_arg(args: &[BridgeValue], index: usize) -> Result<f64, String> {
    match args.get(index) {
        Some(BridgeValue::Number(value)) => Ok(*value),
        _ => Err(format!("expected number at argument {index}")),
    }
}

fn boolean_arg(args: &[BridgeValue], index: usize) -> Result<bool, String> {
    match args.get(index) {
        Some(BridgeValue::Boolean(value)) => Ok(*value),
        _ => Err(format!("expected boolean at argument {index}")),
    }
}

fn bytes_arg(args: &[BridgeValue], index: usize) -> Result<Vec<u8>, String> {
    match args.get(index) {
        Some(BridgeValue::Binary(value)) => Ok(value.clone()),
        Some(BridgeValue::Array(values)) => Ok(values
            .iter()
            .map(|value| match value {
                BridgeValue::Number(value) => Ok(*value as u8),
                _ => Err(format!("expected byte at argument {index}")),
            })
            .collect::<Result<Vec<_>, _>>()?),
        _ => Err(format!("expected binary at argument {index}")),
    }
}

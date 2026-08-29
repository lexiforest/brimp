use super::*;

pub(super) fn dispatch(
    state: &BindingState,
    call: &NativeCall<'_>,
    operation: &str,
) -> Result<NativeValue, NativeError> {
    match operation {
        "canvasFeatures" => Ok(NativeValue::String(format!(
            "{{\"canvas\":{},\"webgl\":{},\"webgpu\":{}}}",
            state.features.canvas, state.features.webgl, state.features.webgpu,
        ))),
        "canvas2dAcquire" => {
            if !state.features.canvas {
                return Ok(NativeValue::Boolean(false));
            }
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            let alpha = required_boolean(call, 4, "Canvas alpha setting")?;
            let color_space =
                CanvasColorSpace::parse(&required_string(call, 5, "Canvas color space")?)
                    .map_err(NativeError::new)?;
            let color_type =
                CanvasColorType::parse(&required_string(call, 6, "Canvas color type")?)
                    .map_err(NativeError::new)?;
            let acquired = state
                .canvases
                .borrow_mut()
                .acquire_2d(id, width, height, alpha, color_space, color_type)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Boolean(acquired))
        }
        "canvasOriginClean" => {
            let id = required_canvas_target(state, call)?;
            Ok(NativeValue::Boolean(
                state.canvases.borrow().origin_clean(id),
            ))
        }
        "canvasReset" => {
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            let was_webgl = state.canvases.borrow().is_webgl(id);
            state
                .canvases
                .borrow_mut()
                .reset(id, width, height)
                .map_err(NativeError::new)?;
            if was_webgl {
                state
                    .angles
                    .borrow_mut()
                    .resize(id, width, height)
                    .map_err(NativeError::new)?;
            }
            Ok(NativeValue::Undefined)
        }
        "canvas2dFillRect" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            let rect = required_numbers::<4>(call, 4, "rectangle")?;
            let style = required_canvas_paint_style(call, 8)?;
            let transform =
                required_numbers::<6>(call, 9, "current transform")?.map(|value| value as f32);
            let effects = required_canvas_draw_effects(call, 15)?;
            let composite = required_string(call, 16, "composite operation")?;
            state
                .canvases
                .borrow_mut()
                .fill_rect(
                    id, width, height, rect[0], rect[1], rect[2], rect[3], style, transform,
                    &effects, &composite,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dStrokeRect" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            let rect = required_numbers::<4>(call, 4, "rectangle")?;
            let style = required_canvas_paint_style(call, 8)?;
            let transform =
                required_numbers::<6>(call, 9, "current transform")?.map(|value| value as f32);
            let stroke = required_canvas_stroke_style(call, 15)?;
            let effects = required_canvas_draw_effects(call, 16)?;
            let composite = required_string(call, 17, "composite operation")?;
            state
                .canvases
                .borrow_mut()
                .stroke_rect(
                    id, width, height, rect[0], rect[1], rect[2], rect[3], style, transform,
                    &stroke, &effects, &composite,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dClearRect" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            let rect = required_numbers::<4>(call, 4, "rectangle")?;
            let transform =
                required_numbers::<6>(call, 8, "current transform")?.map(|value| value as f32);
            state
                .canvases
                .borrow_mut()
                .clear_rect(
                    id, width, height, rect[0], rect[1], rect[2], rect[3], transform,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dDrawCanvas" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let target = required_canvas_target(state, call)?;
            let source = required_canvas_argument(state, call, 2)?;
            let target_width = required_u32(call, 3, "target canvas width")?;
            let target_height = required_u32(call, 4, "target canvas height")?;
            let source_width = required_u32(call, 5, "source canvas width")?;
            let source_height = required_u32(call, 6, "source canvas height")?;
            let rectangles = required_numbers::<8>(call, 7, "image rectangles")?;
            let alpha = required_number(call, 15, "global alpha")? as f32;
            let transform =
                required_numbers::<6>(call, 16, "current transform")?.map(|value| value as f32);
            let smoothing = required_boolean(call, 22, "image smoothing")?;
            let effects = required_canvas_draw_effects(call, 23)?;
            let composite = required_string(call, 24, "composite operation")?;
            if state.canvases.borrow().is_webgl(source) && source_width > 0 && source_height > 0 {
                let mut pixels = state
                    .angles
                    .borrow()
                    .read_canvas_rgba(source, 0, 0, source_width, source_height)
                    .map_err(NativeError::new)?;
                flip_rows(&mut pixels, source_width, source_height);
                state
                    .canvases
                    .borrow_mut()
                    .write_rgba(
                        source,
                        source_width,
                        source_height,
                        0,
                        0,
                        source_width,
                        source_height,
                        &pixels,
                    )
                    .map_err(NativeError::new)?;
            }
            state
                .canvases
                .borrow_mut()
                .draw_canvas(
                    target,
                    target_width,
                    target_height,
                    source,
                    source_width,
                    source_height,
                    [
                        rectangles[0] as f32,
                        rectangles[1] as f32,
                        rectangles[2] as f32,
                        rectangles[3] as f32,
                    ],
                    [
                        rectangles[4] as f32,
                        rectangles[5] as f32,
                        rectangles[6] as f32,
                        rectangles[7] as f32,
                    ],
                    alpha,
                    transform,
                    smoothing,
                    &effects,
                    &composite,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvasCreateImageBitmap" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let source = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            if state.canvases.borrow().is_webgl(source) && width > 0 && height > 0 {
                let mut pixels = state
                    .angles
                    .borrow()
                    .read_canvas_rgba(source, 0, 0, width, height)
                    .map_err(NativeError::new)?;
                flip_rows(&mut pixels, width, height);
                state
                    .canvases
                    .borrow_mut()
                    .write_rgba(source, width, height, 0, 0, width, height, &pixels)
                    .map_err(NativeError::new)?;
            }
            let (id, width, height) = state
                .canvases
                .borrow_mut()
                .create_image_bitmap(source, width, height)
                .map_err(NativeError::new)?;
            let origin_clean = state.canvases.borrow().origin_clean(source);
            Ok(NativeValue::String(
                serde_json::json!({ "id": id, "width": width, "height": height, "originClean": origin_clean }).to_string(),
            ))
        }
        "canvasDecodeImageBitmap" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let bytes = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing ImageBitmap encoded bytes"))?
                .to_bytes()?;
            let (id, width, height) = state
                .canvases
                .borrow_mut()
                .decode_image_bitmap(&bytes)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(
                serde_json::json!({ "id": id, "width": width, "height": height, "originClean": true }).to_string(),
            ))
        }
        "canvasDestroyImageBitmap" => {
            let bitmap = required_u64(call, 2, "ImageBitmap")?;
            state
                .canvases
                .borrow_mut()
                .destroy_image_bitmap(bitmap)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dDrawImage" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let target = required_canvas_target(state, call)?;
            let source = required_image_argument(state, call, 2)?;
            let target_width = required_u32(call, 3, "target canvas width")?;
            let target_height = required_u32(call, 4, "target canvas height")?;
            let rectangles = required_numbers::<8>(call, 5, "image rectangles")?;
            let alpha = required_number(call, 13, "global alpha")? as f32;
            let transform =
                required_numbers::<6>(call, 14, "current transform")?.map(|value| value as f32);
            let smoothing = required_boolean(call, 20, "image smoothing")?;
            let effects = required_canvas_draw_effects(call, 21)?;
            let composite = required_string(call, 22, "composite operation")?;
            let origin_clean = required_boolean(call, 23, "image origin cleanliness")?;
            let (image_width, image_height, pixels) = decoded_raster_image(state, source)?;
            state
                .canvases
                .borrow_mut()
                .draw_rgba_image(
                    target,
                    target_width,
                    target_height,
                    image_width,
                    image_height,
                    &pixels,
                    [
                        rectangles[0] as f32,
                        rectangles[1] as f32,
                        rectangles[2] as f32,
                        rectangles[3] as f32,
                    ],
                    [
                        rectangles[4] as f32,
                        rectangles[5] as f32,
                        rectangles[6] as f32,
                        rectangles[7] as f32,
                    ],
                    alpha,
                    transform,
                    smoothing,
                    &effects,
                    &composite,
                    origin_clean,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dDrawImageBitmap" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let target = required_canvas_target(state, call)?;
            let bitmap = required_u64(call, 2, "ImageBitmap")?;
            let target_width = required_u32(call, 3, "target canvas width")?;
            let target_height = required_u32(call, 4, "target canvas height")?;
            let rectangles = required_numbers::<8>(call, 5, "image rectangles")?;
            let alpha = required_number(call, 13, "global alpha")? as f32;
            let transform =
                required_numbers::<6>(call, 14, "current transform")?.map(|value| value as f32);
            let smoothing = required_boolean(call, 20, "image smoothing")?;
            let effects = required_canvas_draw_effects(call, 21)?;
            let composite = required_string(call, 22, "composite operation")?;
            state
                .canvases
                .borrow_mut()
                .draw_image_bitmap(
                    target,
                    target_width,
                    target_height,
                    bitmap,
                    [
                        rectangles[0] as f32,
                        rectangles[1] as f32,
                        rectangles[2] as f32,
                        rectangles[3] as f32,
                    ],
                    [
                        rectangles[4] as f32,
                        rectangles[5] as f32,
                        rectangles[6] as f32,
                        rectangles[7] as f32,
                    ],
                    alpha,
                    transform,
                    smoothing,
                    &effects,
                    &composite,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dCreatePattern" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let _ = required_canvas_target(state, call)?;
            let source = required_canvas_argument(state, call, 2)?;
            let source_width = required_u32(call, 3, "pattern source width")?;
            let source_height = required_u32(call, 4, "pattern source height")?;
            let repetition = required_string(call, 5, "pattern repetition")?;
            if state.canvases.borrow().is_webgl(source) {
                let mut pixels = state
                    .angles
                    .borrow()
                    .read_canvas_rgba(source, 0, 0, source_width, source_height)
                    .map_err(NativeError::new)?;
                flip_rows(&mut pixels, source_width, source_height);
                state
                    .canvases
                    .borrow_mut()
                    .write_rgba(
                        source,
                        source_width,
                        source_height,
                        0,
                        0,
                        source_width,
                        source_height,
                        &pixels,
                    )
                    .map_err(NativeError::new)?;
            }
            let pattern = state
                .canvases
                .borrow_mut()
                .create_pattern(source, source_width, source_height, &repetition)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(pattern as f64))
        }
        "canvas2dCreateImagePattern" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let _ = required_canvas_target(state, call)?;
            let source = required_image_argument(state, call, 2)?;
            let repetition = required_string(call, 3, "pattern repetition")?;
            let origin_clean = required_boolean(call, 4, "image origin cleanliness")?;
            let (width, height, pixels) = decoded_raster_image(state, source)?;
            let pattern = state
                .canvases
                .borrow_mut()
                .create_rgba_pattern(width, height, &pixels, &repetition, origin_clean)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(pattern as f64))
        }
        "canvas2dCreateImageBitmapPattern" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let _ = required_canvas_target(state, call)?;
            let bitmap = required_u64(call, 2, "ImageBitmap")?;
            let repetition = required_string(call, 3, "pattern repetition")?;
            let pattern = state
                .canvases
                .borrow_mut()
                .create_image_bitmap_pattern(bitmap, &repetition)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(pattern as f64))
        }
        "canvas2dSetPatternTransform" => {
            let _ = required_canvas_target(state, call)?;
            let pattern = required_u64(call, 2, "CanvasPattern")?;
            let transform =
                required_numbers::<6>(call, 3, "pattern transform")?.map(|value| value as f32);
            state
                .canvases
                .borrow_mut()
                .set_pattern_transform(pattern, transform)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dCreateGradient" => {
            let _ = required_canvas_target(state, call)?;
            let kind = required_string(call, 2, "gradient type")?;
            let id = match kind.as_str() {
                "linear" => {
                    let coordinates =
                        required_numbers::<4>(call, 3, "linear gradient coordinates")?
                            .map(|value| value as f32);
                    let transform = required_numbers::<6>(call, 7, "gradient transform")?
                        .map(|value| value as f32);
                    state
                        .canvases
                        .borrow_mut()
                        .create_linear_gradient(coordinates, transform)
                }
                "radial" => {
                    let coordinates =
                        required_numbers::<6>(call, 3, "radial gradient coordinates")?
                            .map(|value| value as f32);
                    let transform = required_numbers::<6>(call, 9, "gradient transform")?
                        .map(|value| value as f32);
                    state
                        .canvases
                        .borrow_mut()
                        .create_radial_gradient(coordinates, transform)
                }
                "conic" => {
                    let coordinates = required_numbers::<3>(call, 3, "conic gradient coordinates")?
                        .map(|value| value as f32);
                    let transform = required_numbers::<6>(call, 6, "gradient transform")?
                        .map(|value| value as f32);
                    state
                        .canvases
                        .borrow_mut()
                        .create_conic_gradient(coordinates, transform)
                }
                _ => Err("invalid Canvas gradient type".to_owned()),
            }
            .map_err(NativeError::new)?;
            Ok(NativeValue::Number(id as f64))
        }
        "canvas2dAddColorStop" => {
            let _ = required_canvas_target(state, call)?;
            let gradient = required_u64(call, 2, "CanvasGradient")?;
            let offset = required_number(call, 3, "color stop offset")? as f32;
            let color = required_numbers::<4>(call, 4, "color stop")?.map(|value| value as f32);
            state
                .canvases
                .borrow_mut()
                .add_color_stop(gradient, offset, color)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dMeasureText" => {
            let _ = required_canvas_target(state, call)?;
            let text = required_string(call, 2, "text")?;
            let font_size = required_number(call, 3, "font size")? as f32;
            let font_family = required_string(call, 4, "font family")?;
            let direction = required_string(call, 5, "text direction")?;
            let metrics = state
                .canvases
                .borrow()
                .measure_text(&text, font_size, &font_family, &direction)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(
                serde_json::to_string(&metrics).map_err(|error| {
                    NativeError::new(format!("could not encode TextMetrics: {error}"))
                })?,
            ))
        }
        "canvas2dDrawText" => {
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            let text = required_string(call, 4, "text")?;
            let x = required_number(call, 5, "text x")? as f32;
            let y = required_number(call, 6, "text y")? as f32;
            let font_size = required_number(call, 7, "font size")? as f32;
            let font_family = required_string(call, 8, "font family")?;
            let max_width = required_number(call, 9, "maximum text width")? as f32;
            let stroke = required_boolean(call, 10, "text stroke")?;
            let stroke_style = required_canvas_stroke_style(call, 11)?;
            let style = required_canvas_paint_style(call, 12)?;
            let transform =
                required_numbers::<6>(call, 13, "current transform")?.map(|value| value as f32);
            let effects = required_canvas_draw_effects(call, 19)?;
            let composite = required_string(call, 20, "composite operation")?;
            let direction = required_string(call, 21, "text direction")?;
            state
                .canvases
                .borrow_mut()
                .draw_text(
                    id,
                    width,
                    height,
                    &text,
                    x,
                    y,
                    font_size,
                    &font_family,
                    &direction,
                    (max_width >= 0.0).then_some(max_width),
                    stroke,
                    &stroke_style,
                    style,
                    transform,
                    &effects,
                    &composite,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dCreatePath" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let kind = required_string(call, 2, "Path2D source kind")?;
            let path = match kind.as_str() {
                "empty" => state.canvases.borrow_mut().create_path(),
                "copy" => {
                    state
                        .canvases
                        .borrow_mut()
                        .copy_path(required_u64(call, 3, "source Path2D")?)
                }
                "svg" => state
                    .canvases
                    .borrow_mut()
                    .create_svg_path(&required_string(call, 3, "SVG path data")?),
                _ => Err("invalid Path2D source kind".to_owned()),
            }
            .map_err(NativeError::new)?;
            Ok(NativeValue::Number(path as f64))
        }
        "canvas2dAddPath" => {
            let target = required_u64(call, 2, "target Path2D")?;
            let source = required_u64(call, 3, "source Path2D")?;
            let transform =
                required_numbers::<6>(call, 4, "Path2D transform")?.map(|value| value as f32);
            state
                .canvases
                .borrow_mut()
                .add_path(target, source, transform)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dPath2DClose" => {
            let path = required_u64(call, 2, "Path2D")?;
            state
                .canvases
                .borrow_mut()
                .path2d_close(path)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dPath2DPoint" => {
            let path = required_u64(call, 2, "Path2D")?;
            let operation = required_string(call, 3, "path operation")?;
            let point_count = match operation.as_str() {
                "move" | "line" => 2,
                "quadratic" => 4,
                "bezier" => 6,
                _ => return Err(NativeError::new("invalid Path2D operation")),
            };
            let mut points = Vec::with_capacity(point_count);
            for index in 0..point_count {
                points.push(required_number(call, 4 + index, "path point")?);
            }
            state
                .canvases
                .borrow_mut()
                .path2d_points(path, &operation, &points)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dPath2DArcTo" => {
            let path = required_u64(call, 2, "Path2D")?;
            let arc = required_numbers::<5>(call, 3, "tangent arc")?;
            state
                .canvases
                .borrow_mut()
                .path2d_arc_to(path, [arc[0], arc[1]], [arc[2], arc[3]], arc[4])
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dPath2DRect" => {
            let path = required_u64(call, 2, "Path2D")?;
            let rect = required_numbers::<4>(call, 3, "rectangle")?;
            state
                .canvases
                .borrow_mut()
                .path2d_rect(path, rect)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dPath2DRoundRect" => {
            let path = required_u64(call, 2, "Path2D")?;
            let rect = required_numbers::<4>(call, 3, "rounded rectangle")?;
            let radii = required_numbers::<8>(call, 7, "rounded rectangle radii")?;
            state
                .canvases
                .borrow_mut()
                .path2d_round_rect(path, rect, radii)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dPath2DArc" => {
            let path = required_u64(call, 2, "Path2D")?;
            let arc = required_numbers::<5>(call, 3, "arc")?;
            state
                .canvases
                .borrow_mut()
                .path2d_arc(path, [arc[0], arc[1]], arc[2], arc[3], arc[4])
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dPath2DEllipse" => {
            let path = required_u64(call, 2, "Path2D")?;
            let ellipse = required_numbers::<7>(call, 3, "ellipse")?;
            state
                .canvases
                .borrow_mut()
                .path2d_ellipse(
                    path,
                    [ellipse[0], ellipse[1]],
                    [ellipse[2], ellipse[3]],
                    ellipse[4],
                    ellipse[5],
                    ellipse[6],
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dBeginPath" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            state
                .canvases
                .borrow_mut()
                .begin_path(id, width, height)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dSave" => {
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            state
                .canvases
                .borrow_mut()
                .save(id, width, height)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dRestore" => {
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            state
                .canvases
                .borrow_mut()
                .restore(id, width, height)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dClosePath" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            state
                .canvases
                .borrow_mut()
                .close_path(id, width, height)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dPathPoint" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            let path_operation = required_string(call, 4, "path operation")?;
            let point_count = match path_operation.as_str() {
                "move" | "line" => 2,
                "quadratic" => 4,
                "bezier" => 6,
                _ => return Err(NativeError::new("invalid Canvas path operation")),
            };
            let mut points = Vec::with_capacity(point_count);
            for index in 0..point_count {
                points.push(required_number(call, 5 + index, "path point")?);
            }
            let mut transform = [0.0; 6];
            for (index, value) in transform.iter_mut().enumerate() {
                *value =
                    required_number(call, 5 + point_count + index, "current transform")? as f32;
            }
            state
                .canvases
                .borrow_mut()
                .path_points(id, width, height, &path_operation, &points, transform)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dPathArcTo" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            let arc = required_numbers::<5>(call, 4, "tangent arc")?;
            let transform =
                required_numbers::<6>(call, 9, "current transform")?.map(|value| value as f32);
            state
                .canvases
                .borrow_mut()
                .path_arc_to(
                    id,
                    width,
                    height,
                    [arc[0], arc[1]],
                    [arc[2], arc[3]],
                    arc[4],
                    transform,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dPathRect" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            let rect = required_numbers::<4>(call, 4, "rectangle")?;
            let transform =
                required_numbers::<6>(call, 8, "current transform")?.map(|value| value as f32);
            state
                .canvases
                .borrow_mut()
                .path_rect(
                    id, width, height, rect[0], rect[1], rect[2], rect[3], transform,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dPathRoundRect" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            let rect = required_numbers::<4>(call, 4, "rounded rectangle")?;
            let radii = required_numbers::<8>(call, 8, "rounded rectangle radii")?;
            let transform =
                required_numbers::<6>(call, 16, "current transform")?.map(|value| value as f32);
            state
                .canvases
                .borrow_mut()
                .path_round_rect(id, width, height, rect, radii, transform)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dPathArc" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            let arc = required_numbers::<5>(call, 4, "arc")?;
            let transform =
                required_numbers::<6>(call, 9, "current transform")?.map(|value| value as f32);
            state
                .canvases
                .borrow_mut()
                .path_arc(
                    id, width, height, arc[0], arc[1], arc[2], arc[3], arc[4], transform,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dPathEllipse" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            let ellipse = required_numbers::<7>(call, 4, "ellipse")?;
            let transform =
                required_numbers::<6>(call, 11, "current transform")?.map(|value| value as f32);
            state
                .canvases
                .borrow_mut()
                .path_ellipse(
                    id,
                    width,
                    height,
                    [ellipse[0], ellipse[1]],
                    [ellipse[2], ellipse[3]],
                    ellipse[4],
                    ellipse[5],
                    ellipse[6],
                    transform,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dDrawPath" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            let style = required_string(call, 4, "path drawing style")?;
            let rule = required_string(call, 5, "fill rule")?;
            let paint = required_canvas_paint_style(call, 6)?;
            let stroke = required_canvas_stroke_style(call, 7)?;
            let effects = required_canvas_draw_effects(call, 8)?;
            let composite = required_string(call, 9, "composite operation")?;
            let path = required_u64(call, 10, "Path2D")?;
            state
                .canvases
                .borrow_mut()
                .draw_path(
                    id,
                    width,
                    height,
                    (path != 0).then_some(path),
                    style == "stroke",
                    rule == "evenodd",
                    paint,
                    &stroke,
                    &effects,
                    &composite,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dClip" => {
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            let rule = required_string(call, 4, "fill rule")?;
            let path = required_u64(call, 5, "Path2D")?;
            state
                .canvases
                .borrow_mut()
                .clip_path(
                    id,
                    width,
                    height,
                    (path != 0).then_some(path),
                    rule == "evenodd",
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvas2dIsPointInPath" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            let x = required_number(call, 4, "x")?;
            let y = required_number(call, 5, "y")?;
            let rule = required_string(call, 6, "fill rule")?;
            let path = required_u64(call, 7, "Path2D")?;
            let contains = state
                .canvases
                .borrow_mut()
                .point_in_path(
                    id,
                    width,
                    height,
                    (path != 0).then_some(path),
                    x,
                    y,
                    rule == "evenodd",
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Boolean(contains))
        }
        "canvas2dIsPointInStroke" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            let point = required_numbers::<2>(call, 4, "stroke hit-test point")?;
            let stroke = required_canvas_stroke_style(call, 6)?;
            let path = required_u64(call, 7, "Path2D")?;
            let contains = state
                .canvases
                .borrow_mut()
                .point_in_stroke(
                    id,
                    width,
                    height,
                    (path != 0).then_some(path),
                    point[0],
                    point[1],
                    &stroke,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Boolean(contains))
        }
        "canvas2dGetImageData" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            let source_x = required_i32(call, 4, "source x")?;
            let source_y = required_i32(call, 5, "source y")?;
            let read_width = required_u32(call, 6, "read width")?;
            let read_height = required_u32(call, 7, "read height")?;
            let color_space =
                CanvasColorSpace::parse(&required_string(call, 8, "ImageData color space")?)
                    .map_err(NativeError::new)?;
            let color_type =
                CanvasColorType::parse(&required_string(call, 9, "ImageData pixel format")?)
                    .map_err(NativeError::new)?;
            let bytes = state
                .canvases
                .borrow_mut()
                .read_image_data(
                    id,
                    width,
                    height,
                    source_x,
                    source_y,
                    read_width,
                    read_height,
                    color_space,
                    color_type,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Bytes(bytes))
        }
        "canvas2dPutImageData" => {
            if !state.features.canvas {
                return Err(NativeError::new("Canvas 2D is disabled"));
            }
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            let destination_x = required_i32(call, 4, "destination x")?;
            let destination_y = required_i32(call, 5, "destination y")?;
            let image_width = required_u32(call, 6, "image width")?;
            let image_height = required_u32(call, 7, "image height")?;
            let pixels = call
                .argument(8)
                .ok_or_else(|| NativeError::new("missing ImageData pixels"))?
                .to_bytes()?;
            let color_space =
                CanvasColorSpace::parse(&required_string(call, 9, "ImageData color space")?)
                    .map_err(NativeError::new)?;
            let color_type =
                CanvasColorType::parse(&required_string(call, 10, "ImageData pixel format")?)
                    .map_err(NativeError::new)?;
            state
                .canvases
                .borrow_mut()
                .write_image_data(
                    id,
                    width,
                    height,
                    destination_x,
                    destination_y,
                    image_width,
                    image_height,
                    &pixels,
                    color_space,
                    color_type,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "canvasEncode" => {
            use base64::Engine as _;

            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            let mime_type = required_string(call, 4, "image MIME type")?.to_ascii_lowercase();
            let quality = required_u32(call, 5, "image quality")?.min(100) as u8;
            if state.canvases.borrow().is_webgl(id) && width > 0 && height > 0 {
                let mut pixels = state
                    .angles
                    .borrow()
                    .read_canvas_rgba(id, 0, 0, width, height)
                    .map_err(NativeError::new)?;
                flip_rows(&mut pixels, width, height);
                state
                    .canvases
                    .borrow_mut()
                    .write_rgba(id, width, height, 0, 0, width, height, &pixels)
                    .map_err(NativeError::new)?;
            }
            let Some((encoded_type, bytes)) = state
                .canvases
                .borrow_mut()
                .encode(id, width, height, &mime_type, quality)
                .map_err(NativeError::new)?
            else {
                return Ok(NativeValue::String("data:,".to_owned()));
            };
            Ok(NativeValue::String(format!(
                "data:{encoded_type};base64,{}",
                base64::engine::general_purpose::STANDARD.encode(bytes),
            )))
        }
        _ => Err(NativeError::new(format!(
            "unknown native Canvas operation: {operation}"
        ))),
    }
}

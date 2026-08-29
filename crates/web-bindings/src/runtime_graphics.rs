// Graphics-source conversion and Canvas payload decoding.
fn webgl_texture_source(
    state: &BindingState,
    call: &NativeCall<'_>,
    base: usize,
) -> Result<(u32, u32, Vec<u8>), NativeError> {
    let flip_y = required_boolean(call, base + 4, "texture source orientation")?;
    let premultiply_alpha = required_boolean(call, base + 5, "texture source alpha conversion")?;
    let (width, height, mut pixels) = rgba_texture_source(state, call, base)?;
    if flip_y {
        flip_rows(&mut pixels, width, height);
    }
    if premultiply_alpha {
        premultiply_rgba(&mut pixels);
    }
    Ok((width, height, pixels))
}

fn rgba_texture_source(
    state: &BindingState,
    call: &NativeCall<'_>,
    base: usize,
) -> Result<(u32, u32, Vec<u8>), NativeError> {
    let source_kind = required_string(call, base, "texture source kind")?;
    let expected_width = required_u32(call, base + 1, "texture source width")?;
    let expected_height = required_u32(call, base + 2, "texture source height")?;
    let declared_origin_clean =
        required_boolean(call, base + 6, "texture source origin cleanliness")?;
    if !declared_origin_clean {
        return Err(NativeError::new("texture source is not origin-clean"));
    }

    let (width, height, pixels, origin_clean) = match source_kind.as_str() {
        "image-data" => {
            let pixels = call
                .argument(base + 3)
                .ok_or_else(|| NativeError::new("missing ImageData pixels"))?
                .to_bytes()?;
            let source_color_space =
                CanvasColorSpace::parse(&required_string(call, base + 7, "ImageData color space")?)
                    .map_err(NativeError::new)?;
            let source_color_type =
                CanvasColorType::parse(&required_string(call, base + 8, "ImageData pixel format")?)
                    .map_err(NativeError::new)?;
            let pixels = if source_color_space == CanvasColorSpace::Srgb
                && source_color_type == CanvasColorType::Unorm8
            {
                pixels
            } else {
                CanvasStore::convert_image_data_to_unorm8(
                    expected_width,
                    expected_height,
                    &pixels,
                    source_color_space,
                    source_color_type,
                    CanvasColorSpace::Srgb,
                )
                .map_err(NativeError::new)?
            };
            (expected_width, expected_height, pixels, true)
        }
        "canvas" => {
            let source = required_canvas_argument(state, call, base + 3)?;
            if state.canvases.borrow().is_webgl(source) && expected_width > 0 && expected_height > 0
            {
                let mut pixels = state
                    .angles
                    .borrow()
                    .read_canvas_rgba(source, 0, 0, expected_width, expected_height)
                    .map_err(NativeError::new)?;
                flip_rows(&mut pixels, expected_width, expected_height);
                state
                    .canvases
                    .borrow_mut()
                    .write_rgba(
                        source,
                        expected_width,
                        expected_height,
                        0,
                        0,
                        expected_width,
                        expected_height,
                        &pixels,
                    )
                    .map_err(NativeError::new)?;
            }
            let origin_clean = state.canvases.borrow().origin_clean(source);
            let pixels = state
                .canvases
                .borrow_mut()
                .read_rgba(
                    source,
                    expected_width,
                    expected_height,
                    0,
                    0,
                    expected_width,
                    expected_height,
                )
                .map_err(NativeError::new)?;
            (expected_width, expected_height, pixels, origin_clean)
        }
        "image" => {
            let source = required_image_argument(state, call, base + 3)?;
            let (width, height, pixels) = decoded_raster_image(state, source)?;
            (width, height, pixels, declared_origin_clean)
        }
        "image-bitmap" => {
            let source = required_u64(call, base + 3, "ImageBitmap")?;
            state
                .canvases
                .borrow()
                .image_bitmap_rgba(source)
                .map_err(NativeError::new)?
        }
        _ => return Err(NativeError::new("unsupported texture source kind")),
    };
    if !origin_clean {
        return Err(NativeError::new("texture source is not origin-clean"));
    }
    if width != expected_width || height != expected_height {
        return Err(NativeError::new("texture source dimensions changed"));
    }
    let expected_bytes = usize::try_from(u64::from(width) * u64::from(height) * 4)
        .map_err(|_| NativeError::new("texture source is too large"))?;
    if pixels.len() != expected_bytes {
        return Err(NativeError::new("texture source pixels are incomplete"));
    }
    Ok((width, height, pixels))
}

fn gpu_external_rgba_texture_source(
    state: &BindingState,
    call: &NativeCall<'_>,
    base: usize,
    destination_color_space: CanvasColorSpace,
) -> Result<(u32, u32, Vec<u8>), NativeError> {
    let source_kind = required_string(call, base, "texture source kind")?;
    let expected_width = required_u32(call, base + 1, "texture source width")?;
    let expected_height = required_u32(call, base + 2, "texture source height")?;
    let declared_origin_clean =
        required_boolean(call, base + 6, "texture source origin cleanliness")?;
    if !declared_origin_clean {
        return Err(NativeError::new("texture source is not origin-clean"));
    }

    let (width, height, pixels, origin_clean) = match source_kind.as_str() {
        "image-data" => {
            let pixels = call
                .argument(base + 3)
                .ok_or_else(|| NativeError::new("missing ImageData pixels"))?
                .to_bytes()?;
            let source_color_space = CanvasColorSpace::parse(&required_string(
                call,
                base + 12,
                "ImageData color space",
            )?)
            .map_err(NativeError::new)?;
            let source_color_type = CanvasColorType::parse(&required_string(
                call,
                base + 13,
                "ImageData pixel format",
            )?)
            .map_err(NativeError::new)?;
            let pixels = if source_color_space == destination_color_space
                && source_color_type == CanvasColorType::Unorm8
            {
                pixels
            } else {
                CanvasStore::convert_image_data_to_unorm8(
                    expected_width,
                    expected_height,
                    &pixels,
                    source_color_space,
                    source_color_type,
                    destination_color_space,
                )
                .map_err(NativeError::new)?
            };
            (expected_width, expected_height, pixels, true)
        }
        "canvas" => {
            let source = required_canvas_argument(state, call, base + 3)?;
            if state.canvases.borrow().is_webgl(source) && expected_width > 0 && expected_height > 0
            {
                let mut pixels = state
                    .angles
                    .borrow()
                    .read_canvas_rgba(source, 0, 0, expected_width, expected_height)
                    .map_err(NativeError::new)?;
                flip_rows(&mut pixels, expected_width, expected_height);
                state
                    .canvases
                    .borrow_mut()
                    .write_rgba(
                        source,
                        expected_width,
                        expected_height,
                        0,
                        0,
                        expected_width,
                        expected_height,
                        &pixels,
                    )
                    .map_err(NativeError::new)?;
            }
            let origin_clean = state.canvases.borrow().origin_clean(source);
            let pixels = state
                .canvases
                .borrow_mut()
                .read_image_data(
                    source,
                    expected_width,
                    expected_height,
                    0,
                    0,
                    expected_width,
                    expected_height,
                    destination_color_space,
                    CanvasColorType::Unorm8,
                )
                .map_err(NativeError::new)?;
            (expected_width, expected_height, pixels, origin_clean)
        }
        "image" => {
            let source = required_image_argument(state, call, base + 3)?;
            let (width, height, pixels) = decoded_raster_image(state, source)?;
            let pixels = if destination_color_space == CanvasColorSpace::Srgb {
                pixels
            } else {
                CanvasStore::convert_image_data_to_unorm8(
                    width,
                    height,
                    &pixels,
                    CanvasColorSpace::Srgb,
                    CanvasColorType::Unorm8,
                    destination_color_space,
                )
                .map_err(NativeError::new)?
            };
            (width, height, pixels, declared_origin_clean)
        }
        "image-bitmap" => {
            let source = required_u64(call, base + 3, "ImageBitmap")?;
            state
                .canvases
                .borrow()
                .image_bitmap_rgba_in_color_space(source, destination_color_space)
                .map_err(NativeError::new)?
        }
        _ => return Err(NativeError::new("unsupported texture source kind")),
    };
    if !origin_clean {
        return Err(NativeError::new("texture source is not origin-clean"));
    }
    if width != expected_width || height != expected_height {
        return Err(NativeError::new("texture source dimensions changed"));
    }
    let expected_bytes = usize::try_from(u64::from(width) * u64::from(height) * 4)
        .map_err(|_| NativeError::new("texture source is too large"))?;
    if pixels.len() != expected_bytes {
        return Err(NativeError::new("texture source pixels are incomplete"));
    }
    Ok((width, height, pixels))
}

fn required_canvas_paint_style(
    call: &NativeCall<'_>,
    index: usize,
) -> Result<CanvasPaintStyle, NativeError> {
    let encoded = required_string(call, index, "Canvas paint style")?;
    let value: serde_json::Value = serde_json::from_str(&encoded)
        .map_err(|error| NativeError::new(format!("invalid Canvas paint style: {error}")))?;
    if let Some(values) = value.get("color").and_then(serde_json::Value::as_array) {
        if values.len() != 4 {
            return Err(NativeError::new("Canvas color must have four components"));
        }
        let mut color = [0.0_f32; 4];
        for (target, value) in color.iter_mut().zip(values) {
            *target = value
                .as_f64()
                .ok_or_else(|| NativeError::new("Canvas color component is not a number"))?
                as f32;
        }
        return Ok(CanvasPaintStyle::Color(color));
    }
    let (id, pattern) = if let Some(id) = value.get("gradient").and_then(serde_json::Value::as_u64)
    {
        (id, false)
    } else if let Some(id) = value.get("pattern").and_then(serde_json::Value::as_u64) {
        (id, true)
    } else {
        return Err(NativeError::new(
            "Canvas paint style has no color, gradient, or pattern",
        ));
    };
    let alpha = value
        .get("alpha")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| NativeError::new("Canvas gradient or pattern style has no alpha"))?
        as f32;
    Ok(if pattern {
        CanvasPaintStyle::Pattern { id, alpha }
    } else {
        CanvasPaintStyle::Gradient { id, alpha }
    })
}

fn required_canvas_stroke_style(
    call: &NativeCall<'_>,
    index: usize,
) -> Result<CanvasStrokeStyle, NativeError> {
    let encoded = required_string(call, index, "Canvas stroke style")?;
    let value: serde_json::Value = serde_json::from_str(&encoded)
        .map_err(|error| NativeError::new(format!("invalid Canvas stroke style: {error}")))?;
    let number = |key: &str| {
        value
            .get(key)
            .and_then(serde_json::Value::as_f64)
            .ok_or_else(|| NativeError::new(format!("Canvas stroke style has no {key}")))
            .map(|value| value as f32)
    };
    let string = |key: &str| {
        value
            .get(key)
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| NativeError::new(format!("Canvas stroke style has no {key}")))
            .map(str::to_owned)
    };
    let dash = value
        .get("dash")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| NativeError::new("Canvas stroke style has no dash array"))?
        .iter()
        .map(|value| {
            value
                .as_f64()
                .ok_or_else(|| NativeError::new("Canvas dash entry is not a number"))
                .map(|value| value as f32)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CanvasStrokeStyle {
        width: number("width")?,
        cap: string("cap")?,
        join: string("join")?,
        miter_limit: number("miterLimit")?,
        dash,
        dash_offset: number("dashOffset")?,
    })
}

fn required_canvas_draw_effects(
    call: &NativeCall<'_>,
    index: usize,
) -> Result<CanvasDrawEffects, NativeError> {
    let encoded = required_string(call, index, "Canvas drawing effects")?;
    let value: serde_json::Value = serde_json::from_str(&encoded)
        .map_err(|error| NativeError::new(format!("invalid Canvas drawing effects: {error}")))?;
    let shadow = value
        .get("shadow")
        .ok_or_else(|| NativeError::new("Canvas drawing effects have no shadow"))?;
    let number = |key: &str| {
        shadow
            .get(key)
            .and_then(serde_json::Value::as_f64)
            .ok_or_else(|| NativeError::new(format!("Canvas shadow style has no {key}")))
            .map(|value| value as f32)
    };
    let values = shadow
        .get("color")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| NativeError::new("Canvas shadow style has no color"))?;
    if values.len() != 4 {
        return Err(NativeError::new(
            "Canvas shadow color must have four components",
        ));
    }
    let mut color = [0.0; 4];
    for (target, value) in color.iter_mut().zip(values) {
        *target = value
            .as_f64()
            .ok_or_else(|| NativeError::new("Canvas shadow color component is not a number"))?
            as f32;
    }
    let filters = value
        .get("filters")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| NativeError::new("Canvas drawing effects have no filter list"))?
        .iter()
        .enumerate()
        .map(|(operation_index, operation)| {
            let input = required_canvas_filter_input(operation, "input", operation_index)?;
            if let Some(values) = operation.get("blur").and_then(serde_json::Value::as_array) {
                if values.len() != 2 {
                    return Err(NativeError::new(
                        "Canvas blur filter must have two standard deviations",
                    ));
                }
                let mut sigma = [0.0_f32; 2];
                for (target, value) in sigma.iter_mut().zip(values) {
                    *target = value
                        .as_f64()
                        .filter(|value| *value >= 0.0)
                        .ok_or_else(|| {
                            NativeError::new(
                                "Canvas blur standard deviation must be a non-negative number",
                            )
                        })? as f32;
                }
                return Ok(CanvasFilterOperation::Blur {
                    sigma_x: sigma[0],
                    sigma_y: sigma[1],
                    input,
                });
            }
            if let Some(values) = operation
                .get("offset")
                .and_then(serde_json::Value::as_array)
            {
                if values.len() != 2 {
                    return Err(NativeError::new(
                        "Canvas offset filter must have two components",
                    ));
                }
                let mut offset = [0.0_f32; 2];
                for (target, value) in offset.iter_mut().zip(values) {
                    *target = value.as_f64().ok_or_else(|| {
                        NativeError::new("Canvas offset component is not a number")
                    })? as f32;
                }
                return Ok(CanvasFilterOperation::Offset {
                    x: offset[0],
                    y: offset[1],
                    input,
                });
            }
            if let Some(shadow) = operation.get("dropShadow") {
                let number = |key: &str| {
                    shadow
                        .get(key)
                        .and_then(serde_json::Value::as_f64)
                        .ok_or_else(|| {
                            NativeError::new(format!("Canvas drop-shadow filter has no {key}"))
                        })
                        .map(|value| value as f32)
                };
                let values = shadow
                    .get("color")
                    .and_then(serde_json::Value::as_array)
                    .ok_or_else(|| NativeError::new("Canvas drop-shadow filter has no color"))?;
                if values.len() != 4 {
                    return Err(NativeError::new(
                        "Canvas drop-shadow color must have four components",
                    ));
                }
                let mut color = [0.0; 4];
                for (target, value) in color.iter_mut().zip(values) {
                    *target = value.as_f64().ok_or_else(|| {
                        NativeError::new("Canvas drop-shadow color component is not a number")
                    })? as f32;
                }
                return Ok(CanvasFilterOperation::DropShadow {
                    shadow: CanvasShadowStyle {
                        color,
                        blur: number("blur")?,
                        offset_x: number("offsetX")?,
                        offset_y: number("offsetY")?,
                    },
                    input,
                });
            }
            if let Some(functions) = operation
                .get("componentTransfer")
                .and_then(serde_json::Value::as_array)
            {
                if functions.len() != 4 {
                    return Err(NativeError::new(
                        "Canvas component-transfer filter must have four channel functions",
                    ));
                }
                let mut tables = Box::new([[0_u8; 256]; 4]);
                for (table, function) in tables.iter_mut().zip(functions) {
                    *table = canvas_component_transfer_table(function)?;
                }
                return Ok(CanvasFilterOperation::ComponentTransfer { tables, input });
            }
            if let Some(morphology) = operation.get("morphology") {
                let operator = morphology
                    .get("operator")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| NativeError::new("Canvas morphology filter has no operator"))?;
                let dilate = match operator {
                    "dilate" => true,
                    "erode" => false,
                    _ => {
                        return Err(NativeError::new(
                            "Canvas morphology filter has an invalid operator",
                        ));
                    }
                };
                let values = morphology
                    .get("radius")
                    .and_then(serde_json::Value::as_array)
                    .ok_or_else(|| NativeError::new("Canvas morphology filter has no radius"))?;
                if values.len() != 2 {
                    return Err(NativeError::new(
                        "Canvas morphology filter must have two radii",
                    ));
                }
                let mut radius = [0.0_f32; 2];
                for (target, value) in radius.iter_mut().zip(values) {
                    *target = value
                        .as_f64()
                        .filter(|value| *value >= 0.0)
                        .ok_or_else(|| {
                            NativeError::new(
                                "Canvas morphology radius must be a non-negative number",
                            )
                        })? as f32;
                }
                return Ok(CanvasFilterOperation::Morphology {
                    dilate,
                    radius_x: radius[0],
                    radius_y: radius[1],
                    input,
                });
            }
            if let Some(values) = operation.get("flood").and_then(serde_json::Value::as_array) {
                if values.len() != 4 {
                    return Err(NativeError::new(
                        "Canvas flood color must have four components",
                    ));
                }
                let mut color = [0.0_f32; 4];
                for (target, value) in color.iter_mut().zip(values) {
                    *target = value
                        .as_f64()
                        .filter(|value| value.is_finite())
                        .ok_or_else(|| {
                            NativeError::new("Canvas flood color component is not finite")
                        })? as f32;
                }
                return Ok(CanvasFilterOperation::Flood { color });
            }
            if let Some(convolution) = operation.get("convolveMatrix") {
                let order = convolution
                    .get("order")
                    .and_then(serde_json::Value::as_array)
                    .ok_or_else(|| NativeError::new("Canvas convolution has no order"))?;
                if order.len() != 2 {
                    return Err(NativeError::new(
                        "Canvas convolution order must have two entries",
                    ));
                }
                let width = order[0]
                    .as_i64()
                    .and_then(|value| i32::try_from(value).ok())
                    .filter(|value| (1..=64).contains(value))
                    .ok_or_else(|| NativeError::new("Canvas convolution width is invalid"))?;
                let height = order[1]
                    .as_i64()
                    .and_then(|value| i32::try_from(value).ok())
                    .filter(|value| (1..=64).contains(value))
                    .ok_or_else(|| NativeError::new("Canvas convolution height is invalid"))?;
                let values = convolution
                    .get("kernel")
                    .and_then(serde_json::Value::as_array)
                    .ok_or_else(|| NativeError::new("Canvas convolution has no kernel"))?;
                if values.len() != (width * height) as usize {
                    return Err(NativeError::new(
                        "Canvas convolution kernel size does not match its order",
                    ));
                }
                let mut kernel = values
                    .iter()
                    .map(|value| {
                        value
                            .as_f64()
                            .filter(|value| value.is_finite())
                            .map(|value| value as f32)
                            .ok_or_else(|| {
                                NativeError::new("Canvas convolution kernel entry is not finite")
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                // SVG defines the kernel rotated 180 degrees relative to the source image.
                kernel.reverse();
                let finite_number = |key: &str| {
                    convolution
                        .get(key)
                        .and_then(serde_json::Value::as_f64)
                        .filter(|value| value.is_finite())
                        .map(|value| value as f32)
                        .ok_or_else(|| {
                            NativeError::new(format!("Canvas convolution {key} is not finite"))
                        })
                };
                let target = convolution
                    .get("target")
                    .and_then(serde_json::Value::as_array)
                    .ok_or_else(|| NativeError::new("Canvas convolution has no target"))?;
                if target.len() != 2 {
                    return Err(NativeError::new(
                        "Canvas convolution target must have two entries",
                    ));
                }
                let target_x = target[0]
                    .as_i64()
                    .and_then(|value| i32::try_from(value).ok())
                    .filter(|value| (0..width).contains(value))
                    .ok_or_else(|| NativeError::new("Canvas convolution targetX is invalid"))?;
                let target_y = target[1]
                    .as_i64()
                    .and_then(|value| i32::try_from(value).ok())
                    .filter(|value| (0..height).contains(value))
                    .ok_or_else(|| NativeError::new("Canvas convolution targetY is invalid"))?;
                let edge_mode = convolution
                    .get("edgeMode")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| NativeError::new("Canvas convolution has no edge mode"))?;
                if !matches!(edge_mode, "duplicate" | "wrap" | "none") {
                    return Err(NativeError::new("Canvas convolution edge mode is invalid"));
                }
                let convolve_alpha = convolution
                    .get("convolveAlpha")
                    .and_then(serde_json::Value::as_bool)
                    .ok_or_else(|| {
                        NativeError::new("Canvas convolution has no convolveAlpha flag")
                    })?;
                return Ok(CanvasFilterOperation::ConvolveMatrix {
                    width,
                    height,
                    kernel,
                    gain: finite_number("gain")?,
                    bias: finite_number("bias")?,
                    target_x,
                    target_y,
                    edge_mode: edge_mode.to_owned(),
                    convolve_alpha,
                    input,
                });
            }
            if let Some(displacement) = operation.get("displacementMap") {
                let scale = displacement
                    .get("scale")
                    .and_then(serde_json::Value::as_f64)
                    .filter(|value| value.is_finite())
                    .ok_or_else(|| NativeError::new("Canvas displacement scale is not finite"))?
                    as f32;
                let channel = |key: &str| {
                    displacement
                        .get(key)
                        .and_then(serde_json::Value::as_str)
                        .filter(|value| matches!(*value, "R" | "G" | "B" | "A"))
                        .map(str::to_owned)
                        .ok_or_else(|| {
                            NativeError::new(format!(
                                "Canvas displacement {key} selector is invalid"
                            ))
                        })
                };
                return Ok(CanvasFilterOperation::DisplacementMap {
                    scale,
                    x_channel: channel("xChannel")?,
                    y_channel: channel("yChannel")?,
                    input,
                    input2: required_canvas_filter_input(operation, "input2", operation_index)?,
                });
            }
            if let Some(lighting) = operation.get("lighting") {
                let finite_number = |value: &serde_json::Value, key: &str| {
                    value
                        .get(key)
                        .and_then(serde_json::Value::as_f64)
                        .filter(|value| value.is_finite())
                        .map(|value| value as f32)
                        .ok_or_else(|| {
                            NativeError::new(format!("Canvas lighting {key} is not finite"))
                        })
                };
                let vector = |value: &serde_json::Value, key: &str| {
                    let values = value
                        .get(key)
                        .and_then(serde_json::Value::as_array)
                        .ok_or_else(|| {
                            NativeError::new(format!("Canvas lighting has no {key} vector"))
                        })?;
                    if values.len() != 3 {
                        return Err(NativeError::new(format!(
                            "Canvas lighting {key} vector must have three entries"
                        )));
                    }
                    let mut result = [0.0_f32; 3];
                    for (target, value) in result.iter_mut().zip(values) {
                        *target = value
                            .as_f64()
                            .filter(|value| value.is_finite())
                            .ok_or_else(|| {
                                NativeError::new(format!(
                                    "Canvas lighting {key} vector entry is not finite"
                                ))
                            })? as f32;
                    }
                    Ok(result)
                };
                let color = vector(lighting, "color")?;
                let specular = lighting
                    .get("specular")
                    .and_then(serde_json::Value::as_bool)
                    .ok_or_else(|| NativeError::new("Canvas lighting has no specular flag"))?;
                let surface_scale = finite_number(lighting, "surfaceScale")?;
                let constant = finite_number(lighting, "constant")?;
                let exponent = finite_number(lighting, "exponent")?;
                if constant < 0.0 || !(1.0..=128.0).contains(&exponent) {
                    return Err(NativeError::new("Canvas lighting parameters are invalid"));
                }
                let source = lighting
                    .get("light")
                    .ok_or_else(|| NativeError::new("Canvas lighting has no light source"))?;
                let light = match source.get("kind").and_then(serde_json::Value::as_str) {
                    Some("distant") => CanvasLightSource::Distant {
                        azimuth: finite_number(source, "azimuth")?,
                        elevation: finite_number(source, "elevation")?,
                    },
                    Some("point") => CanvasLightSource::Point {
                        position: vector(source, "position")?,
                    },
                    Some("spot") => {
                        let falloff_exponent = finite_number(source, "falloffExponent")?;
                        let cutoff_angle = finite_number(source, "cutoffAngle")?;
                        if !(1.0..=128.0).contains(&falloff_exponent)
                            || !(0.0..=90.0).contains(&cutoff_angle)
                        {
                            return Err(NativeError::new(
                                "Canvas spot-light parameters are invalid",
                            ));
                        }
                        CanvasLightSource::Spot {
                            position: vector(source, "position")?,
                            target: vector(source, "target")?,
                            falloff_exponent,
                            cutoff_angle,
                        }
                    }
                    _ => {
                        return Err(NativeError::new(
                            "Canvas lighting has an invalid light source",
                        ));
                    }
                };
                return Ok(CanvasFilterOperation::Lighting {
                    specular,
                    color,
                    surface_scale,
                    constant,
                    exponent,
                    light,
                    input,
                });
            }
            if let Some(mode) = operation.get("blend").and_then(serde_json::Value::as_str) {
                return Ok(CanvasFilterOperation::Blend {
                    mode: mode.to_owned(),
                    input,
                    input2: required_canvas_filter_input(operation, "input2", operation_index)?,
                });
            }
            if let Some(composite) = operation.get("composite") {
                let operator = composite
                    .get("operator")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| NativeError::new("Canvas composite filter has no operator"))?;
                let values = composite
                    .get("coefficients")
                    .and_then(serde_json::Value::as_array)
                    .ok_or_else(|| {
                        NativeError::new("Canvas composite filter has no coefficients")
                    })?;
                if values.len() != 4 {
                    return Err(NativeError::new(
                        "Canvas composite filter must have four coefficients",
                    ));
                }
                let mut coefficients = [0.0_f32; 4];
                for (target, value) in coefficients.iter_mut().zip(values) {
                    *target = value.as_f64().ok_or_else(|| {
                        NativeError::new("Canvas composite coefficient is not a number")
                    })? as f32;
                }
                return Ok(CanvasFilterOperation::Composite {
                    operator: operator.to_owned(),
                    coefficients,
                    input,
                    input2: required_canvas_filter_input(operation, "input2", operation_index)?,
                });
            }
            if let Some(values) = operation.get("merge").and_then(serde_json::Value::as_array) {
                let inputs = values
                    .iter()
                    .map(|value| canvas_filter_input_value(value, operation_index))
                    .collect::<Result<Vec<_>, _>>()?;
                if inputs.is_empty() {
                    return Err(NativeError::new("Canvas merge filter has no inputs"));
                }
                return Ok(CanvasFilterOperation::Merge(inputs));
            }
            if let Some(values) = operation
                .get("matrix")
                .and_then(serde_json::Value::as_array)
            {
                if values.len() != 20 {
                    return Err(NativeError::new(
                        "Canvas color filter matrix must have twenty entries",
                    ));
                }
                let mut matrix = [0.0; 20];
                for (target, value) in matrix.iter_mut().zip(values) {
                    *target = value.as_f64().ok_or_else(|| {
                        NativeError::new("Canvas filter matrix entry is not a number")
                    })? as f32;
                }
                return Ok(CanvasFilterOperation::ColorMatrix { matrix, input });
            }
            Err(NativeError::new("invalid Canvas filter operation"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CanvasDrawEffects {
        shadow: CanvasShadowStyle {
            color,
            blur: number("blur")?,
            offset_x: number("offsetX")?,
            offset_y: number("offsetY")?,
        },
        filters,
    })
}

fn required_canvas_filter_input(
    operation: &serde_json::Value,
    key: &str,
    operation_index: usize,
) -> Result<CanvasFilterInput, NativeError> {
    let value = operation
        .get(key)
        .ok_or_else(|| NativeError::new(format!("Canvas filter operation has no {key}")))?;
    canvas_filter_input_value(value, operation_index)
}

fn canvas_filter_input_value(
    value: &serde_json::Value,
    operation_index: usize,
) -> Result<CanvasFilterInput, NativeError> {
    let value = value
        .as_i64()
        .ok_or_else(|| NativeError::new("Canvas filter input is not an integer"))?;
    if value == -1 {
        return Ok(CanvasFilterInput::SourceGraphic);
    }
    let index =
        usize::try_from(value).map_err(|_| NativeError::new("Canvas filter input is invalid"))?;
    if index >= operation_index {
        return Err(NativeError::new(
            "Canvas filter input must refer to an earlier operation",
        ));
    }
    Ok(CanvasFilterInput::Operation(index))
}

fn canvas_component_transfer_table(function: &serde_json::Value) -> Result<[u8; 256], NativeError> {
    enum Parameters {
        Identity,
        Table(Vec<f64>),
        Discrete(Vec<f64>),
        Linear {
            slope: f64,
            intercept: f64,
        },
        Gamma {
            amplitude: f64,
            exponent: f64,
            offset: f64,
        },
    }

    let function_type = function
        .get("type")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| NativeError::new("Canvas component-transfer function has no type"))?;
    let number = |key: &str| {
        function
            .get(key)
            .and_then(serde_json::Value::as_f64)
            .ok_or_else(|| {
                NativeError::new(format!(
                    "Canvas component-transfer function has no numeric {key}"
                ))
            })
    };
    let parse_values = || {
        let values = function
            .get("values")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| {
                NativeError::new("Canvas component-transfer function has no table values")
            })?
            .iter()
            .map(|value| {
                value.as_f64().ok_or_else(|| {
                    NativeError::new("Canvas component-transfer table value is not a number")
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if values.is_empty() {
            return Err(NativeError::new("Canvas component-transfer table is empty"));
        }
        Ok(values)
    };
    let parameters = match function_type {
        "identity" => Parameters::Identity,
        "table" => Parameters::Table(parse_values()?),
        "discrete" => Parameters::Discrete(parse_values()?),
        "linear" => Parameters::Linear {
            slope: number("slope")?,
            intercept: number("intercept")?,
        },
        "gamma" => Parameters::Gamma {
            amplitude: number("amplitude")?,
            exponent: number("exponent")?,
            offset: number("offset")?,
        },
        _ => {
            return Err(NativeError::new(
                "Canvas component-transfer function has an invalid type",
            ));
        }
    };
    let mut table = [0_u8; 256];
    for (index, target) in table.iter_mut().enumerate() {
        let input = index as f64 / 255.0;
        let output = match &parameters {
            Parameters::Identity => input,
            Parameters::Table(values) => {
                if values.len() == 1 {
                    values[0]
                } else {
                    let position = input * (values.len() - 1) as f64;
                    let lower = position.floor() as usize;
                    let upper = (lower + 1).min(values.len() - 1);
                    values[lower] + (values[upper] - values[lower]) * position.fract()
                }
            }
            Parameters::Discrete(values) => {
                values[((input * values.len() as f64).floor() as usize).min(values.len() - 1)]
            }
            Parameters::Linear { slope, intercept } => slope * input + intercept,
            Parameters::Gamma {
                amplitude,
                exponent,
                offset,
            } => amplitude * input.powf(*exponent) + offset,
        };
        *target = if output.is_nan() {
            0
        } else {
            (output.clamp(0.0, 1.0) * 255.0).round() as u8
        };
    }
    Ok(table)
}


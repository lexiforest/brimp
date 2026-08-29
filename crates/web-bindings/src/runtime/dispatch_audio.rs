use super::*;

pub(super) fn dispatch(
    state: &BindingState,
    call: &NativeCall<'_>,
    operation: &str,
) -> Result<NativeValue, NativeError> {
    match operation {
        "audioOutputEnabled" => Ok(NativeValue::Boolean(state.features.webaudio_output)),
        "audioCreateOffline" => {
            if !state.features.webaudio {
                return Err(NativeError::new("WebAudio is disabled"));
            }
            let channels = required_u32(call, 2, "number of channels")? as usize;
            let length = required_u64(call, 3, "render length")? as usize;
            let sample_rate = required_number(call, 4, "sample rate")? as f32;
            let id = state
                .audio
                .borrow_mut()
                .create_offline(channels, length, sample_rate)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(id as f64))
        }
        "audioCreateRealtime" => {
            if !state.features.webaudio {
                return Err(NativeError::new("WebAudio is disabled"));
            }
            let requested_rate = required_number(call, 2, "sample rate")? as f32;
            let sink_id = required_string(call, 3, "audio sink id")?;
            if sink_id != "none" && !state.features.webaudio_output {
                return Err(NativeError::new("WebAudio hardware output is disabled"));
            }
            let (id, sample_rate, base_latency, output_latency, sink_id) = state
                .audio
                .borrow_mut()
                .create_realtime((requested_rate > 0.0).then_some(requested_rate), sink_id)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(
                serde_json::json!({
                    "id": id,
                    "sampleRate": sample_rate,
                    "baseLatency": base_latency,
                    "outputLatency": output_latency,
                    "sinkId": sink_id,
                })
                .to_string(),
            ))
        }
        "audioRealtimeState" => {
            let context = required_u64(call, 2, "audio context")?;
            let (context_state, current_time, output_latency) = state
                .audio
                .borrow()
                .realtime_state(context)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(
                serde_json::json!({
                    "state": context_state,
                    "currentTime": current_time,
                    "outputLatency": output_latency,
                })
                .to_string(),
            ))
        }
        "audioPlaybackStats" => {
            let context = required_u64(call, 2, "audio context")?;
            let (
                underrun_duration,
                underrun_events,
                total_duration,
                average_latency,
                minimum_latency,
                maximum_latency,
            ) = state
                .audio
                .borrow()
                .realtime_playback_stats(context)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(
                serde_json::json!({
                    "underrunDuration": underrun_duration,
                    "underrunEvents": underrun_events,
                    "totalDuration": total_duration,
                    "averageLatency": average_latency,
                    "minimumLatency": minimum_latency,
                    "maximumLatency": maximum_latency,
                })
                .to_string(),
            ))
        }
        "audioResetPlaybackLatency" => {
            let context = required_u64(call, 2, "audio context")?;
            state
                .audio
                .borrow()
                .reset_realtime_playback_latency(context)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioTakeEnded" => {
            let context = required_u64(call, 2, "audio context")?;
            let nodes = state
                .audio
                .borrow()
                .take_ended_nodes(context)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(
                serde_json::to_string(&nodes).expect("audio node ids are serializable"),
            ))
        }
        "audioTakeProcessingEvents" => {
            let context = required_u64(call, 2, "audio context")?;
            let events = state
                .audio
                .borrow_mut()
                .take_processing_events(context)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(events))
        }
        "audioCompleteProcessingEvent" => {
            let context = required_u64(call, 2, "audio context")?;
            let event = required_u64(call, 3, "audio processing event")?;
            let output = required_u64(call, 4, "output AudioBuffer")?;
            state
                .audio
                .borrow_mut()
                .complete_processing_event(context, event, output)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioControlRealtime" => {
            let context = required_u64(call, 2, "audio context")?;
            let operation = required_string(call, 3, "AudioContext lifecycle operation")?;
            state
                .audio
                .borrow_mut()
                .control_realtime(context, &operation)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioCreateNode" => {
            let context = required_u64(call, 2, "audio context")?;
            let kind = required_string(call, 3, "audio node type")?;
            let option = required_number(call, 4, "audio node option")?;
            let id = state
                .audio
                .borrow_mut()
                .create_node(context, &kind, option)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(id as f64))
        }
        "audioCreateScriptProcessor" => {
            let context = required_u64(call, 2, "audio context")?;
            let buffer_size = required_u32(call, 3, "script processor buffer size")? as usize;
            let input_channels = required_u32(call, 4, "script processor input channels")? as usize;
            let output_channels =
                required_u32(call, 5, "script processor output channels")? as usize;
            let (id, buffer_size) = state
                .audio
                .borrow_mut()
                .create_script_processor(context, buffer_size, input_channels, output_channels)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(
                serde_json::json!({ "id": id, "bufferSize": buffer_size }).to_string(),
            ))
        }
        "audioRegisterWorkletModule" => {
            let context = required_u64(call, 2, "audio context")?;
            let source = required_string(call, 3, "AudioWorklet module source")?;
            state
                .audio
                .borrow_mut()
                .register_worklet_module(context, source)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioCreateWorkletNode" => {
            let context = required_u64(call, 2, "audio context")?;
            let name = required_string(call, 3, "AudioWorklet processor name")?;
            let encoded = required_string(call, 4, "AudioWorklet node options")?;
            let request = serde_json::from_str(&encoded).map_err(|error| {
                NativeError::new(format!("invalid AudioWorklet options: {error}"))
            })?;
            let (id, descriptors) = state
                .audio
                .borrow_mut()
                .create_worklet_node(context, &name, request)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(
                serde_json::json!({ "id": id, "descriptors": descriptors }).to_string(),
            ))
        }
        "audioPostWorkletMessage" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "AudioWorklet node")?;
            let data = required_string(call, 4, "AudioWorklet message")?;
            state
                .audio
                .borrow()
                .post_worklet_message(context, node, data)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioTakeWorkletMessages" => {
            let context = required_u64(call, 2, "audio context")?;
            let messages = state
                .audio
                .borrow()
                .take_worklet_messages(context)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(messages))
        }
        "audioMediaStreamTrackState" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "MediaStream destination node")?;
            let state = state
                .audio
                .borrow()
                .media_stream_track_state(context, node)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(state.to_owned()))
        }
        "audioCreateMediaStreamSource" => {
            let context = required_u64(call, 2, "audio context")?;
            let source_context = required_u64(call, 3, "MediaStream audio context")?;
            let source_node = required_u64(call, 4, "MediaStream destination node")?;
            let track_only = required_boolean(call, 5, "MediaStreamTrack source selection")?;
            let id = state
                .audio
                .borrow_mut()
                .create_media_stream_source(context, source_context, source_node, track_only)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(id as f64))
        }
        "audioStopMediaStreamTrack" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "MediaStream destination node")?;
            state
                .audio
                .borrow_mut()
                .stop_media_stream_track(context, node)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioCreateIirFilter" => {
            let context = required_u64(call, 2, "audio context")?;
            let feedforward_bytes = call
                .argument(3)
                .ok_or_else(|| NativeError::new("missing feedforward coefficients"))?
                .to_bytes()?;
            let feedback_bytes = call
                .argument(4)
                .ok_or_else(|| NativeError::new("missing feedback coefficients"))?
                .to_bytes()?;
            if feedforward_bytes.len() % size_of::<f64>() != 0
                || feedback_bytes.len() % size_of::<f64>() != 0
            {
                return Err(NativeError::new("IIR coefficients are not Float64-aligned"));
            }
            let decode = |bytes: Vec<u8>| {
                bytes
                    .chunks_exact(size_of::<f64>())
                    .map(|bytes| {
                        f64::from_ne_bytes(bytes.try_into().expect("eight-byte f64 chunk"))
                    })
                    .collect::<Vec<_>>()
            };
            let id = state
                .audio
                .borrow_mut()
                .create_iir_filter(context, decode(feedforward_bytes), decode(feedback_bytes))
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(id as f64))
        }
        "audioCreatePeriodicWave" => {
            let context = required_u64(call, 2, "audio context")?;
            let real_bytes = call
                .argument(3)
                .ok_or_else(|| NativeError::new("missing PeriodicWave real coefficients"))?
                .to_bytes()?;
            let imag_bytes = call
                .argument(4)
                .ok_or_else(|| NativeError::new("missing PeriodicWave imaginary coefficients"))?
                .to_bytes()?;
            let disable_normalization =
                required_boolean(call, 5, "PeriodicWave normalization option")?;
            if real_bytes.len() % size_of::<f32>() != 0 || imag_bytes.len() % size_of::<f32>() != 0
            {
                return Err(NativeError::new(
                    "PeriodicWave coefficients are not Float32-aligned",
                ));
            }
            let decode = |bytes: Vec<u8>| {
                bytes
                    .chunks_exact(size_of::<f32>())
                    .map(|bytes| f32::from_ne_bytes(bytes.try_into().expect("four-byte f32 chunk")))
                    .collect::<Vec<_>>()
            };
            let id = state
                .audio
                .borrow_mut()
                .create_periodic_wave(
                    context,
                    decode(real_bytes),
                    decode(imag_bytes),
                    disable_normalization,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(id as f64))
        }
        "audioConnect" => {
            let context = required_u64(call, 2, "audio context")?;
            let source = required_u64(call, 3, "source node")?;
            let destination = required_u64(call, 4, "destination node")?;
            let output = required_u32(call, 5, "audio output index")? as usize;
            let input = required_u32(call, 6, "audio input index")? as usize;
            state
                .audio
                .borrow_mut()
                .connect(context, source, destination, output, input)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioDisconnect" => {
            let context = required_u64(call, 2, "audio context")?;
            let source = required_u64(call, 3, "source node")?;
            let has_destination = required_boolean(call, 4, "has destination")?;
            let destination = required_u64(call, 5, "destination node")?;
            state
                .audio
                .borrow_mut()
                .disconnect(context, source, has_destination.then_some(destination))
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioNodeChannelConfig" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "audio node")?;
            let (count, mode, interpretation) = state
                .audio
                .borrow()
                .node_channel_config(context, node)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(
                serde_json::json!({
                    "count": count,
                    "mode": mode,
                    "interpretation": interpretation,
                })
                .to_string(),
            ))
        }
        "audioSetNodeChannelCount" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "audio node")?;
            let count = required_u32(call, 4, "audio channel count")? as usize;
            state
                .audio
                .borrow()
                .set_node_channel_count(context, node, count)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioSetNodeChannelCountMode" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "audio node")?;
            let mode = required_string(call, 4, "audio channel count mode")?;
            state
                .audio
                .borrow()
                .set_node_channel_count_mode(context, node, &mode)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioSetNodeChannelInterpretation" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "audio node")?;
            let interpretation = required_string(call, 4, "audio channel interpretation")?;
            state
                .audio
                .borrow()
                .set_node_channel_interpretation(context, node, &interpretation)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioGetParamAutomationRate" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "audio node")?;
            let name = required_string(call, 4, "audio parameter")?;
            let rate = state
                .audio
                .borrow()
                .param_automation_rate(context, node, &name)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(rate.to_owned()))
        }
        "audioSetParamAutomationRate" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "audio node")?;
            let name = required_string(call, 4, "audio parameter")?;
            let rate = required_string(call, 5, "audio automation rate")?;
            state
                .audio
                .borrow_mut()
                .set_param_automation_rate(context, node, &name, &rate)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioSetParam" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "audio node")?;
            let name = required_string(call, 4, "audio parameter")?;
            let value = required_number(call, 5, "audio parameter value")? as f32;
            state
                .audio
                .borrow_mut()
                .set_param(context, node, &name, value)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioScheduleParam" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "audio node")?;
            let name = required_string(call, 4, "audio parameter")?;
            let operation = required_string(call, 5, "automation operation")?;
            let value = required_number(call, 6, "automation value")? as f32;
            let time = required_number(call, 7, "automation time")?;
            let extra = required_number(call, 8, "automation extra value")?;
            state
                .audio
                .borrow_mut()
                .schedule_param(context, node, &name, &operation, value, time, extra)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioScheduleParamCurve" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "audio node")?;
            let name = required_string(call, 4, "audio parameter")?;
            let values = required_f32_array(call, 5, "audio parameter value curve")?;
            let start_time = required_number(call, 6, "automation start time")?;
            let duration = required_number(call, 7, "automation duration")?;
            state
                .audio
                .borrow_mut()
                .schedule_param_curve(context, node, &name, &values, start_time, duration)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioGetParam" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "audio node")?;
            let name = required_string(call, 4, "audio parameter")?;
            let value = state
                .audio
                .borrow()
                .get_param(context, node, &name)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(f64::from(value)))
        }
        "audioSetOscillatorType" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "oscillator node")?;
            let oscillator_type = required_string(call, 4, "oscillator type")?;
            state
                .audio
                .borrow_mut()
                .set_oscillator_type(context, node, &oscillator_type)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioSetPeriodicWave" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "oscillator node")?;
            let wave = required_u64(call, 4, "periodic wave")?;
            state
                .audio
                .borrow_mut()
                .set_periodic_wave(context, node, wave)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioSetBiquadType" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "biquad filter node")?;
            let filter_type = required_string(call, 4, "biquad filter type")?;
            state
                .audio
                .borrow_mut()
                .set_biquad_type(context, node, &filter_type)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioSetWaveShaperCurve" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "wave shaper node")?;
            let bytes = call
                .argument(4)
                .ok_or_else(|| NativeError::new("missing WaveShaper curve"))?
                .to_bytes()?;
            if bytes.len() % size_of::<f32>() != 0 {
                return Err(NativeError::new("WaveShaper curve is not Float32-aligned"));
            }
            let curve = bytes
                .chunks_exact(size_of::<f32>())
                .map(|bytes| f32::from_ne_bytes(bytes.try_into().expect("four-byte f32 chunk")))
                .collect();
            state
                .audio
                .borrow_mut()
                .configure_wave_shaper(context, node, Some(curve), None)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioSetWaveShaperOversample" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "wave shaper node")?;
            let oversample = required_string(call, 4, "WaveShaper oversample value")?;
            state
                .audio
                .borrow_mut()
                .configure_wave_shaper(context, node, None, Some(&oversample))
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioConfigureConvolver" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "convolver node")?;
            let property = required_string(call, 4, "convolver property")?;
            match property.as_str() {
                "buffer" => {
                    let buffer = required_u64(call, 5, "convolver buffer")?;
                    state
                        .audio
                        .borrow_mut()
                        .configure_convolver(context, node, (buffer != 0).then_some(buffer), None)
                        .map_err(NativeError::new)?;
                }
                "normalize" => {
                    let normalize = required_boolean(call, 5, "convolver normalize")?;
                    state
                        .audio
                        .borrow_mut()
                        .configure_convolver(context, node, None, Some(normalize))
                        .map_err(NativeError::new)?;
                }
                _ => return Err(NativeError::new("unknown ConvolverNode property")),
            }
            Ok(NativeValue::Undefined)
        }
        "audioSetPannerNumber" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "panner node")?;
            let property = required_string(call, 4, "panner property")?;
            let value = required_number(call, 5, "panner value")?;
            state
                .audio
                .borrow_mut()
                .set_panner_number(context, node, &property, value)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioSetPannerModel" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "panner node")?;
            let property = required_string(call, 4, "panner property")?;
            let value = required_string(call, 5, "panner model")?;
            state
                .audio
                .borrow_mut()
                .set_panner_model(context, node, &property, &value)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioIirFrequencyResponse" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "IIR filter node")?;
            let bytes = call
                .argument(4)
                .ok_or_else(|| NativeError::new("missing IIR frequencies"))?
                .to_bytes()?;
            let magnitude = required_boolean(call, 5, "magnitude response")?;
            if bytes.len() % size_of::<f32>() != 0 {
                return Err(NativeError::new("IIR frequencies are not Float32-aligned"));
            }
            let frequencies = bytes
                .chunks_exact(size_of::<f32>())
                .map(|bytes| f32::from_ne_bytes(bytes.try_into().expect("four-byte f32 chunk")))
                .collect::<Vec<_>>();
            let (magnitudes, phases) = state
                .audio
                .borrow()
                .iir_frequency_response(context, node, &frequencies)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Float32Array(if magnitude {
                magnitudes
            } else {
                phases
            }))
        }
        "audioBiquadFrequencyResponse" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "biquad filter node")?;
            let bytes = call
                .argument(4)
                .ok_or_else(|| NativeError::new("missing biquad frequencies"))?
                .to_bytes()?;
            let magnitude = required_boolean(call, 5, "magnitude response")?;
            if bytes.len() % size_of::<f32>() != 0 {
                return Err(NativeError::new(
                    "biquad frequencies are not Float32-aligned",
                ));
            }
            let frequencies = bytes
                .chunks_exact(size_of::<f32>())
                .map(|bytes| f32::from_ne_bytes(bytes.try_into().expect("four-byte f32 chunk")))
                .collect::<Vec<_>>();
            let (magnitudes, phases) = state
                .audio
                .borrow()
                .biquad_frequency_response(context, node, &frequencies)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Float32Array(if magnitude {
                magnitudes
            } else {
                phases
            }))
        }
        "audioSetBufferSource" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "buffer source node")?;
            let buffer = required_u64(call, 4, "audio buffer")?;
            state
                .audio
                .borrow_mut()
                .set_buffer_source_buffer(context, node, (buffer != 0).then_some(buffer))
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioConfigureBufferSource" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "buffer source node")?;
            let property = required_string(call, 4, "buffer source property")?;
            let value = required_number(call, 5, "buffer source value")?;
            state
                .audio
                .borrow_mut()
                .configure_buffer_source(context, node, &property, value)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioStart" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "audio node")?;
            let when = required_number(call, 4, "start time")?;
            state
                .audio
                .borrow_mut()
                .start(context, node, when)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioStartBufferSource" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "buffer source node")?;
            let when = required_number(call, 4, "start time")?;
            let offset = required_number(call, 5, "buffer offset")?;
            let duration = required_number(call, 6, "buffer duration")?;
            state
                .audio
                .borrow_mut()
                .start_buffer_source(
                    context,
                    node,
                    when,
                    offset,
                    (duration >= 0.0).then_some(duration),
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioStop" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "scheduled source node")?;
            let when = required_number(call, 4, "stop time")?;
            state
                .audio
                .borrow_mut()
                .stop(context, node, when)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioAnalyserSettings" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "analyser node")?;
            let settings = state
                .audio
                .borrow()
                .analyser_settings(context, node)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(
                serde_json::to_string(&settings).map_err(|error| {
                    NativeError::new(format!("could not encode analyser settings: {error}"))
                })?,
            ))
        }
        "audioSetAnalyser" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "analyser node")?;
            let property = required_string(call, 4, "analyser property")?;
            let value = required_number(call, 5, "analyser value")?;
            state
                .audio
                .borrow_mut()
                .set_analyser(context, node, &property, value)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioAnalyserFloatData" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "analyser node")?;
            let frequency = required_boolean(call, 4, "frequency data")?;
            let length = required_u64(call, 5, "analyser destination length")? as usize;
            let values = state
                .audio
                .borrow_mut()
                .analyser_float_data(context, node, frequency, length)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Float32Array(values))
        }
        "audioAnalyserByteData" => {
            let context = required_u64(call, 2, "audio context")?;
            let node = required_u64(call, 3, "analyser node")?;
            let frequency = required_boolean(call, 4, "frequency data")?;
            let length = required_u64(call, 5, "analyser destination length")? as usize;
            let values = state
                .audio
                .borrow_mut()
                .analyser_byte_data(context, node, frequency, length)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Bytes(values))
        }
        "audioBeginRender" => {
            let context = required_u64(call, 2, "audio context")?;
            state
                .audio
                .borrow_mut()
                .begin_offline_render(context)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioPollRender" => {
            let context = required_u64(call, 2, "audio context")?;
            let result = state
                .audio
                .borrow_mut()
                .poll_offline_render(context)
                .map_err(NativeError::new)?;
            Ok(
                result.map_or(NativeValue::Null, |(id, channels, length, sample_rate)| {
                    NativeValue::String(
                        serde_json::json!({
                            "id": id,
                            "channels": channels,
                            "length": length,
                            "sampleRate": sample_rate,
                        })
                        .to_string(),
                    )
                }),
            )
        }
        "audioScheduleSuspend" => {
            let context = required_u64(call, 2, "audio context")?;
            let suspend_time = required_number(call, 3, "offline suspension time")?;
            let suspension = state
                .audio
                .borrow_mut()
                .schedule_offline_suspend(context, suspend_time)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(suspension as f64))
        }
        "audioPollSuspend" => {
            let context = required_u64(call, 2, "audio context")?;
            let suspension = required_u64(call, 3, "offline suspension")?;
            let current_time = state
                .audio
                .borrow_mut()
                .poll_offline_suspend(context, suspension)
                .map_err(NativeError::new)?;
            Ok(current_time.map_or(NativeValue::Null, NativeValue::Number))
        }
        "audioBeginResume" => {
            let context = required_u64(call, 2, "audio context")?;
            state
                .audio
                .borrow_mut()
                .begin_offline_resume(context)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "audioPollResume" => {
            let context = required_u64(call, 2, "audio context")?;
            let resumed = state
                .audio
                .borrow_mut()
                .poll_offline_resume(context)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Boolean(resumed))
        }
        "audioChannel" => {
            let context = required_u64(call, 2, "audio context")?;
            let buffer = required_u64(call, 3, "audio buffer")?;
            let channel = required_u32(call, 4, "channel index")? as usize;
            let samples = state
                .audio
                .borrow()
                .channel(context, buffer, channel)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Float32Array(samples))
        }
        "audioCreateBuffer" => {
            let context = required_u64(call, 2, "audio context")?;
            let channels = required_u32(call, 3, "number of channels")? as usize;
            let length = required_u64(call, 4, "buffer length")? as usize;
            let sample_rate = required_number(call, 5, "sample rate")? as f32;
            let (id, channels, length, sample_rate) = state
                .audio
                .borrow_mut()
                .create_buffer(context, channels, length, sample_rate)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(
                serde_json::json!({
                    "id": id,
                    "channels": channels,
                    "length": length,
                    "sampleRate": sample_rate,
                })
                .to_string(),
            ))
        }
        "audioDecode" => {
            let context = required_u64(call, 2, "audio context")?;
            let bytes = call
                .argument(3)
                .ok_or_else(|| NativeError::new("missing encoded audio data"))?
                .to_bytes()?;
            let (id, channels, length, sample_rate) = state
                .audio
                .borrow_mut()
                .decode(context, bytes)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(
                serde_json::json!({
                    "id": id,
                    "channels": channels,
                    "length": length,
                    "sampleRate": sample_rate,
                })
                .to_string(),
            ))
        }
        "audioWriteChannel" => {
            let context = required_u64(call, 2, "audio context")?;
            let buffer = required_u64(call, 3, "audio buffer")?;
            let channel = required_u32(call, 4, "channel index")? as usize;
            let offset = required_u64(call, 5, "buffer offset")? as usize;
            let bytes = call
                .argument(6)
                .ok_or_else(|| NativeError::new("missing audio channel samples"))?
                .to_bytes()?;
            if bytes.len() % size_of::<f32>() != 0 {
                return Err(NativeError::new(
                    "audio channel data is not Float32-aligned",
                ));
            }
            let samples = bytes
                .chunks_exact(size_of::<f32>())
                .map(|bytes| f32::from_ne_bytes(bytes.try_into().expect("four-byte f32 chunk")))
                .collect::<Vec<_>>();
            state
                .audio
                .borrow_mut()
                .write_channel(context, buffer, channel, offset, &samples)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        _ => Err(NativeError::new(format!(
            "unknown native WebAudio operation: {operation}"
        ))),
    }
}

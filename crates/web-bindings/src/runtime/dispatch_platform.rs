use super::*;

pub(super) fn dispatch(
    state: &BindingState,
    call: &NativeCall<'_>,
    operation: &str,
) -> Result<NativeValue, NativeError> {
    match operation {
        "workerCreate" => {
            if !state.features.worker_system {
                return Err(NativeError::new("worker system is disabled"));
            }
            let url = required_string(call, 2, "worker script URL")?;
            let kind = required_string(call, 3, "worker kind")?;
            let name = required_string(call, 4, "worker name")?;
            let scope = required_string(call, 5, "worker scope")?;
            let mut workers = state.workers.borrow_mut();
            let id = workers.next_id;
            workers.next_id = workers.next_id.wrapping_add(1).max(1);
            workers.pending.push_back(PendingWorkerOperation::Create {
                id,
                url,
                kind,
                name,
                scope,
            });
            Ok(NativeValue::Number(id as f64))
        }
        "workerPost" => {
            let id = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing worker id"))?
                .to_number()? as u64;
            let message_json = required_string(call, 3, "worker message")?;
            state
                .workers
                .borrow_mut()
                .pending
                .push_back(PendingWorkerOperation::Post { id, message_json });
            Ok(NativeValue::Undefined)
        }
        "workerTerminate" => {
            let id = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing worker id"))?
                .to_number()? as u64;
            let mut workers = state.workers.borrow_mut();
            workers
                .pending
                .push_back(PendingWorkerOperation::Terminate { id });
            Ok(NativeValue::Undefined)
        }
        "workerUnregister" => {
            let id = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing worker id"))?
                .to_number()? as u64;
            state
                .workers
                .borrow_mut()
                .pending
                .push_back(PendingWorkerOperation::Unregister { id });
            Ok(NativeValue::Undefined)
        }
        "webSocketCreate" => {
            if !state.features.streaming_networking {
                return Err(NativeError::new("streaming networking is disabled"));
            }
            let url = required_string(call, 2, "WebSocket URL")?;
            let mut streaming = state.streaming.borrow_mut();
            let id = streaming.next_id;
            streaming.next_id = streaming.next_id.wrapping_add(1).max(1);
            streaming
                .pending
                .push_back(PendingWebSocketOperation::Create { id, url });
            Ok(NativeValue::Number(id as f64))
        }
        "webSocketSend" => {
            let id = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing WebSocket id"))?
                .to_number()? as u64;
            let message = required_string(call, 3, "WebSocket message")?;
            state
                .streaming
                .borrow_mut()
                .pending
                .push_back(PendingWebSocketOperation::SendText { id, message });
            Ok(NativeValue::Undefined)
        }
        "webSocketClose" => {
            let id = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing WebSocket id"))?
                .to_number()? as u64;
            state
                .streaming
                .borrow_mut()
                .pending
                .push_back(PendingWebSocketOperation::Close { id });
            Ok(NativeValue::Undefined)
        }
        "fetchStreamCancel" => {
            let id = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing Fetch stream id"))?
                .to_number()? as u64;
            state
                .streaming
                .borrow_mut()
                .pending
                .push_back(PendingWebSocketOperation::CancelFetch { id });
            Ok(NativeValue::Undefined)
        }
        "persistentList" => {
            let namespace = required_string(call, 2, "storage namespace")?;
            let storage = persistent_storage(state)?;
            let origin = storage_origin(state)?;
            let entries = storage.list(&origin, &namespace).map_err(err)?;
            Ok(NativeValue::String(
                serde_json::to_string(&entries).map_err(err)?,
            ))
        }
        "persistentGet" => {
            let namespace = required_string(call, 2, "storage namespace")?;
            let key = required_string(call, 3, "storage key")?;
            let storage = persistent_storage(state)?;
            let origin = storage_origin(state)?;
            Ok(match storage.get(&origin, &namespace, &key).map_err(err)? {
                Some(value) => NativeValue::String(value),
                None => NativeValue::Null,
            })
        }
        "persistentSet" => {
            let namespace = required_string(call, 2, "storage namespace")?;
            let key = required_string(call, 3, "storage key")?;
            let value = required_string(call, 4, "storage value")?;
            let storage = persistent_storage(state)?;
            let origin = storage_origin(state)?;
            storage
                .set(&origin, &namespace, &key, &value)
                .map_err(err)?;
            Ok(NativeValue::Undefined)
        }
        "persistentDelete" => {
            let namespace = required_string(call, 2, "storage namespace")?;
            let key = required_string(call, 3, "storage key")?;
            let storage = persistent_storage(state)?;
            let origin = storage_origin(state)?;
            storage.delete(&origin, &namespace, &key).map_err(err)?;
            Ok(NativeValue::Undefined)
        }
        "persistentClear" => {
            let namespace = required_string(call, 2, "storage namespace")?;
            let storage = persistent_storage(state)?;
            let origin = storage_origin(state)?;
            storage.clear(&origin, &namespace).map_err(err)?;
            Ok(NativeValue::Undefined)
        }
        "persistentEstimate" => {
            let storage = persistent_storage(state)?;
            let usage = match storage_origin(state) {
                Ok(origin) => storage.usage(&origin).map_err(err)?,
                Err(_) => 0,
            };
            Ok(NativeValue::String(format!(
                "{{\"usage\":{},\"quota\":{}}}",
                usage,
                storage.quota()
            )))
        }
        "setTimeout" => {
            let callback = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing timer callback"))?
                .to_function()?;
            let delay = call
                .argument(3)
                .map(|value| value.to_number())
                .transpose()?
                .unwrap_or(0.0);
            let id = state.timers.borrow_mut().schedule(delay, callback);
            Ok(NativeValue::Number(f64::from(id)))
        }
        "clearTimeout" => {
            let id = call
                .argument(2)
                .map(|value| value.to_number())
                .transpose()?
                .unwrap_or(0.0) as u32;
            state.timers.borrow_mut().clear(id);
            Ok(NativeValue::Undefined)
        }
        "queueMicrotask" => {
            let callback = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing microtask callback"))?
                .to_function()?;
            state.timers.borrow_mut().queue_microtask(callback);
            Ok(NativeValue::Undefined)
        }
        "location" => {
            let property = required_string(call, 2, "location property")?;
            let raw_url = state
                .browsing_context
                .url
                .lock()
                .expect("browsing URL lock poisoned");
            let Some(raw_url) = raw_url.as_deref() else {
                return Ok(NativeValue::String(String::new()));
            };
            let url = url::Url::parse(raw_url).map_err(err)?;
            let value = match property.as_str() {
                "href" => url.as_str().to_string(),
                "protocol" => format!("{}:", url.scheme()),
                "host" => match (url.host_str(), url.port()) {
                    (Some(host), Some(port)) => format!("{host}:{port}"),
                    (Some(host), None) => host.to_string(),
                    (None, _) => String::new(),
                },
                "hostname" => url.host_str().unwrap_or_default().to_string(),
                "port" => url.port().map(|port| port.to_string()).unwrap_or_default(),
                "pathname" => url.path().to_string(),
                "search" => url
                    .query()
                    .map(|query| format!("?{query}"))
                    .unwrap_or_default(),
                "hash" => url
                    .fragment()
                    .map(|hash| format!("#{hash}"))
                    .unwrap_or_default(),
                "origin" => url.origin().ascii_serialization(),
                _ => return Err(NativeError::new("unknown Location property")),
            };
            Ok(NativeValue::String(value))
        }
        "urlParse" => {
            let input = required_string(call, 2, "URL input")?;
            let base = required_string(call, 3, "URL base")?;
            let base = (!base.is_empty())
                .then(|| url::Url::parse(&base).map_err(err))
                .transpose()?;
            let parsed = url::Url::options()
                .base_url(base.as_ref())
                .parse(&input)
                .map_err(err)?;
            Ok(NativeValue::String(url_record_json(&parsed)?))
        }
        "urlSet" => {
            let href = required_string(call, 2, "URL href")?;
            let component = required_string(call, 3, "URL component")?;
            let value = required_string(call, 4, "URL component value")?;
            Ok(NativeValue::String(set_url_component(
                &href, &component, &value,
            )?))
        }
        "urlSearchParamsParse" => {
            let input = required_string(call, 2, "query")?;
            let pairs = url::form_urlencoded::parse(input.trim_start_matches('?').as_bytes())
                .into_owned()
                .collect::<Vec<_>>();
            Ok(NativeValue::String(
                serde_json::to_string(&pairs).map_err(err)?,
            ))
        }
        "urlSearchParamsSerialize" => {
            let input = required_string(call, 2, "query pairs")?;
            let pairs: Vec<(String, String)> = serde_json::from_str(&input).map_err(err)?;
            let output = url::form_urlencoded::Serializer::new(String::new())
                .extend_pairs(pairs)
                .finish();
            Ok(NativeValue::String(output))
        }
        "encodingCanonical" => {
            let label = required_string(call, 2, "encoding label")?;
            match encoding_rs::Encoding::for_label_no_replacement(label.as_bytes()) {
                Some(encoding) => Ok(NativeValue::String(encoding.name().to_ascii_lowercase())),
                None => Ok(NativeValue::Null),
            }
        }
        "legacyQueryEncodeBlock" => {
            let label = required_string(call, 2, "encoding label")?;
            let block_start = call
                .argument(3)
                .ok_or_else(|| NativeError::new("missing code point block"))?
                .to_number()? as u32;
            let encoding = encoding_rs::Encoding::for_label_no_replacement(label.as_bytes())
                .ok_or_else(|| NativeError::new("invalid document encoding"))?;
            let encoded = (block_start..block_start.saturating_add(256))
                .map(|code_point| {
                    char::from_u32(code_point)
                        .map(|character| legacy_query_encode(encoding, &character.to_string()))
                        .unwrap_or_else(|| "%EF%BF%BD".to_owned())
                })
                .collect::<Vec<_>>();
            Ok(NativeValue::String(
                serde_json::to_string(&encoded).map_err(err)?,
            ))
        }
        "legacyQueryEncode" => {
            let label = required_string(call, 2, "encoding label")?;
            let input = required_string(call, 3, "query input")?;
            let encoding = encoding_rs::Encoding::for_label_no_replacement(label.as_bytes())
                .ok_or_else(|| NativeError::new("invalid document encoding"))?;
            Ok(NativeValue::String(legacy_query_encode(encoding, &input)))
        }
        "formUrlEncode" => {
            let label = required_string(call, 2, "form encoding label")?;
            let input = required_string(call, 3, "form field value")?;
            let encoding = encoding_rs::Encoding::for_label_no_replacement(label.as_bytes())
                .unwrap_or(encoding_rs::UTF_8);
            Ok(NativeValue::String(form_url_encode(encoding, &input)))
        }
        "decodeBytes" => {
            let label = required_string(call, 2, "encoding label")?;
            let bytes_json = required_string(call, 3, "encoded bytes")?;
            let fatal = call
                .argument(4)
                .ok_or_else(|| NativeError::new("missing fatal flag"))?
                .to_boolean();
            let ignore_bom = call
                .argument(5)
                .ok_or_else(|| NativeError::new("missing ignoreBOM flag"))?
                .to_boolean();
            let stream = call
                .argument(6)
                .ok_or_else(|| NativeError::new("missing stream flag"))?
                .to_boolean();
            let bytes: Vec<u8> = serde_json::from_str(&bytes_json).map_err(err)?;
            let encoding = encoding_rs::Encoding::for_label_no_replacement(label.as_bytes())
                .ok_or_else(|| NativeError::new("invalid encoding label"))?;
            match decode_bytes(encoding, &bytes, fatal, ignore_bom, !stream)? {
                Some(decoded) => Ok(NativeValue::String(decoded)),
                None => Ok(NativeValue::Null),
            }
        }
        "encodeUtf8" => {
            let input = required_string(call, 2, "text")?;
            Ok(NativeValue::String(
                serde_json::to_string(input.as_bytes()).map_err(err)?,
            ))
        }
        "base64Encode" => {
            use base64::Engine as _;
            let bytes_json = required_string(call, 2, "bytes")?;
            let bytes: Vec<u8> = serde_json::from_str(&bytes_json).map_err(err)?;
            Ok(NativeValue::String(
                base64::engine::general_purpose::STANDARD.encode(bytes),
            ))
        }
        "base64Decode" => {
            use base64::{Engine as _, alphabet, engine};
            let input = required_string(call, 2, "base64 input")?;
            let input = input
                .bytes()
                .filter(|byte| !matches!(byte, b'\t' | b'\n' | b'\x0C' | b'\r' | b' '))
                .collect::<Vec<_>>();
            let config = engine::general_purpose::GeneralPurposeConfig::new()
                .with_decode_padding_mode(engine::DecodePaddingMode::Indifferent)
                .with_decode_allow_trailing_bits(true);
            let decoder = engine::GeneralPurpose::new(&alphabet::STANDARD, config);
            match decoder.decode(input) {
                Ok(bytes) => Ok(NativeValue::String(
                    serde_json::to_string(&bytes).map_err(err)?,
                )),
                Err(_) => Ok(NativeValue::Null),
            }
        }
        "fetch" | "fetchStream" => {
            let url = required_string(call, 2, "fetch URL")?;
            let method = required_string(call, 3, "fetch method")?;
            let headers_json = required_string(call, 4, "fetch headers")?;
            let body = call
                .argument(5)
                .filter(|value| !value.is_null_or_undefined())
                .map(|value| value.to_string())
                .transpose()?;
            let (promise, settlement) = call.make_deferred_promise()?.into_parts();
            let mut fetches = state.fetches.borrow_mut();
            let id = fetches.next_id();
            fetches.push(
                PendingFetch {
                    id,
                    url,
                    method,
                    headers_json,
                    body,
                    streaming: operation == "fetchStream",
                },
                settlement,
            );
            Ok(NativeValue::ProtectedObject(promise))
        }
        _ => Err(NativeError::new(format!(
            "unknown native platform operation: {operation}"
        ))),
    }
}

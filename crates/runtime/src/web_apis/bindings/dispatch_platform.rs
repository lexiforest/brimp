use super::*;

pub(super) fn dispatch(
    state: &BindingState,
    call: &NativeCall<'_>,
    operation: &str,
) -> Result<NativeValue, NativeError> {
    match operation {
        "storageOperation" => {
            let request: serde_json::Value =
                serde_json::from_str(&required_string(call, 2, "storage request")?).map_err(err)?;
            let context = &state.browsing_context;
            if request["op"] == "subscribe" {
                let callback = call
                    .argument(3)
                    .ok_or_else(|| NativeError::new("missing storage listener"))?
                    .to_function()?;
                state.storage_realm.subscribe(context, callback);
                return Ok(NativeValue::String("null".into()));
            }
            let url = context
                .current_url()
                .and_then(|url| url::Url::parse(&url).ok());
            let tuple_origin = url
                .as_ref()
                .map(|url| url.origin().ascii_serialization())
                .filter(|origin| origin != "null");
            let opaque = tuple_origin.is_none();
            let origin = tuple_origin.unwrap_or_else(|| state.storage_realm.opaque_origin());
            let operation = request["op"].as_str().unwrap_or_default();
            let result = match operation {
                "lockRequest" => {
                    let callback = call
                        .argument(3)
                        .ok_or_else(|| NativeError::new("missing lock callback"))?
                        .to_function()?;
                    serde_json::json!(state.storage_realm.request(origin, &request, callback))
                }
                "lockRelease" => {
                    state
                        .storage_realm
                        .release(request["id"].as_u64().unwrap_or(0));
                    serde_json::Value::Null
                }
                "lockQuery" => state.storage_realm.query(&origin),
                "persistent" => serde_json::json!(!opaque && context.storage.persistent()),
                "cookieGet" if opaque => serde_json::json!([]),
                "cookieSet" if opaque => serde_json::Value::Null,
                "cookieGet" => serde_json::to_value(
                    context
                        .cookies
                        .matching(url.as_ref().unwrap().as_str())
                        .into_iter()
                        .filter(|cookie| !cookie.http_only)
                        .collect::<Vec<_>>(),
                )
                .map_err(err)?,
                "cookieSet" => {
                    context
                        .cookies
                        .store_document(
                            url.as_ref().unwrap().as_str(),
                            request["header"].as_str().unwrap_or_default(),
                        )
                        .map_err(err)?;
                    serde_json::Value::Null
                }
                _ => {
                    let mut data = if opaque {
                        state.storage_realm.ephemeral.lock().unwrap()
                    } else if request["namespace"] == "session" {
                        context.session_storage.lock().unwrap()
                    } else {
                        context.storage.data.lock().unwrap()
                    };
                    let mutation = matches!(
                        operation,
                        "set" | "delete" | "clear" | "cachePut" | "cacheDelete"
                    );
                    let previous = (!opaque
                        && context.storage.persistent()
                        && request["namespace"] != "session"
                        && mutation)
                        .then(|| data.clone());
                    let namespaces = data.entry(origin.clone()).or_default();
                    let namespace = request["namespace"].as_str().unwrap_or_default();
                    let map = namespaces.entry(namespace.to_owned()).or_default();
                    let key = request["key"].as_str().unwrap_or_default();
                    let old_value = map.get(key).cloned();
                    let changed = match operation {
                        "set" => old_value.as_deref() != request["value"].as_str(),
                        "delete" => old_value.is_some(),
                        "clear" => !map.is_empty(),
                        _ => false,
                    };
                    let result = match operation {
                        "cachePut" | "cacheDelete" => {
                            let mut entries: serde_json::Map<String, serde_json::Value> =
                                serde_json::from_str(
                                    map.get(key).map(String::as_str).unwrap_or("{}"),
                                )
                                .map_err(err)?;
                            let url = request["url"].as_str().unwrap_or_default();
                            let existed = if operation == "cachePut" {
                                entries
                                    .insert(
                                        url.to_owned(),
                                        serde_json::from_str(
                                            request["value"].as_str().unwrap_or("null"),
                                        )
                                        .map_err(err)?,
                                    )
                                    .is_some()
                            } else {
                                entries.remove(url).is_some()
                            };
                            map.insert(
                                key.to_owned(),
                                serde_json::to_string(&entries).map_err(err)?,
                            );
                            serde_json::json!(existed)
                        }
                        "get" => serde_json::to_value(map.get(key)).map_err(err)?,
                        "set" => {
                            map.insert(
                                key.to_owned(),
                                request["value"].as_str().unwrap_or_default().to_owned(),
                            );
                            serde_json::Value::Null
                        }
                        "delete" => serde_json::json!(map.shift_remove(key).is_some()),
                        "clear" => {
                            map.clear();
                            serde_json::Value::Null
                        }
                        "list" => serde_json::json!(map.keys().collect::<Vec<_>>()),
                        "estimate" => {
                            serde_json::json!({"usage":namespaces.values().flat_map(|map| map.iter()).map(|(k,v)| 2*(k.len()+v.len())).sum::<usize>(),"quota":null})
                        }
                        _ => return Err(NativeError::new("Unknown storage operation")),
                    };
                    if !opaque
                        && request["namespace"] != "session"
                        && mutation
                        && let Err(error) = context.storage.persist(&data)
                    {
                        if let Some(previous) = previous {
                            *data = previous;
                        }
                        return Err(NativeError::new(error));
                    }
                    if request["namespace"] == "local" && changed {
                        state.storage_realm.broadcast(&origin, serde_json::json!({"key":if operation == "clear" {serde_json::Value::Null} else {request["key"].clone()},"oldValue":if operation == "clear" {None} else {old_value},"newValue":if operation == "set" {request["value"].clone()} else {serde_json::Value::Null},"url":context.current_url()}));
                    }
                    result
                }
            };
            Ok(NativeValue::String(result.to_string()))
        }
        "cryptoOperation" => {
            let request = required_string(call, 2, "crypto request")?;
            state
                .crypto
                .borrow_mut()
                .execute(&request)
                .map(NativeValue::String)
                .map_err(NativeError::new)
        }
        "cryptoRandomBytes" => {
            let length = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing byte length"))?
                .to_number()?;
            if !length.is_finite() || length < 0.0 || length.fract() != 0.0 || length > 65_536.0 {
                return Err(NativeError::new("invalid byte length"));
            }
            let mut bytes = vec![0_u8; length as usize];
            getrandom::fill(&mut bytes).map_err(|error| NativeError::new(error.to_string()))?;
            Ok(NativeValue::String(
                serde_json::to_string(&bytes).map_err(err)?,
            ))
        }
        "webSocketCreate" => {
            let url = required_string(call, 2, "WebSocket URL")?;
            let mut streaming = state.streaming.borrow_mut();
            let id = streaming.next_id;
            streaming.next_id = streaming.next_id.wrapping_add(1).max(1);
            streaming
                .pending
                .push_back(PendingWebSocketOperation::Create { id, url });
            Ok(NativeValue::Number(id as f64))
        }
        "webSocketSend" | "webSocketSendBinary" => {
            let id = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing WebSocket id"))?
                .to_number()? as u64;
            let message = required_string(call, 3, "WebSocket message")?;
            let operation = if operation == "webSocketSendBinary" {
                PendingWebSocketOperation::SendBinary {
                    id,
                    message: serde_json::from_str(&message).map_err(err)?,
                }
            } else {
                PendingWebSocketOperation::SendText { id, message }
            };
            state.streaming.borrow_mut().pending.push_back(operation);
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
        "fetchStreamConsumed" => {
            let id = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing stream id"))?
                .to_number()? as u64;
            let bytes = call
                .argument(3)
                .ok_or_else(|| NativeError::new("missing byte count"))?
                .to_number()? as usize;
            state
                .streaming
                .borrow_mut()
                .pending
                .push_back(PendingWebSocketOperation::ConsumeFetch { id, bytes });
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
        "taskPriority" => {
            let id = call.argument(2).unwrap().to_number()? as u32;
            let priority = call.argument(3).unwrap().to_number()? as u8;
            if let Some(timer) = state
                .timers
                .borrow_mut()
                .timers
                .iter_mut()
                .find(|t| t.id == id)
            {
                timer.priority = priority.min(3);
            }
            Ok(NativeValue::Undefined)
        }
        "setTimeout" | "setInterval" | "scheduleTask" => {
            let callback = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing timer callback"))?
                .to_function()?;
            let delay = call
                .argument(3)
                .map(|value| value.to_number())
                .transpose()?
                .unwrap_or(0.0);
            let id =
                state
                    .timers
                    .borrow_mut()
                    .schedule(delay, operation == "setInterval", callback);
            if operation == "scheduleTask" {
                let priority = call
                    .argument(4)
                    .map(|v| v.to_number())
                    .transpose()?
                    .unwrap_or(1.0) as u8;
                if let Some(timer) = state
                    .timers
                    .borrow_mut()
                    .timers
                    .iter_mut()
                    .find(|t| t.id == id)
                {
                    timer.priority = priority.min(3);
                }
            }
            Ok(NativeValue::Number(f64::from(id)))
        }
        "clearTimeout" | "clearInterval" => {
            let id = call
                .argument(2)
                .map(|value| value.to_number())
                .transpose()?
                .unwrap_or(0.0) as u32;
            state.timers.borrow_mut().clear(id);
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
        "locationNavigate" => {
            let href = required_string(call, 2, "navigation URL")?;
            url::Url::parse(&href).map_err(err)?;
            state.browsing_context.request_navigation(href);
            Ok(NativeValue::Undefined)
        }
        "historyUpdateUrl" => {
            let href = required_string(call, 2, "history URL")?;
            let current = state
                .browsing_context
                .current_url()
                .ok_or_else(|| NativeError::new("the document has no URL"))?;
            let current = url::Url::parse(&current).map_err(err)?;
            let next = url::Url::parse(&href).map_err(err)?;
            if current.origin() != next.origin() {
                return Err(NativeError::new(
                    "SecurityError: history URL must be same-origin",
                ));
            }
            state.browsing_context.set_url(next.as_str());
            Ok(NativeValue::String(next.to_string()))
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
        "urlSetAnchor" => {
            let href = required_string(call, 2, "URL href")?;
            let component = required_string(call, 3, "URL component")?;
            let value = required_string(call, 4, "URL component value")?;
            let mut url = url::Url::parse(&href).map_err(err)?;
            Ok(if mutate_url_component(&mut url, &component, &value) {
                NativeValue::String(url.into())
            } else {
                NativeValue::Null
            })
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
                .map(|value| {
                    let value = value.to_string()?;
                    serde_json::from_str::<Vec<u8>>(&value).map_err(err)
                })
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

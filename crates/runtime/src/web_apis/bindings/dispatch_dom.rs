use super::*;
use crate::dom::HTML_NAMESPACE;

pub(super) fn dispatch(
    state: &BindingState,
    call: &NativeCall<'_>,
    operation: &str,
) -> Result<NativeValue, NativeError> {
    match operation {
        "isNativeNode" => {
            let object = call
                .argument(1)
                .map(|value| value.as_object())
                .transpose()?
                .flatten();
            Ok(NativeValue::Boolean(
                object
                    .and_then(|object| state.wrappers.node_id(&object))
                    .is_some(),
            ))
        }
        "cssParseSheet" => Ok(NativeValue::String(
            serde_json::to_string(
                &brimp_lexbor::stylesheet(&required_string(call, 2, "CSS")?).map_err(err)?,
            )
            .map_err(err)?,
        )),

        "documentWriteError" => {
            let id = required_document_target(state, call)?;
            Ok(state
                .document
                .borrow()
                .document_write_error(id)
                .map_or(NativeValue::Null, |error| NativeValue::String(error.into())))
        }
        "documentOpen" => {
            let id = required_document_target(state, call)?;
            if call.argument(2).is_some_and(|value| value.to_boolean()) {
                let mut url = url::Url::parse(&document_url(state, 0)).map_err(err)?;
                url.set_fragment(None);
                state.document.borrow().set_document_url(id, url.into());
            }
            let writer = state.document.borrow().writer(id).map_err(err)?;
            state.document_writers.borrow_mut().insert(id, writer);
            Ok(NativeValue::Undefined)
        }
        "documentWrite" | "documentWriteNext" | "documentClose" => {
            let id = required_document_target(state, call)?;
            let mut writers = state.document_writers.borrow_mut();
            let Some(writer) = writers.get_mut(&id) else {
                return Ok(NativeValue::Null);
            };
            if operation == "documentWrite" {
                writer.write(&required_string(call, 2, "HTML")?);
            }
            let script = if operation == "documentClose" {
                writer.finish().map_err(err)?;
                None
            } else {
                writer.resume().map_err(err)?
            };
            let mutations = writer.take_mutations().map_err(err)?;
            let doc = state.document.borrow();
            let script = script.map(|node| doc.writer_node_id(node));
            let records: Vec<_> = mutations.into_iter().map(|r|serde_json::json!({"target":doc.writer_node_id(r.target),"node":doc.writer_node_id(r.node),"previous":r.previous.map(|n|doc.writer_node_id(n)),"next":r.next.map(|n|doc.writer_node_id(n)),"removed":r.removed,"oldValue":r.old_value})).collect();
            if operation == "documentClose" {
                writers.remove(&id);
            }
            Ok(NativeValue::String(
                serde_json::json!({"script":script,"records":records}).to_string(),
            ))
        }
        "htmlDecodedTextLength" => {
            let text = required_string(call, 2, "text")?;
            // Reuse Lexbor's entity and newline handling in a temporary arena;
            // this metadata check must not mutate the page DOM or emit events.
            let arena = brimp_lexbor::Document::new().map_err(err)?;
            let body = arena
                .create_element("body", HTML_NAMESPACE, "")
                .map_err(err)?;
            let fragment = arena.fragment(body, &text).map_err(err)?;
            Ok(NativeValue::Number(
                arena.text(fragment).map_err(err)?.to_utf16().len() as f64,
            ))
        }
        "createDocument" => {
            let root = state.document.borrow().create_document().map_err(err)?;
            state
                .document
                .borrow()
                .set_content_type(root, "application/xml");
            node_value(state, call, root)
        }
        "domParserParse" | "dataDocumentParse" => {
            let mut input = required_string(call, 2, "input")?;
            let mut encoding = encoding_rs::UTF_8;
            let mut document_url_value = document_url(state, 0);
            let mut content_type = "text/html".to_owned();
            if operation == "dataDocumentParse" {
                let data = data_url::DataUrl::process(&input).map_err(err)?;
                content_type = data.mime_type().to_string();
                document_url_value = input.clone();
                let (bytes, _) = data.decode_to_vec().map_err(err)?;
                let (decoded, selected) =
                    crate::dom::decode_html(&bytes, Some(&data.mime_type().to_string()))
                        .map_err(err)?;
                input = decoded.into_owned();
                encoding = selected;
            } else if required_string(call, 3, "type")? != "text/html" {
                return Err(NativeError::new("only text/html parsing is supported"));
            }
            let root = state.document.borrow().create_document().map_err(err)?;
            state.document.borrow().set_encoding(root, encoding);
            state
                .document
                .borrow()
                .set_document_url(root, document_url_value);
            state
                .document
                .borrow()
                .set_content_type(root, &content_type);
            let mut parser =
                HtmlParserSession::new_at_root(Rc::clone(&state.document), &input, root)
                    .map_err(err)?;
            while !matches!(parser.resume().map_err(err)?, ParseProgress::Done) {}
            if call.argument(4).is_some_and(|value| value.to_boolean()) {
                state.document.borrow().set_no_quirks(root).map_err(err)?;
            }
            node_value(state, call, root)
        }
        "documentURL" => {
            let id = required_document_target(state, call)?;
            Ok(NativeValue::String(document_url(state, id)))
        }
        "nodeBaseURI" => {
            let id = required_node_target(state, call)?;
            let doc = state.document.borrow();
            let owner = doc.node_document(id).ok_or_else(stale_wrapper)?;
            let fallback = document_url(state, owner);
            let base = resolve_document_base_url(&doc, owner, &fallback, url::Url::options());
            Ok(NativeValue::String(
                base.map(String::from).unwrap_or(fallback),
            ))
        }
        "documentSetURL" => {
            let id = required_document_target(state, call)?;
            state
                .document
                .borrow()
                .set_document_url(id, required_string(call, 2, "document URL")?);
            Ok(NativeValue::Undefined)
        }
        "documentContentType" => {
            let id = required_document_target(state, call)?;
            Ok(NativeValue::String(
                state.document.borrow().content_type(id),
            ))
        }
        "documentEncoding" => {
            let root = required_document_target(state, call)?;
            Ok(NativeValue::String(
                state.document.borrow().encoding(root).name().into(),
            ))
        }
        "compatMode" => {
            let root = required_document_target(state, call)?;
            Ok(NativeValue::String(
                state
                    .document
                    .borrow()
                    .compat_mode(root)
                    .map_err(err)?
                    .into(),
            ))
        }
        "documentElement" | "head" | "body" | "doctype" => {
            let root = required_document_target(state, call)?;
            let doc = state.document.borrow();
            let id = if operation == "documentElement" || operation == "doctype" {
                doc.children(root).map_err(err)?.into_iter().find(|id| {
                    doc.kind(*id).ok() == Some(if operation == "doctype" { 10 } else { 1 })
                })
            } else {
                let element = doc
                    .children(root)
                    .map_err(err)?
                    .into_iter()
                    .find(|id| doc.kind(*id).ok() == Some(1));
                let mut selected = None;
                if let Some(element) = element {
                    let eligible = operation == "head"
                        || doc.namespace(element).map_err(err)? == HTML_NAMESPACE
                            && doc.local_name(element).map_err(err)? == "html";
                    if eligible {
                        for child in doc.children(element).map_err(err)? {
                            if doc.kind(child).map_err(err)? != 1
                                || doc.namespace(child).map_err(err)? != HTML_NAMESPACE
                            {
                                continue;
                            }
                            let name = doc.local_name(child).map_err(err)?;
                            if name == operation || operation == "body" && name == "frameset" {
                                selected = Some(child);
                                break;
                            }
                        }
                    }
                }
                selected
            };
            drop(doc);
            optional_node(state, call, id)
        }
        "documentCollection" => {
            let root = required_document_target(state, call)?;
            let collection = required_string(call, 2, "document collection")?;
            let doc = state.document.borrow();
            let mut nodes = Vec::new();
            if collection != "applets" {
                for node in doc.descendants(root).map_err(err)? {
                    if doc.kind(node).map_err(err)? != 1
                        || doc.namespace(node).map_err(err)? != HTML_NAMESPACE
                    {
                        continue;
                    }
                    let name = doc.local_name(node).map_err(err)?;
                    let matches = match collection.as_str() {
                        "forms" => name == "form",
                        "images" => name == "img",
                        "embeds" => name == "embed",
                        "scripts" => name == "script",
                        "links" | "anchors" => {
                            let (eligible, attribute) = if collection == "links" {
                                (name == "a" || name == "area", "href")
                            } else {
                                (name == "a", "name")
                            };
                            eligible
                                && doc.attributes(node).map_err(err)?.iter().any(|item| {
                                    item.namespace.is_none() && item.local_name == attribute
                                })
                        }
                        _ => false,
                    };
                    if matches {
                        nodes.push(node);
                    }
                }
            }
            drop(doc);
            node_array(state, call, &nodes)
        }
        "documentTitleElement" => {
            let document = required_document_target(state, call)?;
            let doc = state.document.borrow();
            let root = doc
                .children(document)
                .map_err(err)?
                .into_iter()
                .find(|id| doc.kind(*id).ok() == Some(1));
            let svg_root = root.filter(|id| {
                doc.namespace(*id).ok().as_deref() == Some("http://www.w3.org/2000/svg")
                    && doc.local_name(*id).ok().as_deref() == Some("svg")
            });
            let (nodes, namespace) = if let Some(root) = svg_root {
                (
                    doc.children(root).map_err(err)?,
                    "http://www.w3.org/2000/svg",
                )
            } else {
                (doc.descendants(document).map_err(err)?, HTML_NAMESPACE)
            };
            let title = nodes.into_iter().find(|id| {
                doc.local_name(*id).ok().as_deref() == Some("title")
                    && doc.namespace(*id).ok().as_deref() == Some(namespace)
            });
            drop(doc);
            optional_node(state, call, title)
        }
        "cookie" => {
            required_document_target(state, call)?;
            Ok(NativeValue::String(
                state.browsing_context.document_cookies(),
            ))
        }
        "setCookie" => {
            required_document_target(state, call)?;
            state
                .browsing_context
                .set_document_cookie(&required_string(call, 2, "cookie")?);
            Ok(NativeValue::Undefined)
        }
        "createElement" | "createElementNS" => {
            let owner = required_document_target(state, call)?;
            let (namespace, qualified_name) = if operation == "createElement" {
                (
                    HTML_NAMESPACE.to_owned(),
                    required_string(call, 2, "tag name")?.to_ascii_lowercase(),
                )
            } else {
                (
                    call.argument(2)
                        .filter(|v| !v.is_null_or_undefined())
                        .map(|v| v.to_string())
                        .transpose()?
                        .unwrap_or_default(),
                    required_string(call, 3, "qualified name")?,
                )
            };
            let (prefix, name) = qualified_name
                .split_once(':')
                .unwrap_or(("", &qualified_name));
            if name.is_empty()
                || name.contains(':')
                || (!prefix.is_empty() && namespace.is_empty())
                || qualified_name.starts_with(':')
            {
                return Err(NativeError::new("invalid qualified name or namespace"));
            }
            let id = state
                .document
                .borrow()
                .create_element(owner, name, &namespace, prefix)
                .map_err(err)?;
            node_value(state, call, id)
        }
        "createTextNode" | "createComment" | "createDocumentFragment" => {
            let owner = required_document_target(state, call)?;
            let doc = state.document.borrow();
            let id = match operation {
                "createTextNode" => doc.create_text(owner, required_dom_string(call, 2, "text")?),
                "createComment" => {
                    doc.create_comment(owner, required_dom_string(call, 2, "comment")?)
                }
                _ => doc.create_document_fragment(owner),
            }
            .map_err(err)?;
            drop(doc);
            node_value(state, call, id)
        }
        "adoptNode" => {
            let owner = required_document_target(state, call)?;
            let id = state
                .wrappers
                .node_id(&required_object(call, 2, "node")?)
                .ok_or_else(stale_wrapper)?;
            state
                .document
                .borrow()
                .adopt_subtree(id, owner)
                .map_err(err)?;
            node_value(state, call, id)
        }
        "cloneNode" | "importNode" => {
            let target = required_node_target(state, call)?;
            let (source, index) = if operation == "importNode" {
                required_document_target(state, call)?;
                (
                    state
                        .wrappers
                        .node_id(&required_object(call, 2, "node")?)
                        .ok_or_else(stale_wrapper)?,
                    3,
                )
            } else {
                (target, 2)
            };
            let deep = call.argument(index).is_some_and(|v| v.to_boolean());
            let doc = state.document.borrow();
            if doc.is_document(source) && operation == "importNode" {
                return Err(NativeError::new(
                    "NotSupportedError: cannot import a document",
                ));
            }
            let owner = if operation == "importNode" {
                target
            } else {
                doc.node_document(source).unwrap_or(0)
            };
            let clone = doc.clone_node(source, deep, owner).map_err(err)?;
            if doc.is_document(source) {
                doc.set_document_url(clone, document_url(state, source));
            }
            drop(doc);
            node_value(state, call, clone)
        }
        "getElementById"
        | "getElementsByTagName"
        | "getElementsByClassName"
        | "getElementsByName" => {
            let root = required_parent_node_target(state, call)?;
            let dom_name = required_dom_string(call, 2, "name")?;
            let name = dom_name.to_string_lossy();
            let units = dom_name.to_utf16();
            let whitespace = |unit: &u16| matches!(unit, 0x09 | 0x0a | 0x0c | 0x0d | 0x20);
            let names: Vec<_> = units
                .split(whitespace)
                .filter(|name| !name.is_empty())
                .collect();
            let doc = state.document.borrow();
            let nodes: Vec<_> = doc
                .descendants(root)
                .map_err(err)?
                .into_iter()
                .filter(|id| doc.is_element(*id))
                .filter(|id| match operation {
                    "getElementById" => doc.attribute(*id, "id").as_ref() == Some(&dom_name),
                    "getElementsByName" => doc.attribute(*id, "name").as_ref() == Some(&dom_name),
                    "getElementsByTagName" => {
                        name == "*"
                            || doc.local_name(*id).is_ok_and(|tag| {
                                if doc.namespace(*id).ok().as_deref() == Some(HTML_NAMESPACE) {
                                    tag == name.to_ascii_lowercase()
                                } else {
                                    tag == name
                                }
                            })
                    }
                    _ => {
                        !names.is_empty()
                            && doc.attribute(*id, "class").is_some_and(|classes| {
                                let units = classes.to_utf16();
                                names
                                    .iter()
                                    .all(|name| units.split(whitespace).any(|class| class == *name))
                            })
                    }
                })
                .collect();
            drop(doc);
            if operation == "getElementById" {
                optional_node(state, call, nodes.first().copied())
            } else {
                node_array(state, call, &nodes)
            }
        }
        "querySelector" | "querySelectorAll" => {
            let root = required_parent_node_target(state, call)?;
            let nodes = state
                .document
                .borrow()
                .select(root, &required_string(call, 2, "selector")?)
                .map_err(err)?;
            if operation == "querySelector" {
                optional_node(state, call, nodes.first().copied())
            } else {
                node_array(state, call, &nodes)
            }
        }
        "matches" => {
            let id = required_element_target(state, call)?;
            Ok(NativeValue::Boolean(
                state
                    .document
                    .borrow()
                    .matches(id, &required_string(call, 2, "selector")?)
                    .map_err(err)?,
            ))
        }
        "ownerDocument" | "parentNode" | "firstChild" | "lastChild" | "previousSibling"
        | "nextSibling" | "templateContent" | "parserForm" => {
            let id = required_node_target(state, call)?;
            let doc = state.document.borrow();
            let node = match operation {
                "ownerDocument" => {
                    if doc.is_document(id) {
                        None
                    } else {
                        doc.node_document(id)
                    }
                }
                "parentNode" => doc.parent(id).map_err(err)?,
                "parserForm" => doc.parser_form(id).map_err(err)?,
                "firstChild" => doc.children(id).map_err(err)?.first().copied(),
                "lastChild" => doc.children(id).map_err(err)?.last().copied(),
                "templateContent" => doc.template_content(id).map_err(err)?,
                _ => doc.parent(id).map_err(err)?.and_then(|parent| {
                    let children = doc.children(parent).ok()?;
                    let index = children.iter().position(|child| *child == id)?;
                    let index = if operation == "previousSibling" {
                        index.checked_sub(1)?
                    } else {
                        index + 1
                    };
                    children.get(index).copied()
                }),
            };
            drop(doc);
            optional_node(state, call, node)
        }
        "previousElementSibling" | "nextElementSibling" => {
            let id = required_node_target(state, call)?;
            let doc = state.document.borrow();
            let mut sibling = None;
            if let Some(parent) = doc.parent(id).map_err(err)? {
                let children = doc.children(parent).map_err(err)?;
                if operation == "previousElementSibling" {
                    for child in children {
                        if child == id {
                            break;
                        }
                        if doc.kind(child).map_err(err)? == 1 {
                            sibling = Some(child);
                        }
                    }
                } else {
                    let mut found = false;
                    for child in children {
                        if found && doc.kind(child).map_err(err)? == 1 {
                            sibling = Some(child);
                            break;
                        }
                        if child == id {
                            found = true;
                        }
                    }
                }
            }
            drop(doc);
            optional_node(state, call, sibling)
        }
        "elementChildren" | "childElementCount" | "firstElementChild" | "lastElementChild" => {
            let id = required_parent_node_target(state, call)?;
            let doc = state.document.borrow();
            let mut nodes = Vec::new();
            for child in doc.children(id).map_err(err)? {
                if doc.kind(child).map_err(err)? == 1 {
                    nodes.push(child);
                }
            }
            drop(doc);
            match operation {
                "elementChildren" => node_array(state, call, &nodes),
                "childElementCount" => Ok(NativeValue::Number(nodes.len() as f64)),
                "firstElementChild" => optional_node(state, call, nodes.first().copied()),
                _ => optional_node(state, call, nodes.last().copied()),
            }
        }
        "childNodes" => {
            let id = required_node_target(state, call)?;
            let nodes = state.document.borrow().children(id).map_err(err)?;
            node_array(state, call, &nodes)
        }
        "nodeType" => {
            let id = required_node_target(state, call)?;
            Ok(NativeValue::Number(f64::from(
                state.document.borrow().kind(id).map_err(err)?,
            )))
        }
        "nodeName" | "tagName" | "localName" | "namespaceURI" | "prefix" | "textContent"
        | "doctypePublicId" | "doctypeSystemId" => {
            let id = required_node_target(state, call)?;
            let doc = state.document.borrow();
            if operation == "textContent" {
                return if matches!(doc.kind(id).map_err(err)?, 9 | 10) {
                    Ok(NativeValue::Null)
                } else {
                    Ok(NativeValue::StringUtf16(
                        doc.text(id).map_err(err)?.to_utf16(),
                    ))
                };
            }
            let value = match operation {
                "localName" => doc.local_name(id),
                "namespaceURI" => doc.namespace(id),
                "prefix" => doc.prefix(id),
                "doctypePublicId" => doc.doctype_public_id(id),
                "doctypeSystemId" => doc.doctype_system_id(id),
                _ => doc.name(id),
            }
            .map_err(err)?;
            if value.is_empty() && matches!(operation, "namespaceURI" | "prefix") {
                Ok(NativeValue::Null)
            } else {
                Ok(NativeValue::String(value))
            }
        }
        "setTextContent" => {
            let id = required_node_target(state, call)?;
            if matches!(state.document.borrow().kind(id).map_err(err)?, 9 | 10) {
                return Ok(NativeValue::Undefined);
            }
            state
                .document
                .borrow()
                .set_text(id, required_dom_string(call, 2, "textContent")?)
                .map_err(err)?;
            Ok(NativeValue::Undefined)
        }
        "replaceChild" => {
            let parent = required_parent_node_target(state, call)?;
            let node = state
                .wrappers
                .node_id(&required_object(call, 2, "node")?)
                .ok_or_else(stale_wrapper)?;
            let child_object = required_object(call, 3, "child")?;
            let child = state
                .wrappers
                .node_id(&child_object)
                .ok_or_else(stale_wrapper)?;
            let doc = state.document.borrow();
            let children = doc.children(parent).map_err(err)?;
            let index = children
                .iter()
                .position(|n| *n == child)
                .ok_or_else(|| NativeError::new("Node is not a child"))?;
            let reference = children[index + 1..].iter().copied().find(|n| *n != node);
            if doc.parent(node).map_err(err)?.is_some() {
                doc.detach(node).map_err(err)?;
            }
            if node != child {
                doc.detach(child).map_err(err)?;
            }
            doc.insert(parent, node, reference).map_err(err)?;
            Ok(NativeValue::Object(child_object))
        }
        "appendChild" => mutate_child(state, call, ChildMutation::Append),
        "removeChild" => mutate_child(state, call, ChildMutation::Remove),
        "insertBefore" => mutate_child(state, call, ChildMutation::InsertBefore),
        "getAttribute" | "getAttributeOrEmpty" | "setAttribute" | "removeAttribute" => {
            let id = required_element_target(state, call)?;
            let doc = state.document.borrow();
            let mut name = required_string(call, 2, "attribute name")?;
            if doc.namespace(id).ok().as_deref() == Some(HTML_NAMESPACE) {
                name.make_ascii_lowercase();
            }
            match operation {
                "setAttribute" => {
                    doc.set_attribute(id, &name, required_dom_string(call, 3, "attribute value")?)
                        .map_err(err)?;
                    Ok(NativeValue::Undefined)
                }
                "removeAttribute" => {
                    doc.remove_attribute(id, &name).map_err(err)?;
                    Ok(NativeValue::Undefined)
                }
                _ => match doc.attribute(id, &name) {
                    Some(value) => Ok(NativeValue::StringUtf16(value.to_utf16())),
                    None if operation == "getAttributeOrEmpty" => {
                        Ok(NativeValue::String(String::new()))
                    }
                    None => Ok(NativeValue::Null),
                },
            }
        }
        "setAttributeNS" | "removeAttributeNS" => {
            let id = required_element_target(state, call)?;
            let ns = required_string(call, 2, "namespace")?;
            let name = required_string(call, 3, "name")?;
            let doc = state.document.borrow();
            if operation == "setAttributeNS" {
                doc.set_attribute_ns(id, &ns, &name, required_dom_string(call, 4, "value")?)
                    .map_err(err)?;
            } else {
                doc.remove_attribute_ns(id, &ns, &name).map_err(err)?;
            }
            Ok(NativeValue::Undefined)
        }
        "elementAttributes" => {
            let id = required_element_target(state, call)?;
            let attributes = state.document.borrow().attributes(id).map_err(err)?;
            let records = attributes
                .into_iter()
                .map(|attribute| {
                    call.make_value_array(vec![
                        NativeValue::String(attribute.name),
                        NativeValue::String(attribute.local_name),
                        attribute
                            .namespace
                            .map_or(NativeValue::Null, NativeValue::String),
                        attribute
                            .prefix
                            .map_or(NativeValue::Null, NativeValue::String),
                        NativeValue::StringUtf16(attribute.value.to_utf16()),
                    ])
                    .map(NativeValue::ProtectedObject)
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(NativeValue::ProtectedObject(
                call.make_value_array(records)?,
            ))
        }
        "elementUrl" => {
            let id = required_element_target(state, call)?;
            let property = required_string(call, 2, "URL property")?;
            let document_url = required_string(call, 3, "document URL")?;
            let attribute = if property == "origin" {
                "href"
            } else {
                &property
            };
            let doc = state.document.borrow();
            let encoding = call
                .argument(4)
                .map(|value| value.to_string())
                .transpose()?;
            let resolved = resolve_element_attribute_url(
                &doc,
                id,
                attribute,
                &document_url,
                encoding.as_deref(),
            );
            if property == "origin" {
                let value = resolved
                    .and_then(|value| url::Url::parse(&value).ok())
                    .map(|url| url.origin().ascii_serialization())
                    .unwrap_or_default();
                Ok(NativeValue::String(value))
            } else if let Some(value) = resolved {
                Ok(NativeValue::String(value))
            } else {
                // Safari returns the original DOMString when parsing fails,
                // including unpaired code units written through setAttribute.
                let value = doc
                    .attributes(id)
                    .map_err(err)?
                    .into_iter()
                    .find(|item| item.namespace.is_none() && item.local_name == attribute)
                    .map(|item| item.value)
                    .unwrap_or_default();
                Ok(NativeValue::StringUtf16(value.to_utf16()))
            }
        }
        "innerHTML" | "outerHTML" | "serializeNode" => {
            let id = required_node_target(state, call)?;
            let doc = state.document.borrow();
            let target = if operation == "innerHTML" {
                doc.template_content(id).map_err(err)?.unwrap_or(id)
            } else {
                id
            };
            Ok(NativeValue::StringUtf16(
                doc.serialize(target, operation == "innerHTML")
                    .map_err(err)?
                    .to_utf16(),
            ))
        }
        "setInnerHTML" => {
            let id = required_element_target(state, call)?;
            state
                .document
                .borrow()
                .set_inner_html(id, &required_string(call, 2, "innerHTML")?)
                .map_err(err)?;
            Ok(NativeValue::Undefined)
        }
        "style" => {
            let id = required_element_target(state, call)?;
            Ok(NativeValue::Object(
                state.style_wrappers.wrap_with_prototype(
                    call,
                    id,
                    prototypes(state).css_style.identity(),
                ),
            ))
        }
        "styleGetProperty" => {
            let id = required_style_target(state, call)?;
            Ok(NativeValue::String(inline_style_property(
                state,
                id,
                &required_string(call, 2, "property")?,
            )))
        }
        "styleDeclarations" => {
            let id = required_style_target(state, call)?;
            Ok(NativeValue::String(
                serde_json::to_string(
                    &state
                        .document
                        .borrow()
                        .inline_style_declarations(id)
                        .unwrap_or_default(),
                )
                .map_err(err)?,
            ))
        }
        "styleCssText" => {
            let id = required_style_target(state, call)?;
            Ok(NativeValue::String(
                state
                    .document
                    .borrow()
                    .inline_style_css(id)
                    .unwrap_or_default(),
            ))
        }
        "styleWritable" => {
            required_style_target(state, call)?;
            Ok(NativeValue::Boolean(true))
        }
        "styleSetCssText" => {
            let id = required_style_target(state, call)?;
            state
                .document
                .borrow()
                .set_inline_style_css(id, &required_string(call, 2, "CSS")?);
            Ok(NativeValue::Undefined)
        }
        "styleSetProperty" => {
            let id = required_style_target(state, call)?;
            state.document.borrow().set_style_property(
                id,
                &required_string(call, 2, "property")?,
                &required_string(call, 3, "value")?,
            );
            Ok(NativeValue::Undefined)
        }
        "styleRemoveProperty" => {
            let id = required_style_target(state, call)?;
            let name = required_string(call, 2, "property")?;
            let old = inline_style_property(state, id, &name);
            state.document.borrow().remove_style_property(id, &name);
            Ok(NativeValue::String(old))
        }
        _ => Err(NativeError::new(format!(
            "unknown native DOM operation: {operation}"
        ))),
    }
}

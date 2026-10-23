use super::*;
use crate::dom::HTML_NAMESPACE;

pub(super) fn dispatch(
    state: &BindingState,
    call: &NativeCall<'_>,
    operation: &str,
) -> Result<NativeValue, NativeError> {
    match operation {
        "cssParseSheet" => Ok(NativeValue::String(
            serde_json::to_string(
                &brimp_lexbor::stylesheet(&required_string(call, 2, "CSS")?).map_err(err)?,
            )
            .map_err(err)?,
        )),

        "documentOpen" => {
            let id = required_document_target(state, call)?;
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
        "domParserParse" => {
            let input = required_string(call, 2, "input")?;
            if required_string(call, 3, "type")? != "text/html" {
                return Err(NativeError::new("only text/html parsing is supported"));
            }
            let root = state.document.borrow().create_document().map_err(err)?;
            let mut parser =
                HtmlParserSession::new_at_root(Rc::clone(&state.document), &input, root)
                    .map_err(err)?;
            while !matches!(parser.resume().map_err(err)?, ParseProgress::Done) {}
            node_value(state, call, root)
        }
        "documentElement" | "head" | "body" => {
            let root = required_document_target(state, call)?;
            let doc = state.document.borrow();
            let id = if operation == "documentElement" {
                doc.children(root)
                    .map_err(err)?
                    .into_iter()
                    .find(|id| doc.is_element(*id))
            } else {
                doc.select(root, operation).map_err(err)?.into_iter().next()
            };
            drop(doc);
            optional_node(state, call, id)
        }
        "title" => {
            let root = required_document_target(state, call)?;
            let doc = state.document.borrow();
            let text = doc
                .select(root, "title")
                .map_err(err)?
                .into_iter()
                .next()
                .map(|id| doc.text(id))
                .transpose()
                .map_err(err)?
                .unwrap_or_default();
            Ok(NativeValue::String(text))
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
                "createTextNode" => doc.create_text(owner, &required_string(call, 2, "text")?),
                "createComment" => doc.create_comment(owner, &required_string(call, 2, "comment")?),
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
            drop(doc);
            node_value(state, call, clone)
        }
        "getElementById"
        | "getElementsByTagName"
        | "getElementsByClassName"
        | "getElementsByName" => {
            let root = required_parent_node_target(state, call)?;
            let name = required_string(call, 2, "name")?;
            let names: Vec<_> = name.split_ascii_whitespace().collect();
            let doc = state.document.borrow();
            let nodes: Vec<_> = doc
                .descendants(root)
                .map_err(err)?
                .into_iter()
                .filter(|id| doc.is_element(*id))
                .filter(|id| match operation {
                    "getElementById" => doc.attribute(*id, "id").as_deref() == Some(name.as_str()),
                    "getElementsByName" => {
                        doc.attribute(*id, "name").as_deref() == Some(name.as_str())
                    }
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
                                names.iter().all(|name| {
                                    classes.split_ascii_whitespace().any(|class| class == *name)
                                })
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
        | "nextSibling" | "templateContent" => {
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
        "nodeName" | "tagName" | "localName" | "namespaceURI" | "prefix" | "textContent" => {
            let id = required_node_target(state, call)?;
            let doc = state.document.borrow();
            let value = match operation {
                "localName" => doc.local_name(id),
                "namespaceURI" => doc.namespace(id),
                "prefix" => doc.prefix(id),
                "textContent" => doc.text(id),
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
            state
                .document
                .borrow()
                .set_text(id, &required_string(call, 2, "textContent")?)
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
                    doc.set_attribute(id, &name, &required_string(call, 3, "attribute value")?)
                        .map_err(err)?;
                    Ok(NativeValue::Undefined)
                }
                "removeAttribute" => {
                    doc.remove_attribute(id, &name).map_err(err)?;
                    Ok(NativeValue::Undefined)
                }
                _ => match doc.attribute(id, &name) {
                    Some(value) => Ok(NativeValue::String(value)),
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
                doc.set_attribute_ns(id, &ns, &name, &required_string(call, 4, "value")?)
                    .map_err(err)?;
            } else {
                doc.remove_attribute_ns(id, &ns, &name).map_err(err)?;
            }
            Ok(NativeValue::Undefined)
        }
        "elementAttributes" => {
            let id = required_element_target(state, call)?;
            let attributes = state.document.borrow().attributes(id).map_err(err)?;
            Ok(NativeValue::String(
                serde_json::to_string(&attributes).map_err(err)?,
            ))
        }
        "elementUrl" => {
            let id = required_element_target(state, call)?;
            let property = required_string(call, 2, "URL property")?;
            let attribute = if property == "origin" {
                "href"
            } else {
                &property
            };
            let doc = state.document.borrow();
            let value = resolve_element_attribute_url(&doc, &state.browsing_context, id, attribute)
                .unwrap_or_else(|| doc.attribute(id, attribute).unwrap_or_default());
            let value = if property == "origin" {
                url::Url::parse(&value)
                    .map(|url| url.origin().ascii_serialization())
                    .unwrap_or_default()
            } else {
                value
            };
            Ok(NativeValue::String(value))
        }
        "innerHTML" | "outerHTML" | "serializeNode" => {
            let id = required_node_target(state, call)?;
            let doc = state.document.borrow();
            let target = if operation == "innerHTML" {
                doc.template_content(id).map_err(err)?.unwrap_or(id)
            } else {
                id
            };
            Ok(NativeValue::String(
                doc.serialize(target, operation == "innerHTML")
                    .map_err(err)?,
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

use super::*;

pub(super) fn dispatch(
    state: &BindingState,
    call: &NativeCall<'_>,
    operation: &str,
) -> Result<NativeValue, NativeError> {
    match operation {
        "innerWidth" | "innerHeight" | "devicePixelRatio" => {
            let metrics = state.document.borrow().viewport_metrics();
            let index = match operation {
                "innerWidth" => 0,
                "innerHeight" => 1,
                _ => 2,
            };
            Ok(NativeValue::Number(metrics[index]))
        }
        "domParserParse" => {
            let input = required_string(call, 2, "input")?;
            let content_type = required_string(call, 3, "type")?;
            let root = state.document.borrow_mut().create_document();
            if content_type == "text/html" {
                let mut parser =
                    HtmlParserSession::new_at_root(Rc::clone(&state.document), &input, root);
                while !matches!(parser.resume(), ParseProgress::Done) {}
            } else if parse_xml_at_root(Rc::clone(&state.document), &input, root) {
                let mut document = state.document.borrow_mut();
                document
                    .blitz_mut()
                    .mutate()
                    .remove_and_drop_all_children(root);
                let name = QualName::new(
                    None,
                    Namespace::from("http://www.mozilla.org/newlayout/xml/parsererror.xml"),
                    LocalName::from("parsererror"),
                );
                let mut mutator = document.blitz_mut().mutate();
                let error = mutator.create_element(name, vec![]);
                let text = mutator.create_text_node("XML parsing error");
                mutator.append_children(error, &[text]);
                mutator.append_children(root, &[error]);
                drop(mutator);
                document.adopt_subtree(error, root);
            }
            node_value(state, call, root)
        }
        "documentElement" => {
            let root = required_document_target(state, call)?;
            let document = state.document.borrow();
            let id = document.node(root).and_then(|node| {
                node.children
                    .iter()
                    .copied()
                    .find(|id| document.node(*id).is_some_and(|node| node.is_element()))
            });
            drop(document);
            optional_node(state, call, id)
        }
        "title" => {
            let root = required_document_target(state, call)?;
            let document = state.document.borrow();
            let title = subtree_query_selector_all(&document, root, "title")?
                .into_iter()
                .next()
                .and_then(|id| document.node(id))
                .map(|node| node.text_content())
                .unwrap_or_default();
            Ok(NativeValue::String(title))
        }
        "cookie" => {
            required_document_target(state, call)?;
            Ok(NativeValue::String(
                state.browsing_context.document_cookies(),
            ))
        }
        "setCookie" => {
            required_document_target(state, call)?;
            let cookie = required_string(call, 2, "cookie")?;
            state.browsing_context.set_document_cookie(&cookie);
            Ok(NativeValue::Undefined)
        }
        "head" => {
            let root = required_document_target(state, call)?;
            let document = state.document.borrow();
            let id = subtree_query_selector_all(&document, root, "head")?
                .into_iter()
                .next();
            drop(document);
            optional_node(state, call, id)
        }
        "body" => {
            let root = required_document_target(state, call)?;
            let document = state.document.borrow();
            let id = subtree_query_selector_all(&document, root, "body")?
                .into_iter()
                .next();
            drop(document);
            optional_node(state, call, id)
        }
        "imageMetadata" => {
            let image = required_image_target(state, call)?;
            let (complete, width, height, source_url, cors_mode) = {
                let document = state.document.borrow();
                let element = document
                    .node(image)
                    .and_then(|node| node.element_data())
                    .ok_or_else(stale_wrapper)?;
                let raster = element.raster_image_data();
                let source_url =
                    resolve_element_attribute_url(&document, &state.browsing_context, image, "src");
                let cors_mode = element.attr(LocalName::from("crossorigin")).map(|value| {
                    if value.eq_ignore_ascii_case("use-credentials") {
                        "use-credentials"
                    } else {
                        "anonymous"
                    }
                });
                (
                    raster.is_some(),
                    raster.map_or(0, |image| image.width),
                    raster.map_or(0, |image| image.height),
                    source_url,
                    cors_mode,
                )
            };
            let origin_clean = source_url.as_deref().is_none_or(|source_url| {
                state
                    .browsing_context
                    .resource_origin_clean(source_url, cors_mode)
            });
            Ok(NativeValue::String(
                serde_json::json!({
                    "complete": complete,
                    "width": width,
                    "height": height,
                    "originClean": origin_clean,
                })
                .to_string(),
            ))
        }
        "mediaElementOriginClean" => {
            let media = required_element_target(state, call)?;
            let (source_url, cors_mode) = {
                let document = state.document.borrow();
                let element = document
                    .node(media)
                    .and_then(|node| node.element_data())
                    .ok_or_else(stale_wrapper)?;
                let source_url =
                    resolve_element_attribute_url(&document, &state.browsing_context, media, "src");
                let cors_mode = element.attr(LocalName::from("crossorigin")).map(|value| {
                    if value.eq_ignore_ascii_case("use-credentials") {
                        "use-credentials"
                    } else {
                        "anonymous"
                    }
                });
                (source_url, cors_mode)
            };
            Ok(NativeValue::Boolean(source_url.as_deref().is_none_or(
                |source_url| {
                    state
                        .browsing_context
                        .resource_origin_clean(source_url, cors_mode)
                },
            )))
        }
        "createElement" => {
            let owner = required_document_target(state, call)?;
            let tag = required_string(call, 2, "tag name")?.to_ascii_lowercase();
            if tag.is_empty() {
                return Err(NativeError::new("tag name cannot be empty"));
            }
            let name = QualName::new(None, ns!(html), LocalName::from(tag));
            let id = {
                let mut document = state.document.borrow_mut();
                let id = document.blitz_mut().mutate().create_element(name, vec![]);
                document.set_node_document(id, owner);
                id
            };
            node_value(state, call, id)
        }
        "createElementNS" => {
            let owner = required_document_target(state, call)?;
            let namespace = call
                .argument(2)
                .filter(|value| !value.is_null_or_undefined())
                .map(|value| value.to_string())
                .transpose()?
                .unwrap_or_default();
            let qualified_name = required_string(call, 3, "qualified name")?;
            let mut parts = qualified_name.split(':');
            let first = parts.next().unwrap_or_default();
            let second = parts.next();
            if qualified_name.is_empty()
                || parts.next().is_some()
                || first.is_empty()
                || second.is_some_and(str::is_empty)
            {
                return Err(NativeError::new("invalid qualified name"));
            }
            let (prefix, local_name) = match second {
                Some(local_name) => (Some(Prefix::from(first)), local_name),
                None => (None, first),
            };
            if prefix.is_some() && namespace.is_empty() {
                return Err(NativeError::new("a prefixed name requires a namespace"));
            }
            let name = QualName::new(
                prefix,
                Namespace::from(namespace),
                LocalName::from(local_name),
            );
            let id = {
                let mut document = state.document.borrow_mut();
                let id = document.blitz_mut().mutate().create_element(name, vec![]);
                document.set_node_document(id, owner);
                id
            };
            node_value(state, call, id)
        }
        "createTextNode" => {
            let owner = required_document_target(state, call)?;
            let text = required_string(call, 2, "text")?;
            let id = {
                let mut document = state.document.borrow_mut();
                let id = document.blitz_mut().mutate().create_text_node(&text);
                document.set_node_document(id, owner);
                id
            };
            node_value(state, call, id)
        }
        "createComment" => {
            let owner = required_document_target(state, call)?;
            let data = required_string(call, 2, "comment data")?;
            let id = {
                let mut document = state.document.borrow_mut();
                let id = document.create_comment(&data);
                document.set_node_document(id, owner);
                id
            };
            node_value(state, call, id)
        }
        "createDocumentFragment" => {
            let owner = required_document_target(state, call)?;
            let id = {
                let mut document = state.document.borrow_mut();
                let id = document.create_document_fragment();
                document.set_node_document(id, owner);
                id
            };
            node_value(state, call, id)
        }
        "getElementById" => {
            let root = required_document_target(state, call)?;
            let id = required_string(call, 2, "id")?;
            let document = state.document.borrow();
            let node = descendant_ids(&document, root)?
                .into_iter()
                .find(|node_id| {
                    document
                        .node(*node_id)
                        .and_then(|node| node.element_data())
                        .and_then(|element| element.attr(LocalName::from("id")))
                        == Some(id.as_str())
                });
            drop(document);
            optional_node(state, call, node)
        }
        "elementFromPoint" => {
            required_document_target(state, call)?;
            let x = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing x coordinate"))?
                .to_number()?;
            let y = call
                .argument(3)
                .ok_or_else(|| NativeError::new("missing y coordinate"))?
                .to_number()?;
            let viewport = state.document.borrow().viewport_metrics();
            if !x.is_finite()
                || !y.is_finite()
                || x < 0.0
                || y < 0.0
                || x >= viewport[0]
                || y >= viewport[1]
            {
                return Ok(NativeValue::Null);
            }
            resolve_document(state);
            let node = state.document.borrow().element_at_point(x, y);
            optional_node(state, call, node)
        }
        "getElementsByTagName" => {
            let root_id = required_parent_node_target(state, call)?;
            let name = required_string(call, 2, "name")?.to_ascii_lowercase();
            let document = state.document.borrow();
            let nodes = descendant_ids(&document, root_id)?
                .into_iter()
                .filter(|id| {
                    document.node(*id).is_some_and(|node| match &node.data {
                        NodeData::Element(element) => {
                            name == "*" || element.name.local.as_ref() == name
                        }
                        _ => false,
                    })
                })
                .collect::<Vec<_>>();
            drop(document);
            node_array(state, call, &nodes)
        }
        "getElementsByClassName" => {
            let root_id = required_parent_node_target(state, call)?;
            let names = required_string(call, 2, "class names")?
                .split_ascii_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            let document = state.document.borrow();
            let nodes = if names.is_empty() {
                Vec::new()
            } else {
                descendant_ids(&document, root_id)?
                    .into_iter()
                    .filter(|id| {
                        document
                            .node(*id)
                            .and_then(|node| node.element_data())
                            .and_then(|element| element.attr(LocalName::from("class")))
                            .is_some_and(|classes| {
                                let classes =
                                    classes.split_ascii_whitespace().collect::<HashSet<_>>();
                                names.iter().all(|name| classes.contains(name.as_str()))
                            })
                    })
                    .collect()
            };
            drop(document);
            node_array(state, call, &nodes)
        }
        "getElementsByName" => {
            let root_id = required_document_target(state, call)?;
            let name = required_string(call, 2, "name")?;
            let document = state.document.borrow();
            let nodes = descendant_ids(&document, root_id)?
                .into_iter()
                .filter(|id| {
                    document
                        .node(*id)
                        .and_then(|node| node.element_data())
                        .is_some_and(|element| {
                            element.name.ns == ns!(html)
                                && element.attr(LocalName::from("name")) == Some(name.as_str())
                        })
                })
                .collect::<Vec<_>>();
            drop(document);
            node_array(state, call, &nodes)
        }
        "querySelector" => {
            let root_id = required_parent_node_target(state, call)?;
            let selector = required_string(call, 2, "selector")?;
            let document = state.document.borrow();
            let node = subtree_query_selector_all(&document, root_id, &selector)?
                .into_iter()
                .next();
            drop(document);
            optional_node(state, call, node)
        }
        "querySelectorAll" => {
            let root_id = required_parent_node_target(state, call)?;
            let selector = required_string(call, 2, "selector")?;
            let document = state.document.borrow();
            let nodes = subtree_query_selector_all(&document, root_id, &selector)?;
            drop(document);
            node_array(state, call, &nodes)
        }
        "matches" => {
            let id = required_element_target(state, call)?;
            let selector = required_string(call, 2, "selector")?;
            Ok(NativeValue::Boolean(
                state
                    .document
                    .borrow()
                    .query_selector_all(&selector)
                    .map_err(err)?
                    .contains(&id),
            ))
        }
        "ownerDocument" => {
            let id = required_node_target(state, call)?;
            let document = state.document.borrow();
            let owner = if document.is_document(id) {
                None
            } else {
                document.node_document(id)
            };
            drop(document);
            optional_node(state, call, owner)
        }
        "nodeType" => {
            let id = required_node_target(state, call)?;
            let document = state.document.borrow();
            let node = document.node(id).ok_or_else(stale_wrapper)?;
            let node_type = if document.is_document_fragment(id) {
                11.0
            } else {
                match node.data {
                    NodeData::Element(_) | NodeData::AnonymousBlock(_) => 1.0,
                    NodeData::Text(_) => 3.0,
                    NodeData::Comment => 8.0,
                    NodeData::Document => 9.0,
                }
            };
            Ok(NativeValue::Number(node_type))
        }
        "nodeName" => {
            let id = required_node_target(state, call)?;
            let document = state.document.borrow();
            let node = document.node(id).ok_or_else(stale_wrapper)?;
            let name = if document.is_document_fragment(id) {
                "#document-fragment".to_owned()
            } else {
                match &node.data {
                    NodeData::Element(element) | NodeData::AnonymousBlock(element) => {
                        element.name.local.to_string().to_ascii_uppercase()
                    }
                    NodeData::Text(_) => "#text".to_owned(),
                    NodeData::Comment => "#comment".to_owned(),
                    NodeData::Document => "#document".to_owned(),
                }
            };
            Ok(NativeValue::String(name))
        }
        "parentNode" => {
            let id = required_node_target(state, call)?;
            let parent = state
                .document
                .borrow()
                .node(id)
                .ok_or_else(stale_wrapper)?
                .parent;
            optional_node(state, call, parent)
        }
        "firstChild" => {
            let id = required_node_target(state, call)?;
            let child = state
                .document
                .borrow()
                .node(id)
                .ok_or_else(stale_wrapper)?
                .children
                .first()
                .copied();
            optional_node(state, call, child)
        }
        "lastChild" => {
            let id = required_node_target(state, call)?;
            let child = state
                .document
                .borrow()
                .node(id)
                .ok_or_else(stale_wrapper)?
                .children
                .last()
                .copied();
            optional_node(state, call, child)
        }
        "previousSibling" | "nextSibling" => {
            let id = required_node_target(state, call)?;
            let document = state.document.borrow();
            let node = document.node(id).ok_or_else(stale_wrapper)?;
            let parent = node.parent.and_then(|parent| document.node(parent));
            let sibling = parent.and_then(|parent| {
                let index = parent.children.iter().position(|child| *child == id)?;
                if operation == "previousSibling" {
                    index.checked_sub(1).map(|index| parent.children[index])
                } else {
                    parent.children.get(index + 1).copied()
                }
            });
            drop(document);
            optional_node(state, call, sibling)
        }
        "childNodes" => {
            let id = required_node_target(state, call)?;
            let children = state
                .document
                .borrow()
                .node(id)
                .ok_or_else(stale_wrapper)?
                .children
                .clone();
            node_array(state, call, &children)
        }
        "textContent" => {
            let id = required_node_target(state, call)?;
            let document = state.document.borrow();
            let node = document.node(id).ok_or_else(stale_wrapper)?;
            let text = document
                .comment_data(id)
                .map(str::to_owned)
                .unwrap_or_else(|| node.text_content());
            Ok(NativeValue::String(text))
        }
        "setTextContent" => {
            let id = required_node_target(state, call)?;
            let value = required_string(call, 2, "textContent")?;
            set_text_content(state, id, &value)?;
            Ok(NativeValue::Undefined)
        }
        "appendChild" => mutate_child(state, call, ChildMutation::Append),
        "removeChild" => mutate_child(state, call, ChildMutation::Remove),
        "insertBefore" => mutate_child(state, call, ChildMutation::InsertBefore),
        "cloneNode" => {
            let id = required_node_target(state, call)?;
            let deep = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing deep flag"))?
                .to_boolean();
            let clone = {
                let mut document = state.document.borrow_mut();
                let mut mutator = document.blitz_mut().mutate();
                let clone = mutator.deep_clone_node(id);
                if !deep {
                    mutator.remove_and_drop_all_children(clone);
                }
                drop(mutator);
                document.copy_node_metadata(id, clone, deep);
                clone
            };
            node_value(state, call, clone)
        }
        "tagName" => {
            let id = required_element_target(state, call)?;
            let document = state.document.borrow();
            let element = document
                .node(id)
                .and_then(|node| node.element_data())
                .ok_or_else(stale_wrapper)?;
            let local_name = element.name.local.to_string();
            let tag_name = if element.name.ns == ns!(html) {
                local_name.to_ascii_uppercase()
            } else if let Some(prefix) = &element.name.prefix {
                format!("{prefix}:{local_name}")
            } else {
                local_name
            };
            Ok(NativeValue::String(tag_name))
        }
        "localName" | "namespaceURI" | "prefix" => {
            let id = required_element_target(state, call)?;
            let document = state.document.borrow();
            let element = document
                .node(id)
                .and_then(|node| node.element_data())
                .ok_or_else(stale_wrapper)?;
            match operation {
                "localName" => Ok(NativeValue::String(element.name.local.to_string())),
                "namespaceURI" => {
                    if element.name.ns.is_empty() {
                        Ok(NativeValue::Null)
                    } else {
                        Ok(NativeValue::String(element.name.ns.to_string()))
                    }
                }
                "prefix" => match &element.name.prefix {
                    Some(prefix) => Ok(NativeValue::String(prefix.to_string())),
                    None => Ok(NativeValue::Null),
                },
                _ => unreachable!(),
            }
        }
        "getAttribute" | "getAttributeOrEmpty" => {
            let id = required_element_target(state, call)?;
            let name = required_string(call, 2, "attribute name")?.to_ascii_lowercase();
            let document = state.document.borrow();
            let value = document
                .node(id)
                .and_then(|node| node.element_data())
                .and_then(|element| element.attr(LocalName::from(name)));
            match (operation, value) {
                ("getAttributeOrEmpty", None) => Ok(NativeValue::String(String::new())),
                (_, None) => Ok(NativeValue::Null),
                (_, Some(value)) => Ok(NativeValue::String(value.to_owned())),
            }
        }
        "elementAttributes" => {
            let id = required_element_target(state, call)?;
            let document = state.document.borrow();
            let element = document
                .node(id)
                .and_then(|node| node.element_data())
                .ok_or_else(stale_wrapper)?;
            let attributes = element
                .attrs()
                .iter()
                .map(|attribute| {
                    let prefix = attribute.name.prefix.as_ref().map(ToString::to_string);
                    let local_name = attribute.name.local.to_string();
                    let name = prefix
                        .as_ref()
                        .map(|prefix| format!("{prefix}:{local_name}"))
                        .unwrap_or_else(|| local_name.clone());
                    serde_json::json!({
                        "namespaceURI": if attribute.name.ns.is_empty() {
                            None
                        } else {
                            Some(attribute.name.ns.to_string())
                        },
                        "prefix": prefix,
                        "localName": local_name,
                        "name": name,
                        "value": attribute.value,
                    })
                })
                .collect::<Vec<_>>();
            Ok(NativeValue::String(
                serde_json::to_string(&attributes).map_err(err)?,
            ))
        }
        "setAttribute" => {
            let id = required_element_target(state, call)?;
            let name = required_string(call, 2, "attribute name")?.to_ascii_lowercase();
            let value = required_string(call, 3, "attribute value")?;
            let name = QualName::new(None, ns!(), LocalName::from(name));
            state
                .document
                .borrow_mut()
                .blitz_mut()
                .mutate()
                .set_attribute(id, name, &value);
            Ok(NativeValue::Undefined)
        }
        "removeAttribute" => {
            let id = required_element_target(state, call)?;
            let name = required_string(call, 2, "attribute name")?.to_ascii_lowercase();
            let name = QualName::new(None, ns!(), LocalName::from(name));
            state
                .document
                .borrow_mut()
                .blitz_mut()
                .mutate()
                .clear_attribute(id, name);
            Ok(NativeValue::Undefined)
        }
        "elementUrl" => {
            let id = required_element_target(state, call)?;
            let property = required_string(call, 2, "URL property")?;
            let document = state.document.borrow();
            let element = document
                .node(id)
                .and_then(|node| node.element_data())
                .ok_or_else(stale_wrapper)?;
            let attribute = if property == "origin" {
                "href"
            } else {
                property.as_str()
            };
            let input = element
                .attr(LocalName::from(attribute))
                .unwrap_or_default()
                .to_owned();
            let document_url = state
                .browsing_context
                .current_url()
                .and_then(|url| url::Url::parse(&url).ok());
            let base_url = if element.name.local.as_ref() == "base" {
                document_url.clone()
            } else {
                document
                    .query_selector("base[href]")
                    .ok()
                    .flatten()
                    .and_then(|base_id| {
                        document
                            .node(base_id)
                            .and_then(|node| node.element_data())
                            .and_then(|base| base.attr(LocalName::from("href")))
                    })
                    .and_then(|base| {
                        url::Url::options()
                            .base_url(document_url.as_ref())
                            .parse(base)
                            .ok()
                    })
                    .or(document_url)
            };
            let parsed = url::Url::options()
                .base_url(base_url.as_ref())
                .parse(&input);
            let value = match (property.as_str(), parsed) {
                ("origin", Ok(parsed)) => parsed.origin().ascii_serialization(),
                (_, Ok(parsed)) => parsed.as_str().to_owned(),
                (_, Err(_)) => input,
            };
            Ok(NativeValue::String(value))
        }
        "innerHTML" => {
            let id = required_element_target(state, call)?;
            let document = state.document.borrow();
            let node = document.node(id).ok_or_else(stale_wrapper)?;
            let mut html = String::new();
            for child in &node.children {
                document
                    .node(*child)
                    .ok_or_else(stale_wrapper)?
                    .write_outer_html(&mut html);
            }
            Ok(NativeValue::String(html))
        }
        "setInnerHTML" => {
            let id = required_element_target(state, call)?;
            let html = required_string(call, 2, "innerHTML")?;
            let removed = descendant_ids(&state.document.borrow(), id)?;
            state
                .document
                .borrow_mut()
                .blitz_mut()
                .mutate()
                .set_inner_html(id, &html);
            state.wrappers.remove_nodes(&removed);
            state.style_wrappers.remove_nodes(&removed);
            Ok(NativeValue::Undefined)
        }
        "style" => {
            let id = required_element_target(state, call)?;
            let prototype = prototypes(state).css_style.identity();
            let style = state
                .style_wrappers
                .wrap_with_prototype(call, id, prototype);
            Ok(NativeValue::Object(style))
        }
        "getComputedStyle" => {
            let id = required_element_target(state, call)?;
            resolve_document(state);
            let prototype = prototypes(state).css_style.identity();
            let style = state
                .computed_style_wrappers
                .wrap_with_prototype(call, id, prototype);
            Ok(NativeValue::Object(style))
        }
        "styleSheetElements" => {
            required_document_target(state, call)?;
            let nodes = state.document.borrow().stylesheet_node_ids();
            node_array(state, call, &nodes)
        }
        "styleSheetRules" => {
            let id = required_element_target(state, call)?;
            cssom_json(
                state
                    .document
                    .borrow()
                    .stylesheet_rule_texts(id)
                    .ok_or(CssomError::NotAStyleSheet),
            )
        }
        "parseStyleSheetRule" => {
            let rule = required_string(call, 2, "CSS rule")?;
            cssom_json(
                state
                    .document
                    .borrow()
                    .parse_stylesheet_rule(&rule)
                    .map(|rule| vec![rule]),
            )
        }
        "parseStyleSheetText" => {
            let css = required_string(call, 2, "stylesheet text")?;
            cssom_json(Ok(state.document.borrow().parse_stylesheet_text(&css)))
        }
        "styleRuleDeclarations" => {
            let rule = required_string(call, 2, "CSS style rule")?;
            let declarations = state
                .document
                .borrow()
                .style_rule_declarations(&rule)
                .unwrap_or_default();
            Ok(NativeValue::String(
                serde_json::to_string(&declarations).map_err(err)?,
            ))
        }
        "styleRuleGetProperty" => {
            let rule = required_string(call, 2, "CSS style rule")?;
            let name = required_string(call, 3, "CSS property name")?;
            Ok(NativeValue::String(
                state
                    .document
                    .borrow()
                    .style_rule_property(&rule, &name)
                    .unwrap_or_default(),
            ))
        }
        "nestedRuleTexts" => {
            let rule = required_string(call, 2, "CSS grouping rule")?;
            cssom_json(
                state
                    .document
                    .borrow()
                    .nested_rule_texts(&rule)
                    .ok_or(CssomError::Syntax),
            )
        }
        "styleSheetInsertRule" => {
            let id = required_element_target(state, call)?;
            let rule = required_string(call, 2, "CSS rule")?;
            let index = call
                .argument(3)
                .ok_or_else(|| NativeError::new("missing CSS rule index"))?
                .to_number()? as usize;
            cssom_json(
                state
                    .document
                    .borrow_mut()
                    .insert_stylesheet_rule(id, &rule, index),
            )
        }
        "styleSheetDeleteRule" => {
            let id = required_element_target(state, call)?;
            let index = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing CSS rule index"))?
                .to_number()? as usize;
            cssom_json(
                state
                    .document
                    .borrow_mut()
                    .delete_stylesheet_rule(id, index),
            )
        }
        "styleSheetReplaceRule" => {
            let id = required_element_target(state, call)?;
            let rule = required_string(call, 2, "CSS rule")?;
            let index = call
                .argument(3)
                .ok_or_else(|| NativeError::new("missing CSS rule index"))?
                .to_number()? as usize;
            cssom_json(
                state
                    .document
                    .borrow_mut()
                    .replace_stylesheet_rule(id, &rule, index),
            )
        }
        "styleSheetReplace" => {
            let id = required_element_target(state, call)?;
            let css = required_string(call, 2, "stylesheet text")?;
            cssom_json(state.document.borrow_mut().replace_stylesheet(id, &css))
        }
        "styleGetProperty" => {
            let name = required_string(call, 2, "property name")?;
            let object = required_object(call, 1, "style receiver")?;
            if let Some(id) = state.style_wrappers.node_id(object) {
                Ok(NativeValue::String(inline_style_property(state, id, &name)))
            } else if let Some(id) = state.computed_style_wrappers.node_id(object) {
                resolve_document(state);
                Ok(NativeValue::String(
                    state
                        .document
                        .borrow()
                        .computed_style_property(id, &name)
                        .unwrap_or_default(),
                ))
            } else {
                Err(NativeError::new("receiver is not a CSSStyleDeclaration"))
            }
        }
        "styleDeclarations" => {
            let object = required_object(call, 1, "style receiver")?;
            let declarations = if let Some(id) = state.style_wrappers.node_id(object) {
                state
                    .document
                    .borrow()
                    .inline_style_declarations(id)
                    .unwrap_or_default()
            } else if let Some(id) = state.computed_style_wrappers.node_id(object) {
                resolve_document(state);
                state.document.borrow().computed_style_declarations(id)
            } else {
                return Err(NativeError::new("receiver is not a CSSStyleDeclaration"));
            };
            Ok(NativeValue::String(
                serde_json::to_string(&declarations).map_err(err)?,
            ))
        }
        "styleCssText" => {
            let object = required_object(call, 1, "style receiver")?;
            if let Some(id) = state.style_wrappers.node_id(object) {
                Ok(NativeValue::String(
                    state
                        .document
                        .borrow()
                        .inline_style_css(id)
                        .unwrap_or_default(),
                ))
            } else if state.computed_style_wrappers.node_id(object).is_some() {
                Ok(NativeValue::String(String::new()))
            } else {
                Err(NativeError::new("receiver is not a CSSStyleDeclaration"))
            }
        }
        "styleWritable" => {
            let object = required_object(call, 1, "style receiver")?;
            Ok(NativeValue::Boolean(
                state.style_wrappers.node_id(object).is_some(),
            ))
        }
        "styleSetCssText" => {
            let id = required_style_target(state, call)?;
            let css = required_string(call, 2, "declaration text")?;
            state.document.borrow_mut().set_inline_style_css(id, &css);
            Ok(NativeValue::Undefined)
        }
        "styleSetProperty" => {
            let id = required_style_target(state, call)?;
            let name = required_string(call, 2, "property name")?;
            let value = required_string(call, 3, "property value")?;
            state
                .document
                .borrow_mut()
                .set_style_property(id, &name, &value);
            Ok(NativeValue::Undefined)
        }
        "styleRemoveProperty" => {
            let id = required_style_target(state, call)?;
            let name = required_string(call, 2, "property name")?;
            let old = inline_style_property(state, id, &name);
            state.document.borrow_mut().remove_style_property(id, &name);
            Ok(NativeValue::String(old))
        }
        "clientWidth" | "clientHeight" | "offsetWidth" | "offsetHeight" => {
            let id = required_element_target(state, call)?;
            resolve_document(state);
            let document = state.document.borrow();
            let size = if operation.starts_with("client") {
                document.client_size(id)
            } else {
                document.offset_size(id)
            }
            .ok_or_else(stale_wrapper)?;
            let index = usize::from(operation.ends_with("Height"));
            Ok(NativeValue::Number(size[index]))
        }
        "boundingRect" => {
            let id = required_element_target(state, call)?;
            resolve_document(state);
            let rect = state
                .document
                .borrow()
                .bounding_rect(id)
                .ok_or_else(stale_wrapper)?;
            let values = rect.into_iter().map(NativeValue::Number).collect();
            Ok(NativeValue::ProtectedObject(call.make_value_array(values)?))
        }
        _ => Err(NativeError::new(format!(
            "unknown native DOM operation: {operation}"
        ))),
    }
}

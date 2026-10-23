use crate::{Result, invalid};
use brimp_runtime::runtime::plugins::{BridgeValue, Plugin};
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashSet},
    path::Path,
    sync::{Arc, Mutex},
};

pub fn catalog() -> Value {
    serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/catalog.json"))).unwrap()
}
fn source(name: &str) -> Option<&'static str> {
    macro_rules! sources { ($($name:literal),*) => { match name {
        $($name => Some(include_str!(concat!(env!("OUT_DIR"), "/", $name, ".js"))),)*
        _ => None,
    } }; }
    sources!(
        "storage",
        "canvas",
        "services",
        "extraction",
        "geometry",
        "css-typed-om",
        "scheduling",
        "graphics",
        "audio",
        "host-storage"
    )
}
pub fn load(names: &[String], options: &[String], extraction: bool) -> Result<Vec<Plugin>> {
    let mut configuration = BTreeMap::new();
    for item in options {
        let (name, value) = item
            .split_once('=')
            .ok_or_else(|| invalid("plugin options require NAME=JSON"))?;
        let value: Value = serde_json::from_str(value).map_err(invalid)?;
        if !value.is_object() {
            return Err(invalid("plugin options must be a JSON object"));
        }
        if configuration.insert(name.to_owned(), value).is_some() {
            return Err(invalid("duplicate plugin options"));
        }
    }
    let mut plugins = Vec::new();
    let mut seen = HashSet::new();
    let mut names = names.to_vec();
    if extraction && !names.iter().any(|name| name == "extraction") {
        names.push("extraction".into());
    }
    for requested in names {
        let builtin = source(&requested);
        let name = if builtin.is_some() {
            requested.clone()
        } else {
            Path::new(&requested)
                .file_stem()
                .and_then(|name| name.to_str())
                .filter(|name| !name.is_empty())
                .ok_or_else(|| invalid("invalid plugin filename"))?
                .to_owned()
        };
        if !seen.insert(name.clone()) {
            return Err(invalid(format!("duplicate plugin name: {name}")));
        }
        let options = configuration
            .remove(&name)
            .unwrap_or_else(|| serde_json::json!({}));
        if builtin.is_some() {
            let catalog = catalog();
            let allowed = &catalog[&name]["options"];
            for key in options.as_object().unwrap().keys() {
                if allowed.get(key).is_none() {
                    return Err(invalid(format!("unknown {name} plugin option: {key}")));
                }
            }
        }
        if builtin.is_some() && options.get("backendFactory").is_some() {
            return Err(invalid(
                "host-storage in the CLI uses session memory; backendFactory requires an embedding host",
            ));
        }
        let source = match builtin {
            Some(source) => source.to_owned(),
            None => crate::read_text(Path::new(&requested))?,
        };
        let source = if builtin.is_some() {
            format!("context => ({source}\n)(context, {options})")
        } else {
            format!("context => ({source}\n)({{...context, options: {options}}})")
        };
        let plugin = Plugin::new(
            if name == "extraction" && builtin.is_some() {
                "defuddle"
            } else {
                &name
            },
            source,
        );
        plugins.push(if name == "host-storage" && builtin.is_some() {
            memory_storage(plugin)
        } else {
            plugin
        });
    }
    if !configuration.is_empty() {
        return Err(invalid(
            "options supplied for a plugin that was not selected",
        ));
    }
    Ok(plugins)
}
fn memory_storage(plugin: Plugin) -> Plugin {
    let origins = Arc::new(Mutex::new(
        BTreeMap::<String, BTreeMap<String, BridgeValue>>::new(),
    ));
    plugin.with_handler(
        "transaction",
        Arc::new(move |call| {
            let result = (|| -> std::result::Result<BridgeValue, String> {
                if call.context.is_cancelled() {
                    return Err("cancelled".into());
                }
                let origin = call
                    .context
                    .origin
                    .as_ref()
                    .ok_or("opaque origins cannot use hostStorage")?;
                let [BridgeValue::Array(operations)] = call.arguments.as_slice() else {
                    return Err("expected operation array".into());
                };
                let mut origins = origins.lock().map_err(|_| "storage lock poisoned")?;
                let mut values = origins.get(origin).cloned().unwrap_or_default();
                let mut results = Vec::new();
                for operation in operations {
                    let BridgeValue::Object(op) = operation else {
                        return Err("expected operation object".into());
                    };
                    let Some(BridgeValue::String(kind)) = op.get("type") else {
                        return Err("expected operation type".into());
                    };
                    let key = match (kind.as_str(), op.get("key")) {
                        ("clear", _) => "",
                        (_, Some(BridgeValue::String(key))) => key,
                        _ => return Err("expected string key".into()),
                    };
                    results.push(match kind.as_str() {
                        "get" => values.get(key).cloned().unwrap_or(BridgeValue::Null),
                        "set" => {
                            values.insert(
                                key.to_owned(),
                                op.get("value").ok_or("missing value")?.clone(),
                            );
                            BridgeValue::Null
                        }
                        "delete" => BridgeValue::Boolean(values.remove(key).is_some()),
                        "clear" => {
                            values.clear();
                            BridgeValue::Null
                        }
                        _ => return Err("unknown storage operation".into()),
                    });
                }
                if call.context.is_cancelled() {
                    return Err("cancelled".into());
                }
                origins.insert(origin.clone(), values);
                Ok(BridgeValue::Array(results))
            })();
            match result {
                Ok(value) => call.completion.resolve(value),
                Err(error) => call.completion.reject(error),
            }
        }),
    )
}

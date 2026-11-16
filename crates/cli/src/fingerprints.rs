//! Explicit catalog IO; runtime requests never read credentials or fingerprint files.

use crate::{Result, args::Fingerprints, internal, invalid};
use brimp_net::{Client, Config, fingerprint::Fingerprint};
use serde_json::{Map, Value, json};
use std::{env, io::Write, path::PathBuf};

fn config_dir() -> Result<PathBuf> {
    if let Some(path) = env::var_os("IMPERSONATE_CONFIG_DIR") {
        return Ok(path.into());
    }
    if cfg!(windows) {
        if let Some(path) = env::var_os("APPDATA") {
            return Ok(PathBuf::from(path).join("impersonate"));
        }
        return env::var_os("USERPROFILE")
            .map(|path| PathBuf::from(path).join("AppData/Roaming/impersonate"))
            .ok_or_else(|| invalid("set IMPERSONATE_CONFIG_DIR or APPDATA"));
    }
    if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(path).join("impersonate"));
    }
    env::var_os("HOME")
        .map(|path| PathBuf::from(path).join(".config/impersonate"))
        .ok_or_else(|| invalid("set IMPERSONATE_CONFIG_DIR or HOME"))
}

fn read_json(name: &str) -> Result<Value> {
    let path = config_dir()?.join(name);
    match std::fs::read_to_string(&path) {
        Ok(source) => serde_json::from_str(&source).map_err(invalid),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(json!({})),
        Err(error) => Err(invalid(format!("{}: {error}", path.display()))),
    }
}

fn write_json(name: &str, value: &Value) -> Result<()> {
    let directory = config_dir()?;
    std::fs::create_dir_all(&directory).map_err(internal)?;
    // NamedTempFile is private (0600 on Unix) and replacement is atomic.
    let mut file = tempfile::NamedTempFile::new_in(&directory).map_err(internal)?;
    serde_json::to_writer_pretty(&mut file, value).map_err(internal)?;
    writeln!(file).map_err(internal)?;
    file.persist(directory.join(name)).map_err(internal)?;
    Ok(())
}

fn catalog() -> Result<Map<String, Value>> {
    read_json("fingerprints.json")?
        .as_object()
        .cloned()
        .ok_or_else(|| invalid("fingerprints.json must contain an object"))
}

fn print_json(value: &Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value).map_err(internal)?);
    Ok(())
}

pub(crate) fn run(command: Fingerprints) -> Result<()> {
    match command {
        Fingerprints::Config { api_key } => {
            if api_key.trim().is_empty() {
                return Err(invalid("API key must not be empty"));
            }
            let mut config = read_json("config.json")?;
            let object = config
                .as_object_mut()
                .ok_or_else(|| invalid("config.json must contain an object"))?;
            object.insert("api_key".into(), Value::String(api_key));
            write_json("config.json", &config)?;
            println!("API key saved.");
            Ok(())
        }
        Fingerprints::Update { api_root } => update(api_root),
        Fingerprints::List => {
            let rows = catalog()?.into_iter().map(|(name, raw)| {
                let fp: Fingerprint = serde_json::from_value(raw).map_err(invalid)?;
                Ok(json!({"name": name, "client": fp.client, "client_version": fp.client_version,
                    "os": fp.os, "os_version": fp.os_version, "http_version": fp.http_version}))
            }).collect::<Result<Vec<_>>>()?;
            print_json(&Value::Array(rows))
        }
        Fingerprints::Show { name, profile } => {
            let raw = catalog()?
                .remove(&name)
                .ok_or_else(|| invalid(format!("unknown downloaded fingerprint: {name}")))?;
            let fp: Fingerprint = serde_json::from_value(raw).map_err(invalid)?;
            let value = serde_json::to_value(fp).map_err(internal)?;
            print_json(&if profile {
                json!({"values": {"network": {"fingerprint": value}}})
            } else {
                value
            })
        }
    }
}

fn update(api_root: Option<String>) -> Result<()> {
    let root = api_root
        .or_else(|| env::var("IMPERSONATE_API_ROOT").ok())
        .filter(|root| !root.is_empty())
        .unwrap_or_else(|| "https://api.impersonate.pro/v1".into());
    let endpoint = format!("{}/fingerprints", root.trim_end_matches('/'));
    let key = env::var("IMPERSONATE_API_KEY")
        .ok()
        .filter(|key| !key.is_empty());
    let key = match key {
        Some(key) => Some(key),
        None => read_json("config.json")?
            .get("api_key")
            .and_then(Value::as_str)
            .map(str::to_owned),
    };
    // This client uses a built-in preset and never depends on the catalog being updated.
    // Redirects stay disabled so credentials remain at the configured API endpoint.
    let client = Client::new(Config::default());
    let mut fingerprints = Map::new();
    let mut skip = 0;
    loop {
        let mut request = http::Request::builder().uri(format!("{endpoint}?skip={skip}&limit=100"));
        if let Some(key) = &key {
            request = request.header(http::header::AUTHORIZATION, format!("Bearer {key}"));
        }
        let response = client
            .send_collect(request.body(None).map_err(invalid)?)
            .map_err(|error| crate::AutomationError::Transport(error.to_string()))?;
        if !response.status.is_success() {
            return Err(invalid(format!(
                "fingerprint endpoint returned HTTP {}",
                response.status
            )));
        }
        let data: Value = serde_json::from_slice(&response.body).map_err(invalid)?;
        let items = data
            .get("items")
            .or_else(|| data.get("data"))
            .and_then(Value::as_array)
            .ok_or_else(|| invalid("fingerprint response must contain an items array"))?;
        for item in items {
            let name = item
                .get("name")
                .or_else(|| item.get("target"))
                .and_then(Value::as_str)
                .filter(|name| !name.is_empty())
                .ok_or_else(|| invalid("fingerprint item has no name"))?;
            let raw = item
                .get("data")
                .or_else(|| item.get("fingerprint"))
                .ok_or_else(|| invalid("fingerprint item has no data"))?;
            let raw = if let Some(source) = raw.as_str() {
                serde_json::from_str(source).map_err(invalid)?
            } else {
                raw.clone()
            };
            let _: Fingerprint = serde_json::from_value(raw.clone()).map_err(invalid)?;
            fingerprints.insert(name.to_owned(), raw);
        }
        if data["pagination"]["has_more"] != true {
            break;
        }
        skip = data["pagination"]["next_skip"]
            .as_u64()
            .filter(|next| *next > skip)
            .ok_or_else(|| invalid("fingerprint pagination must have increasing next_skip"))?;
    }
    let count = fingerprints.len();
    write_json("fingerprints.json", &Value::Object(fingerprints))?;
    println!("Updated {count} fingerprints.");
    Ok(())
}

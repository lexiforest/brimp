use crate::{Budget, Result, args::Capture, internal, invalid, output::Output, plugins, signals};
use brimp_runtime::{
    profile::BrowserProfile,
    runtime::native::{Session, SessionOptions},
};
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    time::Duration,
};
use tiny_http::{Header, Method, Response, Server};

const HTML: &str = include_str!("capture.html");
const REGISTRY: &str = include_str!("../../runtime/js/internal/profile_registry.js");
const RUNNER: &str = include_str!("../../runtime/js/internal/profile_capture.js");
const UI: &str = include_str!("capture_ui.js");
const MAX_PROFILE_BYTES: usize = 8 * 1024 * 1024;

fn script(contract: &Value, selected_plugins: &[String]) -> Result<String> {
    let mut probes = String::from("const probes = Object.create(null);\n");
    for entry in contract["interfaces"]
        .as_array()
        .ok_or_else(|| internal("missing profile interfaces"))?
    {
        if let Some(source) = entry["probe"].as_str() {
            probes.push_str(&format!("probes[{}] = ({source}\n);\n", entry["scope"]));
        }
    }
    let config =
        json!({"toolVersion": env!("CARGO_PKG_VERSION"), "selectedPlugins": selected_plugins});
    Ok(format!(
        "(() => {{\n{REGISTRY}\n{RUNNER}\n{UI}\n{probes}\ninstallCapturePage({contract}, probes, {config});\n}})();\n"
    ))
}

pub fn capture(options: Capture) -> Result<()> {
    if options.output.as_os_str() == "-" {
        return Err(invalid("profile capture requires an output file"));
    }
    let selected = if options.plugins.is_empty() && !options.no_plugins {
        plugins::catalog()
            .as_object()
            .unwrap()
            .iter()
            .filter(|(_, entry)| entry["defaultEnabled"] == true)
            .map(|(name, _)| name.clone())
            .collect()
    } else {
        options.plugins.clone()
    };
    let plugins = plugins::load(&selected, &options.plugin_options, false)?;
    let names = plugins
        .iter()
        .map(|plugin| plugin.name.clone())
        .collect::<Vec<_>>();
    let budget = Budget::new(options.timeout)?;
    let _signals = signals::Signals::install(budget.cancellation.clone())?;
    let session_options = SessionOptions {
        plugins,
        deadline: Some(budget.deadline),
        cancellation: budget.cancellation.clone(),
        ..Default::default()
    };
    let session = Session::new(session_options.clone())?;
    let page = session.new_page()?;
    let contract = page.profile_contract()?;
    let js = script(&contract, &names)?;
    page.close()?;
    if let Some(directory) = &options.directory {
        std::fs::create_dir_all(directory).map_err(internal)?;
        // Check both destinations before creating either file.
        let mut html = Output::new(Some(&directory.join("index.html")), options.overwrite)?;
        let mut script = Output::new(Some(&directory.join("capture.js")), options.overwrite)?;
        html.write(HTML.as_bytes())?;
        script.write(js.as_bytes())?;
        html.finish()?;
        script.finish()?;
        eprintln!("Capture kit exported to {}", directory.display());
    }
    if options.no_serve {
        return Ok(());
    }
    // Reserve the destination now; never truncate an existing profile on a failed capture.
    let mut output = Output::new(Some(&options.output), options.overwrite)?;
    let server = Server::http(("127.0.0.1", options.port)).map_err(internal)?;
    let address = server.server_addr().to_string();
    let origin = format!("http://{address}");
    // tempfile supplies an unpredictable session identifier without a new RNG dependency.
    let token_file = tempfile::Builder::new()
        .prefix("brimp-capture-")
        .rand_bytes(32)
        .tempfile()
        .map_err(internal)?;
    let token = token_file
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    println!("Open {origin}/?token={token}");
    std::io::stdout().flush().map_err(internal)?;
    loop {
        let wait = budget.remaining()?.min(Duration::from_millis(100));
        let Some(mut request) = server.recv_timeout(wait).map_err(internal)? else {
            continue;
        };
        let header = |name: &'static str| {
            request
                .headers()
                .iter()
                .find(|header| header.field.equiv(name))
                .map(|header| header.value.as_str())
        };
        if header("Host") != Some(address.as_str()) {
            respond(
                request,
                403,
                "application/json",
                json!({"error":"invalid host"}).to_string(),
            )?;
            continue;
        }
        let path = request.url().split('?').next().unwrap_or("/").to_owned();
        if request.method() == &Method::Get && (path == "/" || path == "/index.html") {
            respond(request, 200, "text/html; charset=utf-8", HTML.to_owned())?;
        } else if request.method() == &Method::Get && path == "/capture.js" {
            respond(request, 200, "text/javascript; charset=utf-8", js.clone())?;
        } else if request.method() == &Method::Post && path == "/profile" {
            if header("Origin") != Some(origin.as_str())
                || header("X-Brimp-Capture") != Some(token.as_str())
            {
                respond(
                    request,
                    403,
                    "application/json",
                    json!({"error":"invalid capture session"}).to_string(),
                )?;
                continue;
            }
            let validated = (|| -> Result<Value> {
                if request
                    .body_length()
                    .is_some_and(|length| length > MAX_PROFILE_BYTES)
                {
                    return Err(invalid("profile exceeds 8 MiB"));
                }
                let mut body = String::new();
                request
                    .as_reader()
                    .take((MAX_PROFILE_BYTES + 1) as u64)
                    .read_to_string(&mut body)
                    .map_err(invalid)?;
                if body.len() > MAX_PROFILE_BYTES {
                    return Err(invalid("profile exceeds 8 MiB"));
                }
                let profile = BrowserProfile::from_json(&body).map_err(invalid)?;
                // Validate using exactly the selected interfaces, including custom plugins.
                let validation = Session::new(SessionOptions {
                    profile: Some(profile),
                    ..session_options.clone()
                })?;
                let page = validation.new_page()?;
                page.profile_report()?;
                page.close()?;
                serde_json::from_str(&body).map_err(invalid)
            })();
            match validated {
                Ok(profile) => {
                    let bytes = serde_json::to_vec_pretty(&profile).map_err(internal)?;
                    output.write(&bytes)?;
                    output.write(b"\n")?;
                    output.finish()?;
                    respond(
                        request,
                        200,
                        "application/json",
                        json!({"path": options.output}).to_string(),
                    )?;
                    eprintln!("Saved profile to {}", options.output.display());
                    return Ok(());
                }
                Err(error) => respond(
                    request,
                    400,
                    "application/json",
                    json!({"error":error.to_string()}).to_string(),
                )?,
            }
        } else {
            respond(
                request,
                404,
                "application/json",
                json!({"error":"not found"}).to_string(),
            )?;
        }
    }
}

fn respond(
    request: tiny_http::Request,
    status: u16,
    content_type: &str,
    body: String,
) -> Result<()> {
    let response = Response::from_string(body)
        .with_status_code(status)
        .with_header(Header::from_bytes("Content-Type", content_type).unwrap())
        .with_header(Header::from_bytes("Cache-Control", "no-store").unwrap())
        .with_header(Header::from_bytes("X-Content-Type-Options", "nosniff").unwrap());
    request.respond(response).map_err(internal)
}

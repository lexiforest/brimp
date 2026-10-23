mod actions;
mod args;
mod output;
mod plugins;
mod routes;
mod signals;

use args::{Cli, Command, Format, Options};
use brimp_runtime::{
    network::{CurlConfig, Proxy},
    runtime::{
        AutomationError, CancellationToken, ExtractionOptions, NavigationWait,
        clock::{ClockOptions, TimeMode},
        diagnostics::DiagnosticsOptions,
        events::PageEvent,
        native::{IdentityOptions, NativePage, RequestOptions, Session, SessionOptions},
        networking::NetworkOptions,
        routes::{RouteOverrides, RouteStage},
    },
};
use clap::Parser;
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, AutomationError>;
fn invalid(error: impl std::fmt::Display) -> AutomationError {
    AutomationError::InvalidInput(error.to_string())
}
fn internal(error: impl std::fmt::Display) -> AutomationError {
    AutomationError::Internal(error.to_string())
}
fn read_text(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|e| invalid(format!("{}: {e}", path.display())))
}
fn evaluate(page: &NativePage, source: &str) -> Result<Value> {
    page.evaluate(&format!(
        "Promise.resolve((0,eval)({})).then(value => value === undefined ? null : value)",
        json!(source)
    ))
}
struct Budget {
    started: Instant,
    timeout: Duration,
    deadline: Instant,
    cancellation: CancellationToken,
}
impl Budget {
    fn new(timeout: Duration) -> Result<Self> {
        let started = Instant::now();
        let deadline = started
            .checked_add(timeout)
            .ok_or_else(|| invalid("timeout out of range"))?;
        Ok(Self {
            started,
            timeout,
            deadline,
            cancellation: CancellationToken::new(),
        })
    }
    fn remaining(&self) -> Result<Duration> {
        if self.cancellation.is_cancelled() {
            return Err(AutomationError::Cancellation);
        }
        self.timeout
            .checked_sub(self.started.elapsed())
            .filter(|value| !value.is_zero())
            .ok_or(AutomationError::Timeout(self.timeout))
    }
    fn sleep(&self, duration: Duration) -> Result<()> {
        let until = Instant::now()
            .checked_add(duration)
            .ok_or_else(|| invalid("wait out of range"))?;
        while Instant::now() < until {
            std::thread::sleep(
                self.remaining()?
                    .min(until.saturating_duration_since(Instant::now()))
                    .min(Duration::from_millis(5)),
            );
        }
        self.remaining().map(|_| ())
    }
}
fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::from(match error {
                AutomationError::InvalidInput(_) => 2,
                AutomationError::Transport(_) => 10,
                AutomationError::HttpStatus(_) => 11,
                AutomationError::Navigation(_) => 12,
                AutomationError::JavaScript(_) => 13,
                AutomationError::Timeout(_) => 14,
                AutomationError::Cancellation => 15,
                AutomationError::Unsupported(_) => 16,
                AutomationError::Closed | AutomationError::StaleHandle => 17,
                AutomationError::Reentry
                | AutomationError::Extraction(_)
                | AutomationError::Internal(_) => 18,
            })
        }
    }
}
fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Plugins { command } => {
            let catalog = plugins::catalog();
            let value = match command {
                args::Plugins::List => catalog,
                args::Plugins::Show { name } => catalog
                    .get(&name)
                    .cloned()
                    .ok_or_else(|| invalid(format!("unknown plugin: {name}")))?,
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&value).map_err(internal)?
            );
            Ok(())
        }
        Command::Doctor(options) => {
            let budget = Budget::new(options.timeout)?;
            let _signals = signals::Signals::install(budget.cancellation.clone())?;
            let result = (|| {
                let session = Session::new(SessionOptions {
                    persona: persona(options.persona.as_deref())?,
                    timeout: options.timeout,
                    deadline: Some(budget.deadline),
                    cancellation: budget.cancellation.clone(),
                    network: network_config(
                        options.proxy.as_deref(),
                        options.ca_bundle,
                        options.timeout,
                    )?,
                    ..Default::default()
                })?;
                let page = session.new_page()?;
                let checks = page.evaluate("({javascriptCore: 1+1 === 2, lexbor: document.createElement('p').tagName === 'P', crypto: typeof crypto.subtle.digest === 'function'})")?;
                if checks
                    .as_object()
                    .is_none_or(|checks| checks.values().any(|value| value != true))
                {
                    return Err(internal("runtime smoke check failed"));
                }
                page.evaluate("crypto.subtle.digest('SHA-256', new Uint8Array()).then(value => value.byteLength)")?;
                session.close();
                budget.remaining()?;
                println!(
                    "{}",
                    json!({"javascriptCore":"ok", "lexbor":"ok", "libcurlImpersonate":"ok", "crypto":"ok"})
                );
                Ok(())
            })();
            budget.remaining()?;
            result
        }
        Command::Fetch { url, options } => execute(Some(url), None, *options),
        Command::Run {
            script,
            url,
            options,
        } => execute(url, Some(script), *options),
    }
}
fn persona(path: Option<&Path>) -> Result<Option<brimp_runtime::persona::PersonaConfig>> {
    path.map(|path| {
        let value: brimp_runtime::persona::PersonaConfig =
            serde_json::from_str(&read_text(path)?).map_err(invalid)?;
        value.validate().map_err(invalid)?;
        Ok(value)
    })
    .transpose()
}
fn network_config(
    proxy: Option<&str>,
    ca_bundle: Option<PathBuf>,
    timeout: Duration,
) -> Result<CurlConfig> {
    if let Some(path) = &ca_bundle {
        std::fs::File::open(path).map_err(invalid)?;
    }
    Ok(CurlConfig {
        proxy: proxy.map(Proxy::parse).transpose().map_err(invalid)?,
        ca_bundle,
        request_timeout: timeout,
        ..Default::default()
    })
}
fn format(options: &Options, running: bool) -> Result<Option<Format>> {
    if options.eval.is_some() || options.eval_file.is_some() || running {
        if running && options.format.is_some() {
            return Err(invalid(
                "run outputs its JavaScript result; --format applies to fetch",
            ));
        }
        return Ok(None);
    }
    if let Some(format) = options.format {
        return Ok(Some(format));
    }
    match options.output.as_deref() {
        None => Ok(Some(Format::Html)),
        Some(path) => match path
            .extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("html" | "htm") => Ok(Some(Format::Html)),
            Some("md" | "markdown") => Ok(Some(Format::Markdown)),
            Some("json") => Ok(Some(Format::Json)),
            _ => Err(invalid("cannot infer output format; pass --format")),
        },
    }
}
fn headers(options: &Options) -> Result<Vec<(String, String)>> {
    options
        .header
        .iter()
        .map(|header| {
            let (name, value) = header
                .split_once(':')
                .ok_or_else(|| invalid("headers require NAME: VALUE"))?;
            let name = http::HeaderName::from_bytes(name.trim().as_bytes()).map_err(invalid)?;
            if matches!(
                name.as_str(),
                "user-agent" | "accept" | "accept-language" | "accept-encoding"
            ) || name.as_str().starts_with("sec-ch-ua")
            {
                return Err(invalid(format!(
                    "header {name} is owned by the identity/persona"
                )));
            }
            http::HeaderValue::from_str(value.trim()).map_err(invalid)?;
            Ok((name.to_string(), value.trim().to_owned()))
        })
        .collect()
}
fn check_paths(options: &Options) -> Result<()> {
    let mut seen = std::collections::HashSet::new();
    for path in [
        options.output.as_deref(),
        options.response_info.as_deref(),
        options.trace.as_deref(),
        options.events.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        if path == Path::new("-") {
            if options.output.as_deref() != Some(path)
                || options.response_info.as_deref() == Some(path)
                || options.trace.as_deref() == Some(path)
                || options.events.as_deref() == Some(path)
            {
                return Err(invalid("only primary output may use stdout (-)"));
            }
            continue;
        }
        let absolute = std::path::absolute(path).map_err(invalid)?;
        let resolved = if absolute.exists() {
            absolute.canonicalize().map_err(invalid)?
        } else {
            absolute
                .parent()
                .unwrap()
                .canonicalize()
                .map_err(invalid)?
                .join(
                    absolute
                        .file_name()
                        .ok_or_else(|| invalid("invalid output filename"))?,
                )
        };
        if !seen.insert(resolved) {
            return Err(invalid(
                "output, metadata, trace and events need distinct paths",
            ));
        }
        if path.exists() && !options.overwrite {
            return Err(invalid(format!(
                "{} exists; pass --overwrite",
                path.display()
            )));
        }
    }
    Ok(())
}
fn execute(url: Option<String>, script: Option<PathBuf>, options: Options) -> Result<()> {
    let format = format(&options, script.is_some())?;
    let extraction = matches!(format, Some(Format::Markdown | Format::Json));
    let wait_seconds = match options.wait.as_str() {
        "load" | "domcontentloaded" | "networkidle" => None,
        value => Some(actions::seconds_duration(value.parse().map_err(invalid)?)?),
    };
    if options.max_tasks > 1_000_000 || options.trace_capacity > 1_000_000 {
        return Err(invalid("task/trace capacities cannot exceed 1000000"));
    }
    if (options.epoch_ms.is_some() || options.advance_time.is_some())
        && options.clock != args::Clock::Logical
    {
        return Err(invalid(
            "--epoch-ms and --advance-time require --clock logical",
        ));
    }
    if options.stream
        && (script.is_some()
            || format != Some(Format::Raw)
            || !options.script.is_empty()
            || !options.init_script.is_empty()
            || options.actions.is_some()
            || options.advance_time.is_some()
            || options.wait_selector.is_some()
            || options.wait != "load")
    {
        return Err(invalid(
            "--stream requires fetch --format raw and cannot use page scripts, actions or waits",
        ));
    }
    if !extraction
        && (options.content.is_some()
            || options.remove_images
            || options.language.is_some()
            || options.extract_debug)
    {
        return Err(invalid("extraction options require markdown/json output"));
    }
    if let Some(url) = &url {
        url::Url::parse(url).map_err(invalid)?;
    } else if options.method != "GET"
        || options.data.is_some()
        || options.data_file.is_some()
        || options.auth.is_some()
        || !options.cookie.is_empty()
        || options.response_info.is_some()
        || options.fail_on_http_error
    {
        return Err(invalid("request options require a URL"));
    }
    http::Method::from_bytes(options.method.as_bytes()).map_err(invalid)?;
    let request_headers = headers(&options)?;
    let persona = persona(options.persona.as_deref())?;
    let body = match &options.data_file {
        Some(path) => Some(std::fs::read(path).map_err(invalid)?),
        None => options.data.as_ref().map(|value| value.as_bytes().to_vec()),
    };
    let scripts = options
        .script
        .iter()
        .map(|path| read_text(path))
        .collect::<Result<Vec<_>>>()?;
    let init_scripts = options
        .init_script
        .iter()
        .map(|path| read_text(path))
        .collect::<Result<Vec<_>>>()?;
    let expression = match &options.eval_file {
        Some(path) => Some(read_text(path)?),
        None => options.eval.clone(),
    };
    let program = script
        .map(|path| {
            if path == Path::new("-") {
                let mut text = String::new();
                std::io::stdin()
                    .read_to_string(&mut text)
                    .map_err(invalid)?;
                Ok(text)
            } else {
                read_text(&path)
            }
        })
        .transpose()?;
    let actions = actions::Actions::load(options.actions.as_deref())?;
    let routes = routes::Routes::load(options.routes.as_deref())?;
    let plugins = plugins::load(&options.plugin, &options.plugin_options, extraction)?;
    let auth = options
        .auth
        .as_ref()
        .map(|value| {
            value
                .split_once(':')
                .map(|(a, b)| (a.to_owned(), b.to_owned()))
                .ok_or_else(|| invalid("--auth requires USER:PASSWORD"))
        })
        .transpose()?;
    let cookies = options
        .cookie
        .iter()
        .map(|cookie| {
            let (name, value) = cookie
                .split_once('=')
                .ok_or_else(|| invalid("cookies require NAME=VALUE"))?;
            if name.is_empty() {
                return Err(invalid("empty cookie name"));
            }
            Ok((name.to_owned(), value.to_owned()))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut network = network_config(
        options.proxy.as_deref(),
        options.ca_bundle.clone(),
        options.timeout,
    )?;
    network.connect_timeout = options.connect_timeout;
    network.max_response_bytes = options.max_response_bytes;
    check_paths(&options)?;
    let mut output = output::Output::new(options.output.as_deref(), options.overwrite)?;
    let mut info = options
        .response_info
        .as_deref()
        .map(|path| output::Output::new(Some(path), options.overwrite))
        .transpose()?;
    if let Some(path) = &options.trace {
        output::log_file(path, options.overwrite)?;
    }
    let events = options
        .events
        .as_deref()
        .map(|path| output::log_file(path, options.overwrite))
        .transpose()?
        .map(|file| Arc::new(Mutex::new(file)));
    let budget = Budget::new(options.timeout)?;
    let _signals = signals::Signals::install(budget.cancellation.clone())?;
    let result = (|| {
        let session = Session::new(SessionOptions {
            persona,
            deadline: Some(budget.deadline),
            cancellation: budget.cancellation.clone(),
            timeout: budget.remaining()?,
            wait_until: if options.wait == "domcontentloaded" {
                NavigationWait::DomContentLoaded
            } else {
                NavigationWait::Load
            },
            identity: IdentityOptions {
                user_agent: options.user_agent.clone(),
                languages: (!options.languages.is_empty()).then(|| options.languages.clone()),
                platform: options.platform.clone(),
            },
            network,
            headers: request_headers,
            plugins,
            clock: ClockOptions {
                mode: if options.clock == args::Clock::Logical {
                    TimeMode::Logical
                } else {
                    TimeMode::System
                },
                epoch_ms: options.epoch_ms,
            },
            diagnostics: options.trace.as_ref().map(|path| DiagnosticsOptions {
                path: Some(path.to_string_lossy().into_owned()),
                include: options.trace_include.clone(),
                capacity: options.trace_capacity,
                stack: options.trace_stack,
            }),
        })?;
        let page = session.new_page()?;
        let errors = Arc::new(Mutex::new(None::<String>));
        let mut subscriptions = Vec::new();
        for name in [
            "console",
            "error",
            "lifecycle",
            "mutation",
            "request",
            "response",
            "request_finished",
            "request_failed",
            "websocket",
            "eventsource",
        ] {
            let file = events.clone();
            let errors = errors.clone();
            subscriptions.push(page.on(
                name,
                Arc::new(move |event| {
                    match &event {
                        PageEvent::Console { level, message } => eprintln!("[{level}] {message}"),
                        PageEvent::Error { message } => eprintln!("[page error] {message}"),
                        _ => {}
                    }
                    if let Some(file) = &file {
                        let result = (|| -> std::io::Result<()> {
                            let mut file = file.lock().unwrap();
                            serde_json::to_writer(&mut *file, &event)?;
                            file.write_all(b"\n")
                        })();
                        if let Err(error) = result {
                            *errors.lock().unwrap() = Some(error.to_string());
                        }
                    }
                }),
            )?);
        }
        let mut registrations = routes.install(&page, errors.clone())?;
        if let (Some((username, password)), Some(url)) = (auth, &url) {
            let origin = url::Url::parse(url).map_err(invalid)?.origin();
            registrations.push(page.route(
                "*",
                RouteStage::Auth,
                Arc::new(move |route| {
                    let allowed = url::Url::parse(&route.info().request.url)
                        .is_ok_and(|url| url.origin() == origin)
                        && route
                            .info()
                            .response
                            .as_ref()
                            .is_some_and(|response| response.status == 401);
                    let result = if allowed {
                        route.authenticate(username.clone(), password.clone())
                    } else {
                        route.continue_request(RouteOverrides::default())
                    };
                    if result.is_err() && route.is_pending() {
                        let _ = route.fail("authentication failed");
                    }
                }),
            )?);
        }
        page.configure_network(NetworkOptions {
            offline: options.offline,
            latency: options.latency.unwrap_or_default(),
            bypass_cache: options.bypass_cache,
            ..Default::default()
        })?;
        for source in &init_scripts {
            if url.is_some() {
                page.add_init_script(source)?;
            } else {
                evaluate(&page, source)?;
            }
        }
        if let Some(url) = &url {
            for (name, value) in cookies {
                session.cookies().set_cookie(url, &name, &value)?;
            }
        }
        let request = RequestOptions {
            body,
            timeout: Some(budget.remaining()?),
            allow_redirects: !options.no_redirects,
            max_redirects: options.max_redirects,
            cancellation: budget.cancellation.clone(),
            ..Default::default()
        };
        if options.stream {
            let response = page.stream(&options.method, url.as_deref().unwrap(), request)?;
            if let Some(info) = &mut info {
                info.write(&serde_json::to_vec_pretty(&response.metadata).map_err(internal)?)?;
            }
            if options.fail_on_http_error && response.metadata.status >= 400 {
                if let Some(info) = info.take() {
                    info.finish()?;
                }
                return Err(AutomationError::HttpStatus(response.metadata.status));
            }
            let mut size = 0usize;
            while let Some(chunk) = response.body.next_chunk()? {
                budget.remaining()?;
                size = size
                    .checked_add(chunk.len())
                    .ok_or_else(|| internal("stream byte count overflow"))?;
                if size > options.max_response_bytes {
                    return Err(AutomationError::Transport(
                        "response exceeds --max-response-bytes".into(),
                    ));
                }
                output.write(&chunk)?;
            }
        } else {
            let response = url
                .as_ref()
                .map(|url| page.request(&options.method, url, request))
                .transpose()?;
            if let Some(response) = &response {
                if let Some(info) = &mut info {
                    info.write(&serde_json::to_vec_pretty(response).map_err(internal)?)?;
                }
                if options.fail_on_http_error && response.status_code >= 400 {
                    if let Some(info) = info.take() {
                        info.finish()?;
                    }
                    return Err(AutomationError::HttpStatus(response.status_code));
                }
            }
            if options.wait == "networkidle" {
                page.wait_for_network_idle(
                    options.network_idle,
                    budget.remaining()?,
                    budget.cancellation.clone(),
                )?;
            }
            if let Some(seconds) = wait_seconds {
                budget.sleep(seconds)?;
            }
            if let Some(selector) = &options.wait_selector {
                page.wait_for_selector(selector, budget.remaining()?, budget.cancellation.clone())?;
            }
            for source in scripts {
                evaluate(&page, &source)?;
            }
            let mut value = if let Some(program) = program {
                evaluate(&page, &program)?
            } else {
                Value::Null
            };
            actions.run(&page, &budget, options.max_tasks)?;
            if let Some(duration) = options.advance_time {
                page.advance_time(duration.as_secs_f64() * 1000., options.max_tasks)?;
            }
            let bytes = if let Some(expression) = expression {
                value = evaluate(&page, &expression)?;
                json_bytes(&value)?
            } else {
                match format {
                    None => json_bytes(&value)?,
                    Some(Format::Raw) => {
                        response
                            .ok_or_else(|| invalid("raw output requires URL"))?
                            .content
                    }
                    Some(Format::Html) => page.html()?.into_bytes(),
                    Some(Format::Markdown | Format::Json) => {
                        let document = page.extract(ExtractionOptions {
                            content_selector: options.content.clone(),
                            remove_images: options.remove_images,
                            language: options.language.clone(),
                            debug: options.extract_debug,
                        })?;
                        if format == Some(Format::Markdown) {
                            document
                                .content_markdown
                                .ok_or_else(|| {
                                    AutomationError::Extraction(
                                        "Defuddle returned no Markdown".into(),
                                    )
                                })?
                                .into_bytes()
                        } else {
                            serde_json::to_vec_pretty(&document).map_err(internal)?
                        }
                    }
                }
            };
            budget.remaining()?;
            output.write(&bytes)?;
        }
        if options.trace.is_some() {
            let diagnostics = page.take_diagnostics()?;
            if let Some(error) = diagnostics["writeError"].as_str() {
                return Err(internal(error));
            }
        }
        session.close();
        drop(registrations);
        drop(subscriptions);
        if let Some(error) = errors.lock().unwrap().take() {
            return Err(internal(error));
        }
        budget.remaining()?;
        output.finish()?;
        if let Some(info) = info {
            info.finish()?;
        }
        Ok(())
    })();
    budget.remaining()?;
    result
}
fn json_bytes(value: &Value) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec(value).map_err(internal)?;
    bytes.push(b'\n');
    Ok(bytes)
}

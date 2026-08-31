use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use brimp_cdp::{ServerConfig, ServerError, parse_bind, start};
use web_runtime::{AutomationBrowser, AutomationError, CancellationToken, PageOptions};

mod common;
mod crawl;

static INTERRUPTED: AtomicBool = AtomicBool::new(false);
#[cfg(unix)]
unsafe extern "C" {
    fn signal(number: i32, handler: extern "C" fn(i32)) -> usize;
}
#[cfg(unix)]
extern "C" fn interrupt(_: i32) {
    INTERRUPTED.store(true, Ordering::Release);
}

fn main() -> ExitCode {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    #[cfg(unix)]
    if arguments.first().map(String::as_str) != Some("cdp") {
        unsafe {
            signal(2, interrupt);
        }
    }
    match run(arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(exit_code(&error))
        }
    }
}

fn run(arguments: Vec<String>) -> Result<(), AutomationError> {
    let Some(command) = arguments.first().map(String::as_str) else {
        return Err(AutomationError::InvalidInput(usage()));
    };
    match command {
        "doctor" => doctor(),
        "cdp" => cdp_command(&arguments[1..]),
        "get" => get_command(&arguments[1..]),
        "crawl" => crawl::run(&arguments[1..]),
        "--help" | "-h" => print_help(None),
        "help" => print_help(arguments.get(1).map(String::as_str)),
        command => Err(AutomationError::InvalidInput(format!(
            "unknown command `{command}`\n{}",
            usage()
        ))),
    }
}

fn print_help(command: Option<&str>) -> Result<(), AutomationError> {
    let help = match command {
        None => usage(),
        Some("get") => get_usage().into(),
        Some("crawl") => crawl::usage().into(),
        Some("cdp") => cdp_usage().into(),
        Some("doctor") => "usage: brimp doctor".into(),
        Some(command) => {
            return Err(AutomationError::InvalidInput(format!(
                "unknown command `{command}`\n{}",
                usage()
            )));
        }
    };
    println!("{help}");
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputFormat {
    Raw,
    Html,
    Markdown,
    Json,
    Png,
}

impl OutputFormat {
    fn parse(value: &str) -> Result<Self, AutomationError> {
        match value {
            "raw" => Ok(Self::Raw),
            "html" => Ok(Self::Html),
            "markdown" => Ok(Self::Markdown),
            "json" => Ok(Self::Json),
            "png" => Ok(Self::Png),
            _ => Err(AutomationError::InvalidInput(format!(
                "unknown output format `{value}`"
            ))),
        }
    }

    fn infer(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
            "html" | "htm" => Some(Self::Html),
            "md" | "markdown" => Some(Self::Markdown),
            "json" => Some(Self::Json),
            "png" => Some(Self::Png),
            _ => None,
        }
    }
}

struct GetOptions {
    url: String,
    format: Option<OutputFormat>,
    output: Option<PathBuf>,
    overwrite: bool,
    full_page: bool,
    expression: Option<String>,
    expression_file: Option<PathBuf>,
    navigation: common::NavigationOptions,
}

#[derive(Clone, Copy, Debug)]
enum WaitCondition {
    DomContentLoaded,
    Load,
    NetworkIdle,
    Fixed(Duration),
}

impl WaitCondition {
    fn parse(value: &str) -> Result<Self, AutomationError> {
        match value {
            "domcontentloaded" => Ok(Self::DomContentLoaded),
            "load" => Ok(Self::Load),
            "networkidle" => Ok(Self::NetworkIdle),
            value => {
                let seconds = value.parse::<f64>().map_err(|_| {
                    AutomationError::InvalidInput(format!("unknown wait condition `{value}`"))
                })?;
                if !seconds.is_finite() || seconds < 0.0 {
                    return Err(AutomationError::InvalidInput(
                        "fixed wait must be a finite non-negative number of seconds".into(),
                    ));
                }
                Ok(Self::Fixed(Duration::from_secs_f64(seconds)))
            }
        }
    }
}

impl GetOptions {
    fn parse(arguments: &[String]) -> Result<Option<Self>, AutomationError> {
        let mut parser = pico_args::Arguments::from_vec(
            arguments.iter().map(OsString::from).collect::<Vec<_>>(),
        );
        if parser.contains(["-h", "--help"]) {
            println!("{}", get_usage());
            return Ok(None);
        }
        let format = parser
            .opt_value_from_str::<_, String>("--format")
            .map_err(argument_error)?
            .map(|value| OutputFormat::parse(&value))
            .transpose()?;
        let output = parser
            .opt_value_from_os_str("--output", |value| {
                Ok::<_, pico_args::Error>(PathBuf::from(value))
            })
            .map_err(argument_error)?;
        let overwrite = parser.contains("--overwrite");
        let full_page = parser.contains("--full-page");
        let expression = parser
            .opt_value_from_str("--eval")
            .map_err(argument_error)?;
        let expression_file = parser
            .opt_value_from_os_str("--eval-file", |value| {
                Ok::<_, pico_args::Error>(PathBuf::from(value))
            })
            .map_err(argument_error)?;
        let navigation = common::NavigationOptions::parse(&mut parser)?;
        let url = parser.free_from_str::<String>().map_err(argument_error)?;
        let remaining = parser.finish();
        if !remaining.is_empty() {
            return Err(AutomationError::InvalidInput(format!(
                "unknown get argument `{}`",
                remaining[0].to_string_lossy()
            )));
        }
        if expression.is_some() && expression_file.is_some() {
            return Err(AutomationError::InvalidInput(
                "--eval and --eval-file are mutually exclusive".into(),
            ));
        }
        if (expression.is_some() || expression_file.is_some())
            && (format.is_some() || output.is_some())
        {
            return Err(AutomationError::InvalidInput(
                "evaluation cannot be combined with --format or --output".into(),
            ));
        }
        if output.as_deref() == Some(Path::new("-")) && format.is_none() {
            return Err(AutomationError::InvalidInput(
                "--output - requires --format".into(),
            ));
        }
        Ok(Some(Self {
            url,
            format,
            output,
            overwrite,
            full_page,
            expression,
            expression_file,
            navigation,
        }))
    }

    fn output_format(&self) -> Result<OutputFormat, AutomationError> {
        if let Some(format) = self.format {
            return Ok(format);
        }
        match self.output.as_deref() {
            Some(path) => OutputFormat::infer(path).ok_or_else(|| {
                AutomationError::InvalidInput(format!(
                    "cannot infer output format from `{}`; pass --format",
                    path.display()
                ))
            }),
            None => Ok(OutputFormat::Html),
        }
    }
}

fn get_command(arguments: &[String]) -> Result<(), AutomationError> {
    let Some(options) = GetOptions::parse(arguments)? else {
        return Ok(());
    };
    let result_is_evaluation = options.expression.is_some() || options.expression_file.is_some();
    let output_format = (!result_is_evaluation)
        .then(|| options.output_format())
        .transpose()?;
    if let Some(path) = options
        .output
        .as_deref()
        .filter(|path| *path != Path::new("-"))
        && path.exists()
        && !options.overwrite
    {
        return Err(AutomationError::InvalidInput(format!(
            "output `{}` exists; pass --overwrite to replace it",
            path.display()
        )));
    }
    let shared = &options.navigation;
    let started = Instant::now();
    let interrupt = InterruptMonitor::new();
    let browser = AutomationBrowser::with_persona_and_network_config(
        shared.persona.clone(),
        shared.network.clone(),
    )?;
    let context = browser.default_context();
    for (name, value) in &shared.cookies {
        context.set_cookie(&options.url, name, value)?;
    }
    let page = browser.new_page(shared.page.clone())?;
    let navigation = page.navigate_cancellable(
        options.url.clone(),
        remaining_timeout(started, shared.timeout)?,
        interrupt.token(),
    )?;
    match shared.wait {
        WaitCondition::DomContentLoaded | WaitCondition::Load => {}
        WaitCondition::NetworkIdle => page.wait_for_network_idle(
            shared.network_idle,
            remaining_timeout(started, shared.timeout)?,
            interrupt.token(),
        )?,
        WaitCondition::Fixed(duration) => {
            wait_fixed(duration, started, shared.timeout, interrupt.token())?
        }
    }
    if let Some(selector) = &shared.wait_selector {
        page.wait_for_selector(
            selector.clone(),
            remaining_timeout(started, shared.timeout)?,
            interrupt.token(),
        )?;
    }
    check_interrupted()?;
    for source in &shared.scripts {
        remaining_timeout(started, shared.timeout)?;
        let encoded = serde_json::to_string(&source)
            .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
        page.evaluate(format!("(0, eval)({encoded}); null"))?;
        check_interrupted()?;
    }
    remaining_timeout(started, shared.timeout)?;
    let bytes = if let Some(expression) = options.expression {
        let value = page.evaluate(expression)?;
        let mut bytes = serde_json::to_vec(&value)
            .map_err(|error| AutomationError::Internal(error.to_string()))?;
        bytes.push(b'\n');
        bytes
    } else if let Some(path) = options.expression_file {
        let expression = std::fs::read_to_string(&path).map_err(|error| {
            AutomationError::InvalidInput(format!(
                "cannot read expression `{}`: {error}",
                path.display()
            ))
        })?;
        let value = page.evaluate(expression)?;
        let mut bytes = serde_json::to_vec(&value)
            .map_err(|error| AutomationError::Internal(error.to_string()))?;
        bytes.push(b'\n');
        bytes
    } else {
        match output_format.expect("non-evaluation output format was resolved") {
            OutputFormat::Raw => navigation.content,
            OutputFormat::Html => page
                .evaluate("document.documentElement.outerHTML")?
                .as_str()
                .ok_or_else(|| AutomationError::Internal("rendered HTML was not a string".into()))?
                .as_bytes()
                .to_vec(),
            OutputFormat::Markdown => page
                .extract(shared.extraction.clone())?
                .content_markdown
                .ok_or_else(|| AutomationError::Extraction("Defuddle returned no Markdown".into()))?
                .into_bytes(),
            OutputFormat::Json => serde_json::to_vec(&page.extract(shared.extraction.clone())?)
                .map_err(|error| AutomationError::Internal(error.to_string()))?,
            OutputFormat::Png => page.screenshot(options.full_page)?,
        }
    };
    check_interrupted()?;
    remaining_timeout(started, shared.timeout)?;
    write_output(options.output.as_deref(), &bytes, options.overwrite)?;
    page.close();
    browser.close();
    Ok(())
}

fn check_interrupted() -> Result<(), AutomationError> {
    if INTERRUPTED.load(Ordering::Acquire) {
        Err(AutomationError::Cancellation)
    } else {
        Ok(())
    }
}

struct InterruptMonitor {
    token: CancellationToken,
    finished: std::sync::Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl InterruptMonitor {
    fn new() -> Self {
        INTERRUPTED.store(false, Ordering::Release);
        let token = CancellationToken::new();
        let monitor_token = token.clone();
        let finished = std::sync::Arc::new(AtomicBool::new(false));
        let monitor_finished = std::sync::Arc::clone(&finished);
        let worker = std::thread::spawn(move || {
            while !monitor_finished.load(Ordering::Acquire) {
                if INTERRUPTED.load(Ordering::Acquire) {
                    monitor_token.cancel();
                    break;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        });
        Self {
            token,
            finished,
            worker: Some(worker),
        }
    }

    fn token(&self) -> CancellationToken {
        self.token.clone()
    }
}

impl Drop for InterruptMonitor {
    fn drop(&mut self) {
        self.finished.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn remaining_timeout(started: Instant, timeout: Duration) -> Result<Duration, AutomationError> {
    timeout
        .checked_sub(started.elapsed())
        .filter(|remaining| !remaining.is_zero())
        .ok_or(AutomationError::Timeout(timeout))
}

fn wait_fixed(
    duration: Duration,
    started: Instant,
    timeout: Duration,
    cancellation: CancellationToken,
) -> Result<(), AutomationError> {
    let wait_started = Instant::now();
    while wait_started.elapsed() < duration {
        if cancellation.is_cancelled() {
            return Err(AutomationError::Cancellation);
        }
        remaining_timeout(started, timeout)?;
        let remaining = duration.saturating_sub(wait_started.elapsed());
        std::thread::sleep(remaining.min(Duration::from_millis(5)));
    }
    Ok(())
}

fn write_output(path: Option<&Path>, bytes: &[u8], overwrite: bool) -> Result<(), AutomationError> {
    if path.is_none() || path == Some(Path::new("-")) {
        let mut stdout = io::stdout().lock();
        stdout
            .write_all(bytes)
            .and_then(|_| stdout.flush())
            .map_err(|error| AutomationError::Internal(format!("cannot write stdout: {error}")))?;
        return Ok(());
    }
    let path = path.expect("file output path was checked");
    if path.exists() && !overwrite {
        return Err(AutomationError::InvalidInput(format!(
            "output `{}` exists; pass --overwrite to replace it",
            path.display()
        )));
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|error| {
        AutomationError::Internal(format!(
            "cannot create output beside `{}`: {error}",
            path.display()
        ))
    })?;
    temporary
        .write_all(bytes)
        .and_then(|_| temporary.flush())
        .map_err(|error| {
            AutomationError::Internal(format!("cannot write output `{}`: {error}", path.display()))
        })?;
    if overwrite {
        temporary.persist(path)
    } else {
        temporary.persist_noclobber(path)
    }
    .map_err(|error| {
        AutomationError::Internal(format!(
            "cannot persist output `{}`: {}",
            path.display(),
            error.error
        ))
    })?;
    Ok(())
}

fn parse_duration(value: &str) -> Result<Duration, AutomationError> {
    let (number, multiplier) = if let Some(value) = value.strip_suffix("ms") {
        (value, 1_u64)
    } else if let Some(value) = value.strip_suffix('s') {
        (value, 1_000)
    } else if let Some(value) = value.strip_suffix('m') {
        (value, 60_000)
    } else {
        return Err(AutomationError::InvalidInput(
            "duration must end in ms, s, or m".into(),
        ));
    };
    let amount = number
        .parse::<u64>()
        .map_err(|_| AutomationError::InvalidInput(format!("invalid duration `{value}`")))?;
    let millis = amount
        .checked_mul(multiplier)
        .filter(|value| *value > 0)
        .ok_or_else(|| {
            AutomationError::InvalidInput("duration must be positive and in range".into())
        })?;
    Ok(Duration::from_millis(millis))
}

fn argument_error(error: pico_args::Error) -> AutomationError {
    AutomationError::InvalidInput(error.to_string())
}

fn parse_header(value: &str) -> Result<(String, String), AutomationError> {
    let (name, value) = value
        .split_once(':')
        .ok_or_else(|| AutomationError::InvalidInput("--header requires `NAME: VALUE`".into()))?;
    let name = http::HeaderName::from_bytes(name.trim().as_bytes())
        .map_err(|error| AutomationError::InvalidInput(format!("invalid header name: {error}")))?;
    if [
        "user-agent",
        "accept",
        "accept-language",
        "accept-encoding",
        "sec-ch-ua",
        "sec-ch-ua-mobile",
        "sec-ch-ua-platform",
        "sec-ch-ua-full-version",
        "sec-ch-ua-full-version-list",
        "sec-ch-ua-arch",
        "sec-ch-ua-bitness",
        "sec-ch-ua-platform-version",
        "sec-ch-ua-model",
    ]
    .contains(&name.as_str())
    {
        return Err(AutomationError::InvalidInput(format!(
            "header `{name}` is owned by the persona"
        )));
    }
    let value = value.trim();
    http::HeaderValue::from_str(value)
        .map_err(|error| AutomationError::InvalidInput(format!("invalid header value: {error}")))?;
    Ok((name.as_str().to_owned(), value.to_owned()))
}

fn get_usage() -> &'static str {
    "usage: brimp get URL [--format raw|html|markdown|json|png] [--output PATH|-] [--overwrite] [--timeout DURATION]\n       brimp get URL --eval EXPRESSION\n       brimp get URL --eval-file PATH\n\nEXTRACTION:\n  --content SELECTOR\n  --remove-images\n  --language BCP47\n  --extract-debug\n\nWAITING:\n  --wait domcontentloaded|load|networkidle|SECONDS\n  --wait-selector SELECTOR\n  --network-idle DURATION\n\nNETWORK AND IDENTITY:\n  --proxy URL\n  --header 'NAME: VALUE' (repeatable)\n  --cookie 'NAME=VALUE' (repeatable)\n  --persona PATH\n  --ca-bundle PATH\n\nACTIONS:\n  --script PATH (repeatable)\n  --full-page"
}

fn cdp_command(arguments: &[String]) -> Result<(), AutomationError> {
    let mut parser =
        pico_args::Arguments::from_vec(arguments.iter().map(OsString::from).collect::<Vec<_>>());
    if parser.contains(["-h", "--help"]) {
        println!("{}", cdp_usage());
        return Ok(());
    }
    let bind = parser
        .opt_value_from_str::<_, String>("--bind")
        .map_err(argument_error)?
        .unwrap_or_else(|| "127.0.0.1:9222".into());
    let allow_non_loopback = parser.contains("--allow-non-loopback");
    let page_options = common::PageFeatures::parse(&mut parser)?.build(Vec::new())?;
    let remaining = parser.finish();
    if !remaining.is_empty() {
        return Err(AutomationError::InvalidInput(format!(
            "unknown cdp argument `{}`",
            remaining[0].to_string_lossy()
        )));
    }
    let bind = parse_bind(&bind).map_err(AutomationError::InvalidInput)?;
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| AutomationError::Internal(error.to_string()))?;
    runtime.block_on(async move {
        let server = start(ServerConfig {
            bind,
            allow_non_loopback,
            page_options,
        })
        .await
        .map_err(cdp_error)?;
        println!("{}", server.browser_websocket_url());
        std::future::pending::<()>().await;
        #[allow(unreachable_code)]
        Ok(())
    })
}

fn cdp_usage() -> &'static str {
    "usage: brimp cdp [--bind HOST:PORT] [--allow-non-loopback] [PAGE OPTIONS]\n\nPAGE OPTIONS:\n  --enable-worker\n  --enable-streaming-networking\n  --storage-path PATH [--storage-quota-bytes N]\n  --enable-canvas\n  --enable-webgl\n  --enable-webgpu\n  --enable-webaudio\n  --enable-webaudio-output"
}

fn cdp_error(error: ServerError) -> AutomationError {
    match error {
        ServerError::NonLoopback(_) => AutomationError::InvalidInput(error.to_string()),
        _ => AutomationError::Internal(error.to_string()),
    }
}

fn doctor() -> Result<(), AutomationError> {
    let profile = persona::PersonaConfig::default()
        .resolve()
        .transport_profile;
    let config = network::CurlConfig {
        impersonation_profile: profile.clone(),
        ..network::CurlConfig::default()
    };
    network::CurlResourceLoader::check_profile(&config)
        .map_err(|error| AutomationError::Transport(error.to_string()))?;
    let browser = AutomationBrowser::new()?;
    let page = browser.new_page(PageOptions::default())?;
    page.close();
    browser.close();
    println!(
        "{}",
        serde_json::json!({"javascriptCore": "ok", "libcurlImpersonate": "ok", "profile": profile})
    );
    Ok(())
}

fn exit_code(error: &AutomationError) -> u8 {
    match error {
        AutomationError::InvalidInput(_) => 2,
        AutomationError::Transport(_) => 10,
        AutomationError::HttpStatus(_) => 11,
        AutomationError::Navigation(_) => 12,
        AutomationError::JavaScript(_) => 13,
        AutomationError::Timeout(_) => 14,
        AutomationError::Cancellation => 15,
        AutomationError::Unsupported(_) => 16,
        AutomationError::Closed => 17,
        AutomationError::Screenshot(_)
        | AutomationError::Extraction(_)
        | AutomationError::Internal(_) => 18,
    }
}
fn usage() -> String {
    "usage: brimp doctor | brimp get URL [OPTIONS] | brimp crawl URL [OPTIONS] | brimp cdp [--bind HOST:PORT] [--allow-non-loopback] [PAGE OPTIONS] | brimp help [COMMAND]\n\nRun `brimp help COMMAND` for command-specific options.\n\nPAGE OPTIONS:\n  --enable-worker\n  --enable-streaming-networking\n  --storage-path PATH [--storage-quota-bytes N]\n  --enable-canvas\n  --enable-webgl\n  --enable-webgpu\n  --enable-webaudio\n  --enable-webaudio-output".into()
}

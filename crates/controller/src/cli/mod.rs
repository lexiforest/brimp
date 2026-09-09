use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use brimp_controller::{AutomationError, CancellationToken};

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

pub fn main() -> ExitCode {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    #[cfg(unix)]
    {
        unsafe {
            signal(2, interrupt);
            signal(15, interrupt);
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
        "doctor" => doctor(&arguments[1..]),
        "serve" => serve(&arguments[1..]),
        "fetch" => fetch_command(&arguments[1..]),
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
        Some("fetch") => fetch_usage().into(),
        Some("crawl") => crawl::usage().into(),
        Some("doctor") => "usage: brimp doctor [--worker-path PATH] [--cdp] [--worker-args PATH] [--worker-arg ARG]".into(),
        Some("serve") => brimp_controller::server::SERVE_USAGE.into(),
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

struct FetchOptions {
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

impl FetchOptions {
    fn parse(arguments: &[String]) -> Result<Option<Self>, AutomationError> {
        let mut parser = pico_args::Arguments::from_vec(
            arguments.iter().map(OsString::from).collect::<Vec<_>>(),
        );
        if parser.contains(["-h", "--help"]) {
            println!("{}", fetch_usage());
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
                "unknown fetch argument `{}`",
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

fn fetch_command(arguments: &[String]) -> Result<(), AutomationError> {
    let Some(options) = FetchOptions::parse(arguments)? else {
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
    let browser = shared.worker.open(
        shared.config.clone(),
        remaining_timeout(started, shared.timeout)?,
        interrupt.token(),
        matches!(shared.wait, WaitCondition::DomContentLoaded),
    )?;
    let context = browser.default_context();
    for (name, value) in &shared.cookies {
        context.set_cookie(&options.url, name, value)?;
    }
    let page = browser.new_page()?;
    let navigation = page.navigate_cancellable(
        options.url.clone(),
        interrupt.token(),
        matches!(output_format, Some(OutputFormat::Raw)),
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

fn fetch_usage() -> &'static str {
    "usage: brimp fetch URL [--worker-path PATH] [--format raw|html|markdown|json|png] [--output PATH|-] [--overwrite] [--timeout DURATION]\n       brimp fetch URL --eval EXPRESSION\n       brimp fetch URL --eval-file PATH\n\nWORKER:\n  --worker-path PATH (or BRIMP_WORKER_PATH; macOS only)\n  --cdp (launch a WebSocket CDP worker)\n  --worker-args PATH (JSON array of launch arguments)\n  --worker-arg ARG (repeatable)\n\nEXTRACTION:\n  --content SELECTOR\n  --remove-images\n  --language BCP47\n  --extract-debug\n\nWAITING:\n  --wait domcontentloaded|load|networkidle|SECONDS\n  --wait-selector SELECTOR\n  --network-idle DURATION\n\nNETWORK AND IDENTITY:\n  --proxy URL\n  --header 'NAME: VALUE' (repeatable)\n  --cookie 'NAME=VALUE' (repeatable)\n  --persona PATH\n  --ca-bundle PATH\n\nACTIONS:\n  --script PATH (repeatable)\n  --full-page"
}

fn doctor(arguments: &[String]) -> Result<(), AutomationError> {
    let mut parser = pico_args::Arguments::from_vec(arguments.iter().map(OsString::from).collect());
    if parser.contains(["-h", "--help"]) {
        return print_help(Some("doctor"));
    }
    let worker = common::Worker::parse(&mut parser)?;
    if !parser.finish().is_empty() {
        return Err(AutomationError::InvalidInput(
            "unknown doctor arguments".into(),
        ));
    }
    let interrupt = InterruptMonitor::new();
    let browser = worker.open(
        Default::default(),
        Duration::from_secs(30),
        interrupt.token(),
        false,
    )?;
    println!("{}", browser.doctor()?);
    Ok(())
}

fn serve(arguments: &[String]) -> Result<(), AutomationError> {
    if arguments.iter().any(|arg| arg == "--help" || arg == "-h") {
        return print_help(Some("serve"));
    }
    #[cfg(not(target_os = "macos"))]
    return Err(AutomationError::Unsupported(
        "serve process supervision is supported only on macOS".into(),
    ));
    #[cfg(target_os = "macos")]
    {
        let interrupt = InterruptMonitor::new();
        brimp_controller::server::run(arguments.iter().cloned(), interrupt.token().flag()).map_err(
            |error| {
                if interrupt.token().is_cancelled() {
                    AutomationError::Cancellation
                } else {
                    AutomationError::InvalidInput(error)
                }
            },
        )
    }
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
    "usage: brimp serve [OPTIONS] | brimp doctor [OPTIONS] | brimp fetch URL [OPTIONS] | brimp crawl URL [OPTIONS] | brimp help [COMMAND]\n\nRun `brimp help COMMAND` for command-specific options.\n\nWORKER:\n  --worker-path PATH (or BRIMP_WORKER_PATH)\n  --cdp (launch a WebSocket CDP worker)\n  --worker-args PATH (JSON array of launch arguments)\n  --worker-arg ARG (repeatable)\n  Local worker launch and serve require macOS.\n\nPAGE OPTIONS:\n  --storage-path PATH [--storage-quota-bytes N]".into()
}

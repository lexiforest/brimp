use clap::{Args, Parser, Subcommand, ValueEnum};
use std::{path::PathBuf, time::Duration};

#[derive(Parser)]
#[command(
    name = "brimp",
    version,
    about = "A JavaScript runtime for JavaScript from the browser"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}
#[derive(Subcommand)]
pub enum Command {
    /// Verify native initialization and basic DOM/JavaScript/crypto functionality.
    Doctor(Doctor),
    /// Fetch a URL, execute page scripts, and output its live DOM or an evaluated result.
    Fetch {
        url: String,
        #[command(flatten)]
        options: Box<Options>,
    },
    /// Execute a classic/bundled JavaScript file (or - for stdin) in a browser realm.
    Run {
        #[arg(id = "program")]
        script: PathBuf,
        #[arg(long)]
        url: Option<String>,
        #[command(flatten)]
        options: Box<Options>,
    },
    /// Inspect available plugins without creating a page.
    Plugins {
        #[command(subcommand)]
        command: Plugins,
    },
}
#[derive(Subcommand)]
pub enum Plugins {
    List,
    Show { name: String },
}
#[derive(Args)]
pub struct Doctor {
    /// Load a validated browser persona JSON file.
    #[arg(long)]
    pub persona: Option<PathBuf>,
    /// HTTP/SOCKS proxy URL.
    #[arg(long)]
    pub proxy: Option<String>,
    /// PEM certificate authority bundle.
    #[arg(long)]
    pub ca_bundle: Option<PathBuf>,
    #[arg(long, default_value = "30s", value_parser = duration)]
    pub timeout: Duration,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Format {
    Raw,
    Html,
    Markdown,
    Json,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Clock {
    System,
    Logical,
}
#[derive(Args)]
pub struct Options {
    /// Raw response, live DOM, or Defuddle output. Fetch defaults to HTML.
    #[arg(long, value_enum, conflicts_with_all = ["eval", "eval_file"])]
    pub format: Option<Format>,
    /// Write the result to a file, or use - for stdout.
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Replace existing output and diagnostic files.
    #[arg(long)]
    pub overwrite: bool,
    /// Evaluate JavaScript after scripts/actions and write the JSON result.
    #[arg(long, conflicts_with = "eval_file")]
    pub eval: Option<String>,
    /// Read final evaluation JavaScript from a file.
    #[arg(long)]
    pub eval_file: Option<PathBuf>,
    /// Post-navigation preparation scripts, in order.
    #[arg(long)]
    pub script: Vec<PathBuf>,
    /// Run before document scripts on each navigation.
    #[arg(long)]
    pub init_script: Vec<PathBuf>,
    /// CSS selector for extraction content (Markdown/JSON output only).
    #[arg(long)]
    pub content: Option<String>,
    /// Remove images from extracted Markdown/JSON.
    #[arg(long)]
    pub remove_images: bool,
    /// BCP47 language hint for extraction.
    #[arg(long)]
    pub language: Option<String>,
    /// Include extraction debugging details.
    #[arg(long)]
    pub extract_debug: bool,
    /// domcontentloaded, load, networkidle, or nonnegative seconds.
    #[arg(long, default_value = "load")]
    pub wait: String,
    /// Wait for a matching DOM element; this does not check visibility.
    #[arg(long)]
    pub wait_selector: Option<String>,
    /// Quiet interval used by network-idle waiting.
    #[arg(long, default_value = "500ms", value_parser = duration)]
    pub network_idle: Duration,
    /// Wall-clock deadline for the entire command.
    #[arg(long, default_value = "30s", value_parser = duration)]
    pub timeout: Duration,
    /// Add a request header as NAME: VALUE; repeat for multiple headers.
    #[arg(long)]
    pub header: Vec<String>,
    /// Seed a session cookie as NAME=VALUE for the target URL.
    #[arg(long)]
    pub cookie: Vec<String>,
    /// HTTP/SOCKS proxy URL.
    #[arg(long)]
    pub proxy: Option<String>,
    /// PEM certificate authority bundle.
    #[arg(long)]
    pub ca_bundle: Option<PathBuf>,
    /// Load a validated browser persona JSON file.
    #[arg(long, conflicts_with_all = ["user_agent", "languages", "platform"])]
    pub persona: Option<PathBuf>,
    /// Override the browser user agent; cannot accompany a persona file.
    #[arg(long)]
    pub user_agent: Option<String>,
    /// Comma-separated navigator languages; cannot accompany a persona file.
    #[arg(long, value_delimiter = ',')]
    pub languages: Vec<String>,
    /// Override navigator.platform; cannot accompany a persona file.
    #[arg(long)]
    pub platform: Option<String>,
    /// HTTP method. Supplying data does not automatically change it.
    #[arg(long, default_value = "GET")]
    pub method: String,
    /// Request body as text.
    #[arg(long, conflicts_with = "data_file")]
    pub data: Option<String>,
    /// Read the request body from a file.
    #[arg(long)]
    pub data_file: Option<PathBuf>,
    /// HTTP authentication credentials USER:PASSWORD, scoped to the target origin.
    #[arg(long)]
    pub auth: Option<String>,
    /// Return the first response without following redirects.
    #[arg(long)]
    pub no_redirects: bool,
    /// Maximum number of redirects to follow.
    #[arg(long, default_value_t = 20)]
    pub max_redirects: usize,
    /// Exit with code 11 for HTTP status 400 or higher.
    #[arg(long)]
    pub fail_on_http_error: bool,
    /// Requires raw format. Streams bytes without navigating or executing page scripts.
    #[arg(long)]
    pub stream: bool,
    /// Write response metadata to a separate file.
    #[arg(long)]
    pub response_info: Option<PathBuf>,
    /// Install a bundled plugin name or JavaScript file; repeat in installer order.
    #[arg(long)]
    pub plugin: Vec<String>,
    /// NAME=JSON; file plugin names are their file stems.
    #[arg(long)]
    pub plugin_options: Vec<String>,
    /// Reject network requests in this page.
    #[arg(long)]
    pub offline: bool,
    /// Delay before transport submission; not bandwidth throttling.
    #[arg(long, value_parser = duration)]
    pub latency: Option<Duration>,
    /// Bypass the page response cache.
    #[arg(long)]
    pub bypass_cache: bool,
    /// Connection establishment timeout.
    #[arg(long, default_value = "10s", value_parser = duration)]
    pub connect_timeout: Duration,
    /// Maximum response body size in bytes.
    #[arg(long, default_value_t = 67108864, value_parser = positive)]
    pub max_response_bytes: usize,
    /// Write runtime API diagnostics as JSONL.
    #[arg(long)]
    pub trace: Option<PathBuf>,
    /// Only trace API paths matching this prefix; repeat for more prefixes.
    #[arg(long, requires = "trace")]
    pub trace_include: Vec<String>,
    /// Include JavaScript stacks in diagnostics.
    #[arg(long, requires = "trace")]
    pub trace_stack: bool,
    /// Retained diagnostic ring capacity, up to one million records.
    #[arg(long, default_value_t = 4096, value_parser = positive)]
    pub trace_capacity: usize,
    /// Write page and network events as JSONL.
    #[arg(long)]
    pub events: Option<PathBuf>,
    /// Use real system time or a programmable realm clock.
    #[arg(long, value_enum, default_value = "system")]
    pub clock: Clock,
    /// Unix epoch milliseconds for a logical realm clock.
    #[arg(long)]
    pub epoch_ms: Option<u64>,
    /// Advance logical time after scripts/actions.
    #[arg(long, value_parser = duration)]
    pub advance_time: Option<Duration>,
    /// Maximum tasks per logical-time advance, up to one million.
    #[arg(long, default_value_t = 10000, value_parser = positive)]
    pub max_tasks: usize,
    /// JSON array of ordered input/evaluation/wait actions.
    #[arg(long)]
    pub actions: Option<PathBuf>,
    /// JSON array of declarative request/response rules.
    #[arg(long)]
    pub routes: Option<PathBuf>,
}
pub fn duration(value: &str) -> Result<Duration, String> {
    let (number, scale) = if let Some(s) = value.strip_suffix("ms") {
        (s, 0.001)
    } else if let Some(s) = value.strip_suffix('s') {
        (s, 1.)
    } else if let Some(s) = value.strip_suffix('m') {
        (s, 60.)
    } else {
        return Err("duration must end in ms, s, or m".into());
    };
    let seconds = number.parse::<f64>().map_err(|_| "invalid duration")? * scale;
    if seconds <= 0. {
        return Err("duration must be positive".into());
    }
    let result = Duration::try_from_secs_f64(seconds).map_err(|_| "duration out of range")?;
    if result.is_zero() {
        return Err("duration is too small".into());
    }
    Ok(result)
}
fn positive(value: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .ok()
        .filter(|v| *v > 0)
        .ok_or_else(|| "must be positive".into())
}

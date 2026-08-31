use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

use glob::Pattern;
use robotstxt::DefaultMatcher;
use url::Url;
use web_runtime::{AutomationBrowser, AutomationError, AutomationPage, CancellationToken};

use super::{
    InterruptMonitor, WaitCondition, argument_error, parse_duration, remaining_timeout, wait_fixed,
    write_output,
};
use crate::common::NavigationOptions;

#[derive(Clone, Copy, Debug)]
enum CrawlFormat {
    Markdown,
    Html,
    Json,
}

impl CrawlFormat {
    fn parse(value: &str) -> Result<Self, AutomationError> {
        match value {
            "markdown" => Ok(Self::Markdown),
            "html" => Ok(Self::Html),
            "json" => Ok(Self::Json),
            _ => Err(AutomationError::InvalidInput(format!(
                "crawl format must be markdown, html, or json; got `{value}`"
            ))),
        }
    }

    fn extension(self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Html => "html",
            Self::Json => "json",
        }
    }
}

struct CrawlOptions {
    start: Url,
    output_dir: PathBuf,
    depth: u32,
    workers: usize,
    max_pages: usize,
    format: CrawlFormat,
    allowed_origins: HashSet<String>,
    include: Vec<Pattern>,
    exclude: Vec<Pattern>,
    ignore_robots: bool,
    delay: Duration,
    fail_fast: bool,
    allow_errors: bool,
    overwrite: bool,
    navigation: NavigationOptions,
}

impl CrawlOptions {
    fn parse(arguments: &[String]) -> Result<Option<Self>, AutomationError> {
        let mut parser = pico_args::Arguments::from_vec(
            arguments.iter().map(OsString::from).collect::<Vec<_>>(),
        );
        if parser.contains(["-h", "--help"]) {
            println!("{CRAWL_USAGE}");
            return Ok(None);
        }
        let output_dir = parser
            .opt_value_from_os_str("--output-dir", |value| {
                Ok::<_, pico_args::Error>(PathBuf::from(value))
            })
            .map_err(argument_error)?
            .unwrap_or_else(|| PathBuf::from("brimp-crawl"));
        let depth = parser
            .opt_value_from_str("--depth")
            .map_err(argument_error)?
            .unwrap_or(2);
        let workers = parser
            .opt_value_from_str("--workers")
            .map_err(argument_error)?
            .unwrap_or(2);
        let max_pages = parser
            .opt_value_from_str("--max-pages")
            .map_err(argument_error)?
            .unwrap_or(1_000);
        if workers == 0 || max_pages == 0 {
            return Err(AutomationError::InvalidInput(
                "--workers and --max-pages must be positive".into(),
            ));
        }
        let format = parser
            .opt_value_from_str::<_, String>("--format")
            .map_err(argument_error)?
            .map(|value| CrawlFormat::parse(&value))
            .transpose()?
            .unwrap_or(CrawlFormat::Markdown);
        let allowed_origins = parser
            .values_from_str::<_, String>("--allow-origin")
            .map_err(argument_error)?
            .into_iter()
            .map(|value| {
                Url::parse(&value).map(|url| origin(&url)).map_err(|error| {
                    AutomationError::InvalidInput(format!(
                        "invalid --allow-origin `{value}`: {error}"
                    ))
                })
            })
            .collect::<Result<HashSet<_>, _>>()?;
        let include = patterns(&mut parser, "--include")?;
        let exclude = patterns(&mut parser, "--exclude")?;
        let ignore_robots = parser.contains("--ignore-robots");
        let delay = parser
            .opt_value_from_str::<_, String>("--delay")
            .map_err(argument_error)?
            .map(|value| parse_duration(&value))
            .transpose()?
            .unwrap_or(Duration::ZERO);
        let fail_fast = parser.contains("--fail-fast");
        let allow_errors = parser.contains("--allow-errors");
        if fail_fast && allow_errors {
            return Err(AutomationError::InvalidInput(
                "--fail-fast and --allow-errors are mutually exclusive".into(),
            ));
        }
        let overwrite = parser.contains("--overwrite");
        let navigation = NavigationOptions::parse(&mut parser)?;
        let start = parser
            .free_from_str::<String>()
            .map_err(argument_error)
            .and_then(|value| {
                Url::parse(&value).map_err(|error| {
                    AutomationError::InvalidInput(format!("invalid crawl URL: {error}"))
                })
            })?;
        if !matches!(start.scheme(), "http" | "https") {
            return Err(AutomationError::InvalidInput(
                "crawl URL must use http or https".into(),
            ));
        }
        let remaining = parser.finish();
        if !remaining.is_empty() {
            return Err(AutomationError::InvalidInput(format!(
                "unknown crawl argument `{}`",
                remaining[0].to_string_lossy()
            )));
        }
        Ok(Some(Self {
            start,
            output_dir,
            depth,
            workers,
            max_pages,
            format,
            allowed_origins,
            include,
            exclude,
            ignore_robots,
            delay,
            fail_fast,
            allow_errors,
            overwrite,
            navigation,
        }))
    }

    fn path_allowed(&self, url: &Url) -> bool {
        let path = url.path();
        (self.include.is_empty() || self.include.iter().any(|pattern| pattern.matches(path)))
            && !self.exclude.iter().any(|pattern| pattern.matches(path))
    }
}

fn patterns(
    parser: &mut pico_args::Arguments,
    name: &'static str,
) -> Result<Vec<Pattern>, AutomationError> {
    parser
        .values_from_str::<_, String>(name)
        .map_err(argument_error)?
        .into_iter()
        .map(|value| {
            Pattern::new(&value).map_err(|error| {
                AutomationError::InvalidInput(format!("invalid {name} pattern: {error}"))
            })
        })
        .collect()
}

#[derive(Clone)]
struct Task {
    url: Url,
    depth: u32,
    start: bool,
}

struct PageResult {
    task: Task,
    final_url: Option<Url>,
    status: Option<u16>,
    bytes: Option<Vec<u8>>,
    links: Vec<Url>,
    error: Option<String>,
    skipped: Option<String>,
}

pub(super) fn run(arguments: &[String]) -> Result<(), AutomationError> {
    let Some(options) = CrawlOptions::parse(arguments)? else {
        return Ok(());
    };
    prepare_output_dir(&options)?;
    let manifest_path = options.output_dir.join("manifest.jsonl");
    let mut manifest = BufWriter::new(File::create(&manifest_path).map_err(io_error)?);
    let browser = Arc::new(AutomationBrowser::with_persona_and_network_config(
        options.navigation.persona.clone(),
        options.navigation.network.clone(),
    )?);
    let context = browser.default_context();
    for (name, value) in &options.navigation.cookies {
        context.set_cookie(options.start.as_str(), name, value)?;
    }
    let interrupt = InterruptMonitor::new();
    let started = Instant::now();
    let pacing = Arc::new(Mutex::new(HashMap::<String, Instant>::new()));
    let mut robots = HashMap::<String, String>::new();
    let mut allowed_origins = options.allowed_origins.clone();
    allowed_origins.insert(origin(&options.start));
    let mut seen = HashSet::from([canonical(options.start.clone())]);
    let mut frontier = vec![Task {
        url: canonical(options.start.clone()),
        depth: 0,
        start: true,
    }];
    let mut failures = 0_usize;
    let mut written_paths = HashSet::new();

    while !frontier.is_empty() && !interrupt.token().is_cancelled() {
        frontier.sort_by(|left, right| left.url.as_str().cmp(right.url.as_str()));
        let mut runnable = Vec::new();
        let mut results = Vec::new();
        for task in std::mem::take(&mut frontier) {
            if !options.ignore_robots {
                let site = origin(&task.url);
                if !robots.contains_key(&site) {
                    match fetch_robots(
                        &browser,
                        &options,
                        &task.url,
                        &pacing,
                        started,
                        interrupt.token(),
                    ) {
                        Ok(body) => {
                            robots.insert(site.clone(), body);
                        }
                        Err(error) => {
                            results.push(failed(task, error));
                            continue;
                        }
                    }
                }
                let mut matcher = DefaultMatcher::default();
                if !matcher.one_agent_allowed_by_robots(
                    robots.get(&site).expect("robots policy was cached"),
                    "brimp",
                    task.url.as_str(),
                ) {
                    results.push(PageResult {
                        task,
                        final_url: None,
                        status: None,
                        bytes: None,
                        links: Vec::new(),
                        error: None,
                        skipped: Some("robots".into()),
                    });
                    continue;
                }
            }
            runnable.push(task);
        }
        results.extend(run_tasks(
            Arc::clone(&browser),
            &options,
            runnable,
            Arc::clone(&pacing),
            interrupt.token(),
            &allowed_origins,
            started,
        ));
        results.sort_by(|left, right| left.task.url.as_str().cmp(right.task.url.as_str()));

        let mut next = Vec::new();
        for mut result in results {
            if result.task.start
                && let Some(final_url) = &result.final_url
            {
                allowed_origins.remove(&origin(&options.start));
                allowed_origins.insert(origin(final_url));
            }
            let output = if let Some(bytes) = result.bytes.take() {
                let relative = output_path(&result.task.url, options.format, &mut written_paths);
                let path = options.output_dir.join(&relative);
                let write = path
                    .parent()
                    .map_or(Ok(()), |parent| {
                        fs::create_dir_all(parent).map_err(io_error)
                    })
                    .and_then(|()| write_output(Some(&path), &bytes, options.overwrite));
                match write {
                    Ok(()) => Some(relative.to_string_lossy().replace('\\', "/")),
                    Err(error) => {
                        result.error = Some(error.to_string());
                        None
                    }
                }
            } else {
                None
            };
            let ok = result.error.is_none();
            if !ok {
                failures += 1;
            }
            let record = serde_json::json!({
                "url": result.task.url,
                "finalUrl": result.final_url,
                "depth": result.task.depth,
                "status": result.status,
                "output": output,
                "ok": ok,
                "error": result.error,
                "skipped": result.skipped,
            });
            serde_json::to_writer(&mut manifest, &record)
                .map_err(|error| AutomationError::Internal(error.to_string()))?;
            manifest.write_all(b"\n").map_err(io_error)?;

            if ok && result.task.depth < options.depth {
                result
                    .links
                    .sort_by(|left, right| left.as_str().cmp(right.as_str()));
                for link in result.links {
                    let link = canonical(link);
                    if seen.len() >= options.max_pages {
                        break;
                    }
                    if !matches!(link.scheme(), "http" | "https")
                        || !allowed_origins.contains(&origin(&link))
                        || !options.path_allowed(&link)
                        || !seen.insert(link.clone())
                    {
                        continue;
                    }
                    next.push(Task {
                        url: link,
                        depth: result.task.depth + 1,
                        start: false,
                    });
                }
            }
        }
        manifest.flush().map_err(io_error)?;
        let stopped = if interrupt.token().is_cancelled() {
            Some("cancelled")
        } else if failures > 0 && options.fail_fast {
            Some("fail-fast")
        } else {
            None
        };
        if let Some(reason) = stopped {
            for task in next {
                let record = serde_json::json!({
                    "url": task.url,
                    "finalUrl": null,
                    "depth": task.depth,
                    "status": null,
                    "output": null,
                    "ok": true,
                    "error": null,
                    "skipped": reason,
                });
                serde_json::to_writer(&mut manifest, &record)
                    .map_err(|error| AutomationError::Internal(error.to_string()))?;
                manifest.write_all(b"\n").map_err(io_error)?;
            }
            manifest.flush().map_err(io_error)?;
            break;
        }
        frontier = next;
    }
    browser.close();
    if interrupt.token().is_cancelled() {
        return Err(AutomationError::Cancellation);
    }
    if started.elapsed() >= options.navigation.timeout {
        return Err(AutomationError::Timeout(options.navigation.timeout));
    }
    if failures > 0 && !options.allow_errors {
        return Err(AutomationError::Internal(format!(
            "crawl completed with {failures} failed page(s); see {}",
            manifest_path.display()
        )));
    }
    Ok(())
}

fn run_tasks(
    browser: Arc<AutomationBrowser>,
    options: &CrawlOptions,
    tasks: Vec<Task>,
    pacing: Arc<Mutex<HashMap<String, Instant>>>,
    cancellation: CancellationToken,
    allowed_origins: &HashSet<String>,
    started: Instant,
) -> Vec<PageResult> {
    if tasks.is_empty() {
        return Vec::new();
    }
    let tasks = Arc::new(tasks);
    let next = Arc::new(AtomicUsize::new(0));
    let worker_count = options.workers.min(tasks.len());
    let (sender, receiver) = mpsc::sync_channel(worker_count);
    let expected = tasks.len();
    let mut results = Vec::with_capacity(expected);
    std::thread::scope(|scope| {
        for _ in 0..worker_count {
            let browser = Arc::clone(&browser);
            let tasks = Arc::clone(&tasks);
            let next = Arc::clone(&next);
            let sender = sender.clone();
            let pacing = Arc::clone(&pacing);
            let cancellation = cancellation.clone();
            let allowed_origins = allowed_origins.clone();
            scope.spawn(move || {
                let page = browser.new_page(options.navigation.page.clone());
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(task) = tasks.get(index).cloned() else {
                        break;
                    };
                    let result = match &page {
                        Ok(page) => process_page(
                            page,
                            options,
                            task,
                            &pacing,
                            cancellation.clone(),
                            &allowed_origins,
                            started,
                        ),
                        Err(error) => PageResult {
                            task,
                            final_url: None,
                            status: None,
                            bytes: None,
                            links: Vec::new(),
                            error: Some(error.to_string()),
                            skipped: None,
                        },
                    };
                    let _ = sender.send(result);
                }
                if let Ok(page) = page {
                    page.close();
                }
            });
        }
        drop(sender);
        for _ in 0..expected {
            if let Ok(result) = receiver.recv() {
                results.push(result);
            }
        }
    });
    results
}

fn process_page(
    page: &AutomationPage,
    options: &CrawlOptions,
    task: Task,
    pacing: &Mutex<HashMap<String, Instant>>,
    cancellation: CancellationToken,
    allowed_origins: &HashSet<String>,
    started: Instant,
) -> PageResult {
    pace(pacing, &task.url, options.delay, &cancellation);
    let page_timeout = match remaining_timeout(started, options.navigation.timeout) {
        Ok(timeout) => timeout,
        Err(error) => return failed(task, error),
    };
    let response =
        match page.navigate_cancellable(task.url.as_str(), page_timeout, cancellation.clone()) {
            Ok(response) => response,
            Err(error) => return failed(task, error),
        };
    let final_url = Url::parse(&response.url).ok();
    if !task.start
        && final_url
            .as_ref()
            .is_some_and(|url| !allowed_origins.contains(&origin(url)))
    {
        return PageResult {
            task,
            final_url,
            status: Some(response.status_code),
            bytes: None,
            links: Vec::new(),
            error: None,
            skipped: Some("redirected outside allowed origins".into()),
        };
    }
    let wait_result = match options.navigation.wait {
        WaitCondition::DomContentLoaded | WaitCondition::Load => Ok(()),
        WaitCondition::NetworkIdle => remaining_timeout(started, options.navigation.timeout)
            .and_then(|timeout| {
                page.wait_for_network_idle(
                    options.navigation.network_idle,
                    timeout,
                    cancellation.clone(),
                )
            }),
        WaitCondition::Fixed(duration) => wait_fixed(
            duration,
            started,
            options.navigation.timeout,
            cancellation.clone(),
        ),
    };
    if let Err(error) = wait_result {
        return failed_with_response(task, final_url, response.status_code, error);
    }
    if let Some(selector) = &options.navigation.wait_selector {
        let result = remaining_timeout(started, options.navigation.timeout).and_then(|timeout| {
            page.wait_for_selector(selector.clone(), timeout, cancellation.clone())
        });
        if let Err(error) = result {
            return failed_with_response(task, final_url, response.status_code, error);
        }
    }
    for source in &options.navigation.scripts {
        if let Err(error) = remaining_timeout(started, options.navigation.timeout) {
            return failed_with_response(task, final_url, response.status_code, error);
        }
        let encoded = match serde_json::to_string(source) {
            Ok(encoded) => encoded,
            Err(error) => {
                return failed_with_response(
                    task,
                    final_url,
                    response.status_code,
                    AutomationError::Internal(error.to_string()),
                );
            }
        };
        if let Err(error) = page.evaluate(format!("(0, eval)({encoded}); null")) {
            return failed_with_response(task, final_url, response.status_code, error);
        }
    }
    if let Err(error) = remaining_timeout(started, options.navigation.timeout) {
        return failed_with_response(task, final_url, response.status_code, error);
    }
    let links = page
        .evaluate("Array.from(document.querySelectorAll('a[href]'), link => link.href)")
        .ok()
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().and_then(|value| Url::parse(value).ok()))
        .collect();
    let bytes = match options.format {
        CrawlFormat::Html => page
            .evaluate("document.documentElement.outerHTML")
            .and_then(|value| {
                value
                    .as_str()
                    .map(|value| value.as_bytes().to_vec())
                    .ok_or_else(|| {
                        AutomationError::Internal("rendered HTML was not a string".into())
                    })
            }),
        CrawlFormat::Markdown => page
            .extract(options.navigation.extraction.clone())
            .and_then(|document| {
                document
                    .content_markdown
                    .map(String::into_bytes)
                    .ok_or_else(|| {
                        AutomationError::Extraction("Defuddle returned no Markdown".into())
                    })
            }),
        CrawlFormat::Json => page
            .extract(options.navigation.extraction.clone())
            .and_then(|document| {
                serde_json::to_vec(&document)
                    .map_err(|error| AutomationError::Internal(error.to_string()))
            }),
    };
    match bytes {
        Ok(bytes) if remaining_timeout(started, options.navigation.timeout).is_ok() => PageResult {
            task,
            final_url,
            status: Some(response.status_code),
            bytes: Some(bytes),
            links,
            error: None,
            skipped: None,
        },
        Ok(_) => failed_with_response(
            task,
            final_url,
            response.status_code,
            AutomationError::Timeout(options.navigation.timeout),
        ),
        Err(error) => failed_with_response(task, final_url, response.status_code, error),
    }
}

fn failed(task: Task, error: AutomationError) -> PageResult {
    PageResult {
        task,
        final_url: None,
        status: None,
        bytes: None,
        links: Vec::new(),
        error: Some(error.to_string()),
        skipped: None,
    }
}

fn failed_with_response(
    task: Task,
    final_url: Option<Url>,
    status: u16,
    error: AutomationError,
) -> PageResult {
    PageResult {
        task,
        final_url,
        status: Some(status),
        bytes: None,
        links: Vec::new(),
        error: Some(error.to_string()),
        skipped: None,
    }
}

fn fetch_robots(
    browser: &AutomationBrowser,
    options: &CrawlOptions,
    url: &Url,
    pacing: &Mutex<HashMap<String, Instant>>,
    started: Instant,
    cancellation: CancellationToken,
) -> Result<String, AutomationError> {
    let mut robots = url.clone();
    robots.set_path("/robots.txt");
    robots.set_query(None);
    robots.set_fragment(None);
    pace(pacing, &robots, options.delay, &cancellation);
    let page = browser.new_page(options.navigation.page.clone())?;
    let response = page.navigate_cancellable(
        robots.as_str(),
        remaining_timeout(started, options.navigation.timeout)?,
        cancellation,
    );
    page.close();
    match response {
        Ok(response) if (200..300).contains(&response.status_code) => {
            Ok(String::from_utf8_lossy(&response.content).into_owned())
        }
        Ok(_) => Ok(String::new()),
        Err(error) => Err(error),
    }
}

fn pace(
    starts: &Mutex<HashMap<String, Instant>>,
    url: &Url,
    delay: Duration,
    cancellation: &CancellationToken,
) {
    if delay.is_zero() {
        return;
    }
    let site = origin(url);
    loop {
        let remaining = {
            let mut starts = starts.lock().expect("crawl pacing lock poisoned");
            let remaining = starts
                .get(&site)
                .map(|previous| delay.saturating_sub(previous.elapsed()))
                .unwrap_or(Duration::ZERO);
            if remaining.is_zero() {
                starts.insert(site.clone(), Instant::now());
                return;
            } else {
                remaining
            }
        };
        let began = Instant::now();
        while began.elapsed() < remaining {
            if cancellation.is_cancelled() {
                return;
            }
            std::thread::sleep(
                remaining
                    .saturating_sub(began.elapsed())
                    .min(Duration::from_millis(5)),
            );
        }
    }
}

fn prepare_output_dir(options: &CrawlOptions) -> Result<(), AutomationError> {
    if options.output_dir.exists() {
        if !options.output_dir.is_dir() {
            return Err(AutomationError::InvalidInput(format!(
                "crawl output `{}` is not a directory",
                options.output_dir.display()
            )));
        }
        let nonempty = fs::read_dir(&options.output_dir)
            .map_err(io_error)?
            .next()
            .is_some();
        if nonempty && !options.overwrite {
            return Err(AutomationError::InvalidInput(format!(
                "crawl output `{}` is not empty; pass --overwrite",
                options.output_dir.display()
            )));
        }
    } else {
        fs::create_dir_all(&options.output_dir).map_err(io_error)?;
    }
    Ok(())
}

fn output_path(url: &Url, format: CrawlFormat, used: &mut HashSet<PathBuf>) -> PathBuf {
    let trailing_slash = url.path().ends_with('/');
    let mut path = PathBuf::new();
    for segment in url.path_segments().into_iter().flatten() {
        if !segment.is_empty() {
            path.push(safe_segment(segment));
        }
    }
    if path.as_os_str().is_empty() || trailing_slash {
        path.push("index");
    }
    if url.query().is_some() {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("page");
        path.set_file_name(format!("{name}-q-{:016x}", stable_hash(url.as_str())));
    }
    path.set_extension(format.extension());
    if !used.insert(path.clone()) {
        let name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("page");
        path.set_file_name(format!(
            "{name}-{:016x}.{}",
            stable_hash(url.as_str()),
            format.extension()
        ));
        used.insert(path.clone());
    }
    path
}

fn safe_segment(segment: &str) -> String {
    let mut output = String::new();
    for byte in segment.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.') {
            output.push(char::from(byte));
        } else {
            output.push_str(&format!("_{byte:02X}"));
        }
    }
    if output == "." || output == ".." || output.is_empty() {
        format!("segment-{:016x}", stable_hash(segment))
    } else {
        output
    }
}

fn stable_hash(value: &str) -> u64 {
    value.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

fn canonical(mut url: Url) -> Url {
    url.set_fragment(None);
    url
}

fn origin(url: &Url) -> String {
    url.origin().ascii_serialization()
}

fn io_error(error: std::io::Error) -> AutomationError {
    AutomationError::Internal(error.to_string())
}

const CRAWL_USAGE: &str = "usage: brimp crawl URL [OPTIONS]\n\nBOUNDS:\n  --output-dir PATH       default: ./brimp-crawl\n  --depth N               default: 2\n  --workers N             default: 2\n  --max-pages N           default: 1000\n  --format markdown|html|json\n\nSCOPE:\n  --include GLOB          repeatable\n  --exclude GLOB          repeatable\n  --allow-origin URL      repeatable\n  --ignore-robots\n  --delay DURATION\n\nWAITING AND ACTIONS:\n  --wait domcontentloaded|load|networkidle|SECONDS\n  --wait-selector SELECTOR\n  --network-idle DURATION\n  --script PATH           repeatable\n\nFAILURES:\n  --fail-fast\n  --allow-errors\n  --overwrite\n\nCrawl also accepts get's extraction, network, identity, timeout, and page subsystem options.";

pub(super) fn usage() -> &'static str {
    CRAWL_USAGE
}

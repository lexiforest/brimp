use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use web_runtime::{AutomationBrowser, AutomationError, CancellationToken, PageOptions};

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
    #[cfg(unix)]
    unsafe {
        signal(2, interrupt);
    }
    match run(std::env::args().skip(1).collect()) {
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
        "eval" => eval_command(&arguments[1..]),
        "screenshot" => screenshot_command(&arguments[1..]),
        "--help" | "-h" | "help" => {
            println!("{}", usage());
            Ok(())
        }
        command => Err(AutomationError::InvalidInput(format!(
            "unknown command `{command}`\n{}",
            usage()
        ))),
    }
}

fn doctor() -> Result<(), AutomationError> {
    network::CurlResourceLoader::check_profile(&network::CurlConfig::default())
        .map_err(|error| AutomationError::Transport(error.to_string()))?;
    let browser = AutomationBrowser::new()?;
    let page = browser.new_page(PageOptions::default())?;
    page.close();
    browser.close();
    println!(
        "{}",
        serde_json::json!({"javascriptCore": "ok", "libcurlImpersonate": "ok", "profile": "chrome136"})
    );
    Ok(())
}

fn eval_command(arguments: &[String]) -> Result<(), AutomationError> {
    let url = required_positional(arguments)?;
    let expression = option(arguments, "--js")
        .ok_or_else(|| AutomationError::InvalidInput("eval requires --js EXPRESSION".into()))?;
    let timeout = timeout(arguments)?;
    let (browser, page) = launch()?;
    navigate(&page, url, timeout)?;
    let value = page.evaluate(expression)?;
    println!("{value}");
    page.close();
    browser.close();
    Ok(())
}

fn screenshot_command(arguments: &[String]) -> Result<(), AutomationError> {
    let url = required_positional(arguments)?;
    let output = option(arguments, "--output")
        .ok_or_else(|| AutomationError::InvalidInput("screenshot requires --output PATH".into()))?;
    let path = PathBuf::from(output);
    if path.exists() && !flag(arguments, "--overwrite") {
        return Err(AutomationError::InvalidInput(format!(
            "output `{}` exists; pass --overwrite to replace it",
            path.display()
        )));
    }
    let (browser, page) = launch()?;
    navigate(&page, url, timeout(arguments)?)?;
    let png = page.screenshot(flag(arguments, "--full-page"))?;
    std::fs::write(&path, png).map_err(|error| AutomationError::Screenshot(error.to_string()))?;
    page.close();
    browser.close();
    Ok(())
}

fn launch() -> Result<(AutomationBrowser, web_runtime::AutomationPage), AutomationError> {
    let browser = AutomationBrowser::new()?;
    let page = browser.new_page(PageOptions::default())?;
    Ok((browser, page))
}
fn navigate(
    page: &web_runtime::AutomationPage,
    url: &str,
    timeout: Duration,
) -> Result<(), AutomationError> {
    INTERRUPTED.store(false, Ordering::Release);
    let cancellation = CancellationToken::new();
    let monitor_token = cancellation.clone();
    let finished = std::sync::Arc::new(AtomicBool::new(false));
    let monitor_finished = std::sync::Arc::clone(&finished);
    let monitor = std::thread::spawn(move || {
        while !monitor_finished.load(Ordering::Acquire) {
            if INTERRUPTED.load(Ordering::Acquire) {
                monitor_token.cancel();
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    });
    let result = page.navigate_cancellable(url, timeout, cancellation);
    finished.store(true, Ordering::Release);
    let _ = monitor.join();
    result.map(|_| ())
}
fn required_positional(arguments: &[String]) -> Result<&str, AutomationError> {
    arguments
        .first()
        .filter(|value| !value.starts_with('-'))
        .map(String::as_str)
        .ok_or_else(|| AutomationError::InvalidInput("a URL is required".into()))
}
fn option<'a>(arguments: &'a [String], name: &str) -> Option<&'a str> {
    arguments
        .windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
}
fn flag(arguments: &[String], name: &str) -> bool {
    arguments.iter().any(|value| value == name)
}
fn timeout(arguments: &[String]) -> Result<Duration, AutomationError> {
    let millis = option(arguments, "--timeout-ms")
        .unwrap_or("30000")
        .parse::<u64>()
        .map_err(|_| AutomationError::InvalidInput("--timeout-ms must be an integer".into()))?;
    if millis == 0 {
        return Err(AutomationError::InvalidInput(
            "--timeout-ms must be positive".into(),
        ));
    }
    Ok(Duration::from_millis(millis))
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
        AutomationError::Screenshot(_) | AutomationError::Internal(_) => 18,
    }
}
fn usage() -> String {
    "usage: brimp doctor | brimp eval URL --js EXPRESSION [--timeout-ms N] | brimp screenshot URL --output PATH [--full-page] [--overwrite] [--timeout-ms N]".into()
}

use std::path::PathBuf;
use std::time::Duration;

use brimp_worker_api::{AutomationError, ExtractionOptions, WorkerConfig};

use super::{WaitCondition, argument_error, parse_duration, parse_header};

pub(crate) struct NavigationOptions {
    pub(crate) timeout: Duration,
    pub(crate) wait: WaitCondition,
    pub(crate) wait_selector: Option<String>,
    pub(crate) network_idle: Duration,
    pub(crate) scripts: Vec<String>,
    pub(crate) extraction: ExtractionOptions,
    pub(crate) config: WorkerConfig,
    pub(crate) worker_path: String,
    pub(crate) cookies: Vec<(String, String)>,
}

impl NavigationOptions {
    pub(crate) fn parse(parser: &mut pico_args::Arguments) -> Result<Self, AutomationError> {
        let timeout = parser
            .opt_value_from_str::<_, String>("--timeout")
            .map_err(argument_error)?
            .map(|value| parse_duration(&value))
            .transpose()?
            .unwrap_or(Duration::from_secs(30));
        let wait = parser
            .opt_value_from_str::<_, String>("--wait")
            .map_err(argument_error)?
            .map(|value| WaitCondition::parse(&value))
            .transpose()?
            .unwrap_or(WaitCondition::Load);
        let wait_selector = parser
            .opt_value_from_str("--wait-selector")
            .map_err(argument_error)?;
        let network_idle = parser
            .opt_value_from_str::<_, String>("--network-idle")
            .map_err(argument_error)?
            .map(|value| parse_duration(&value))
            .transpose()?
            .unwrap_or(Duration::from_millis(500));
        let scripts = parser
            .values_from_os_str("--script", |value| {
                Ok::<_, pico_args::Error>(PathBuf::from(value))
            })
            .map_err(argument_error)?
            .into_iter()
            .map(|path| {
                std::fs::read_to_string(&path).map_err(|error| {
                    AutomationError::InvalidInput(format!(
                        "cannot read script `{}`: {error}",
                        path.display()
                    ))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let extraction = ExtractionOptions {
            content_selector: parser
                .opt_value_from_str("--content")
                .map_err(argument_error)?,
            remove_images: parser.contains("--remove-images"),
            language: parser
                .opt_value_from_str("--language")
                .map_err(argument_error)?,
            debug: parser.contains("--extract-debug"),
        };
        let persona_path = parser
            .opt_value_from_os_str("--persona", |value| {
                Ok::<_, pico_args::Error>(PathBuf::from(value))
            })
            .map_err(argument_error)?;
        let persona = persona_path.map_or_else(
            || Ok(None),
            |path| {
                persona::PersonaConfig::from_json_file(path)
                    .map(Some)
                    .map_err(|error| AutomationError::InvalidInput(error.to_string()))
            },
        )?;
        let proxy = parser
            .opt_value_from_str::<_, String>("--proxy")
            .map_err(argument_error)?;
        let ca_bundle = parser
            .opt_value_from_os_str("--ca-bundle", |value| {
                Ok::<_, pico_args::Error>(PathBuf::from(value))
            })
            .map_err(argument_error)?;
        let request_headers = parser
            .values_from_str::<_, String>("--header")
            .map_err(argument_error)?
            .into_iter()
            .map(|value| parse_header(&value))
            .collect::<Result<Vec<_>, _>>()?;
        let cookies = parser
            .values_from_str::<_, String>("--cookie")
            .map_err(argument_error)?
            .into_iter()
            .map(|cookie| {
                let (name, value) = cookie.split_once('=').ok_or_else(|| {
                    AutomationError::InvalidInput("--cookie must use the form NAME=VALUE".into())
                })?;
                if name.is_empty() {
                    return Err(AutomationError::InvalidInput(
                        "--cookie name must not be empty".into(),
                    ));
                }
                Ok((name.to_owned(), value.to_owned()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let worker_path = worker_path(parser)?;
        let config = WorkerConfig {
            navigation_timeout_ms: Some(timeout.as_millis() as u64),
            persona,
            proxy,
            ca_bundle,
            headers: request_headers,
            worker: parser.contains("--enable-worker"),
            streaming_networking: parser.contains("--enable-streaming-networking"),
            canvas: parser.contains("--enable-canvas"),
            storage_path: parser
                .opt_value_from_str::<_, PathBuf>("--storage-path")
                .map_err(argument_error)?,
            storage_quota: parser
                .opt_value_from_str("--storage-quota-bytes")
                .map_err(argument_error)?,
        };
        if config.storage_quota == Some(0) {
            return Err(AutomationError::InvalidInput(
                "--storage-quota-bytes must be positive".into(),
            ));
        }
        if config.storage_quota.is_some() && config.storage_path.is_none() {
            return Err(AutomationError::InvalidInput(
                "--storage-quota-bytes requires --storage-path".into(),
            ));
        }
        Ok(Self {
            timeout,
            wait,
            wait_selector,
            network_idle,
            scripts,
            extraction,
            config,
            worker_path,
            cookies,
        })
    }
}

pub(crate) fn worker_path(parser: &mut pico_args::Arguments) -> Result<String, AutomationError> {
    parser
        .opt_value_from_str::<_, String>("--worker-path")
        .map_err(argument_error)?
        .or_else(|| std::env::var("BRIMP_WORKER_PATH").ok())
        .filter(|path| !path.is_empty())
        .ok_or_else(|| {
            AutomationError::InvalidInput("--worker-path or BRIMP_WORKER_PATH is required".into())
        })
}

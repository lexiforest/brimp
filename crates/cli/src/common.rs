use std::path::PathBuf;
use std::time::Duration;

use web_runtime::{AutomationError, ExtractionOptions, PageOptions, PersistentStorageOptions};

use super::{WaitCondition, argument_error, parse_duration, parse_header};

pub(crate) struct PageFeatures {
    pub(crate) worker: bool,
    pub(crate) streaming_networking: bool,
    pub(crate) canvas: bool,
    pub(crate) webgl: bool,
    pub(crate) webgpu: bool,
    pub(crate) webaudio: bool,
    pub(crate) webaudio_output: bool,
    pub(crate) storage_path: Option<PathBuf>,
    pub(crate) storage_quota: Option<u64>,
}

impl PageFeatures {
    pub(crate) fn parse(parser: &mut pico_args::Arguments) -> Result<Self, AutomationError> {
        Ok(Self {
            worker: parser.contains("--enable-worker"),
            streaming_networking: parser.contains("--enable-streaming-networking"),
            canvas: parser.contains("--enable-canvas"),
            webgl: parser.contains("--enable-webgl"),
            webgpu: parser.contains("--enable-webgpu"),
            webaudio: parser.contains("--enable-webaudio"),
            webaudio_output: parser.contains("--enable-webaudio-output"),
            storage_path: parser
                .opt_value_from_os_str("--storage-path", |value| {
                    Ok::<_, pico_args::Error>(PathBuf::from(value))
                })
                .map_err(argument_error)?,
            storage_quota: parser
                .opt_value_from_str::<_, u64>("--storage-quota-bytes")
                .map_err(argument_error)?,
        })
    }

    pub(crate) fn build(
        self,
        request_headers: Vec<(String, String)>,
    ) -> Result<PageOptions, AutomationError> {
        if self.storage_quota == Some(0) {
            return Err(AutomationError::InvalidInput(
                "--storage-quota-bytes must be positive".into(),
            ));
        }
        if self.storage_quota.is_some() && self.storage_path.is_none() {
            return Err(AutomationError::InvalidInput(
                "--storage-quota-bytes requires --storage-path".into(),
            ));
        }
        let mut page = PageOptions::builder()
            .request_headers(request_headers)
            .worker_system(self.worker)
            .streaming_networking(self.streaming_networking)
            .canvas(self.canvas)
            .webgl(self.webgl)
            .webgpu(self.webgpu)
            .webaudio(self.webaudio)
            .webaudio_output(self.webaudio_output);
        if let Some(path) = self.storage_path {
            page = page.persistent_storage(
                PersistentStorageOptions::new(path)
                    .quota_bytes(self.storage_quota.unwrap_or(1_073_741_824)),
            );
        }
        Ok(page.build())
    }
}

pub(crate) struct NavigationOptions {
    pub(crate) timeout: Duration,
    pub(crate) wait: WaitCondition,
    pub(crate) wait_selector: Option<String>,
    pub(crate) network_idle: Duration,
    pub(crate) scripts: Vec<String>,
    pub(crate) extraction: ExtractionOptions,
    pub(crate) persona: persona::PersonaConfig,
    pub(crate) network: network::CurlConfig,
    pub(crate) page: PageOptions,
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
            || Ok(persona::PersonaConfig::default()),
            |path| {
                persona::PersonaConfig::from_json_file(path)
                    .map_err(|error| AutomationError::InvalidInput(error.to_string()))
            },
        )?;
        let proxy = parser
            .opt_value_from_str::<_, String>("--proxy")
            .map_err(argument_error)?
            .map(network::Proxy::parse)
            .transpose()
            .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
        let ca_bundle = parser
            .opt_value_from_os_str("--ca-bundle", |value| {
                Ok::<_, pico_args::Error>(PathBuf::from(value))
            })
            .map_err(argument_error)?;
        let mut request_headers = parser
            .values_from_str::<_, String>("--header")
            .map_err(argument_error)?
            .into_iter()
            .map(|value| parse_header(&value))
            .collect::<Result<Vec<_>, _>>()?;
        let cookies = parser
            .values_from_str::<_, String>("--cookie")
            .map_err(argument_error)?;
        if !cookies.is_empty() {
            let cookie = cookies.join("; ");
            http::HeaderValue::from_str(&cookie).map_err(|error| {
                AutomationError::InvalidInput(format!("invalid cookie header: {error}"))
            })?;
            request_headers.push(("cookie".into(), cookie));
        }
        let page = PageFeatures::parse(parser)?.build(request_headers)?;
        Ok(Self {
            timeout,
            wait,
            wait_selector,
            network_idle,
            scripts,
            extraction,
            persona,
            network: network::CurlConfig {
                proxy,
                ca_bundle,
                ..network::CurlConfig::default()
            },
            page,
        })
    }
}

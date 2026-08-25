use std::time::Duration;
use thiserror::Error;
use url::Url;

#[derive(Debug, Clone)]
pub struct CurlConfig {
    pub impersonation_profile: String,
    pub default_headers: bool,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub proxy: Option<Proxy>,
    pub queue_capacity: usize,
    pub max_response_bytes: usize,
}
impl Default for CurlConfig {
    fn default() -> Self {
        Self {
            impersonation_profile: "chrome136".into(),
            default_headers: false,
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            proxy: None,
            queue_capacity: 256,
            max_response_bytes: 64 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proxy {
    url: String,
    kind: ProxyKind,
}
impl Proxy {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, ProxyParseError> {
        let value = value.as_ref();
        let parsed = Url::parse(value).map_err(|error| ProxyParseError::InvalidUrl {
            url: value.into(),
            reason: error.to_string(),
        })?;
        if parsed.host_str().is_none() {
            return Err(ProxyParseError::MissingHost(value.into()));
        }
        let kind = match parsed.scheme() {
            "http" => ProxyKind::Http,
            "socks5" => ProxyKind::Socks5,
            "socks5h" => ProxyKind::Socks5h,
            scheme => return Err(ProxyParseError::UnsupportedScheme(scheme.into())),
        };
        Ok(Self {
            url: value.into(),
            kind,
        })
    }
    pub fn url(&self) -> &str {
        &self.url
    }
    pub(crate) fn curl_proxy_type(&self) -> i64 {
        match self.kind {
            ProxyKind::Http => 0,
            ProxyKind::Socks5 => 5,
            ProxyKind::Socks5h => 7,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyKind {
    Http,
    Socks5,
    Socks5h,
}
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProxyParseError {
    #[error("invalid proxy URL `{url}`: {reason}")]
    InvalidUrl { url: String, reason: String },
    #[error("proxy URL `{0}` has no host")]
    MissingHost(String),
    #[error("unsupported proxy scheme `{0}`; use http, socks5, or socks5h")]
    UnsupportedScheme(String),
}

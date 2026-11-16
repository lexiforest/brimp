use std::time::Duration;

use thiserror::Error;
use url::Url;

const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_IMPERSONATION_TARGET: &str = "chrome150";

/// Redirect handling policy for a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectPolicy {
    /// Return the first HTTP response without following redirects.
    None,
    /// Ask libcurl to follow redirects before returning the final response.
    Follow,
}

/// A validated proxy configuration for a [`crate::Client`].
#[derive(Clone, PartialEq, Eq)]
pub struct Proxy {
    url: String,
    kind: ProxyKind,
}

impl std::fmt::Debug for Proxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Proxy")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl Proxy {
    /// Parses an HTTP, HTTPS or SOCKS5 proxy URL.
    ///
    /// Supported schemes are `http`, `https`, `socks5`, and `socks5h`. The `socks5h`
    /// form asks the proxy to resolve destination hostnames.
    pub fn parse(url: impl AsRef<str>) -> Result<Self, ProxyParseError> {
        let url = url.as_ref();
        let parsed = Url::parse(url).map_err(|error| ProxyParseError::InvalidUrl {
            reason: error.to_string(),
        })?;
        if parsed.host_str().is_none() {
            return Err(ProxyParseError::MissingHost);
        }

        let kind = match parsed.scheme() {
            "http" => ProxyKind::Http,
            "https" => ProxyKind::Https,
            "socks5" => ProxyKind::Socks5,
            "socks5h" => ProxyKind::Socks5h,
            scheme => return Err(ProxyParseError::UnsupportedScheme(scheme.to_string())),
        };

        Ok(Self {
            url: url.to_string(),
            kind,
        })
    }

    /// Returns the configured proxy URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns the protocol and hostname-resolution policy for this proxy.
    pub fn kind(&self) -> ProxyKind {
        self.kind
    }

    pub(crate) fn curl_proxy_type(&self) -> i64 {
        match self.kind {
            ProxyKind::Http => 0,
            ProxyKind::Https => 2,
            ProxyKind::Socks5 => 5,
            ProxyKind::Socks5h => 7,
        }
    }
}

/// The protocol and hostname-resolution policy for a [`Proxy`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyKind {
    /// An HTTP forward proxy. HTTPS destinations use HTTP CONNECT.
    Http,
    /// TLS connection to an HTTPS proxy.
    Https,
    /// A SOCKS5 proxy with local hostname resolution.
    Socks5,
    /// A SOCKS5 proxy that resolves destination hostnames itself.
    Socks5h,
}

/// Error returned when parsing a proxy URL.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProxyParseError {
    /// The URL is malformed.
    #[error("invalid proxy URL: {reason}")]
    InvalidUrl {
        /// URL parser error.
        reason: String,
    },
    /// The URL has no proxy hostname.
    #[error("proxy URL has no host")]
    MissingHost,
    /// The URL scheme is not supported by Brimp.
    #[error("unsupported proxy scheme `{0}`; use http, https, socks5, or socks5h")]
    UnsupportedScheme(String),
}

/// Configuration applied to every request sent by a [`crate::Client`].
#[derive(Debug, Clone)]
pub struct Config {
    /// Optional custom CA certificate bundle.
    pub ca_bundle: Option<std::path::PathBuf>,
    /// Prefer HTTP/3 with curl's normal protocol fallback.
    pub prefer_http3: bool,
    /// Maximum queued submissions (at least one).
    pub queue_capacity: usize,
    /// libcurl-impersonate profile selected by the caller's persona.
    pub impersonation_target: String,
    /// Inline fingerprint overrides the named impersonation target.
    pub fingerprint: Option<crate::fingerprint::Fingerprint>,
    /// Maximum time allowed to establish the connection.
    pub connect_timeout: Duration,
    /// Maximum total time allowed for the request.
    pub request_timeout: Duration,
    /// Redirect behavior for requests.
    pub redirect_policy: RedirectPolicy,
    /// Whether libcurl-impersonate should install profile default HTTP headers.
    ///
    /// Disabled by default so callers can provide page-context headers.
    pub default_headers: bool,
    /// Optional HTTP, HTTPS or SOCKS5 proxy used for the transfer.
    pub proxy: Option<Proxy>,
    /// Whether unspecified proxies may be read from the environment.
    pub trust_env: bool,
    /// Explicit bypass list; an empty string disables environment NO_PROXY.
    pub no_proxy: Option<String>,
}

impl Default for Config {
    /// Builds a Chrome-oriented default configuration.
    fn default() -> Self {
        Self {
            ca_bundle: None,
            prefer_http3: false,
            queue_capacity: 256,
            impersonation_target: DEFAULT_IMPERSONATION_TARGET.to_string(),
            fingerprint: None,
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
            redirect_policy: RedirectPolicy::None,
            default_headers: false,
            proxy: None,
            trust_env: true,
            no_proxy: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_accepts_http_and_socks5_urls() {
        let http = Proxy::parse("http://proxy.example:8080").unwrap();
        let socks5 = Proxy::parse("socks5://proxy.example:1080").unwrap();
        let socks5h = Proxy::parse("socks5h://proxy.example:1080").unwrap();

        assert_eq!(http.kind, ProxyKind::Http);
        assert_eq!(socks5.kind, ProxyKind::Socks5);
        assert_eq!(socks5h.kind, ProxyKind::Socks5h);
        let https = Proxy::parse("https://user:secret@proxy.example:8443").unwrap();
        assert_eq!(https.kind(), ProxyKind::Https);
        assert_eq!(https.curl_proxy_type(), 2);
        assert!(!format!("{https:?}").contains("secret"));
        assert!(!format!("{:?}", Proxy::parse("http://user:secret@[")).contains("secret"));
        assert_eq!(http.curl_proxy_type(), 0);
        assert_eq!(socks5.curl_proxy_type(), 5);
        assert_eq!(socks5h.curl_proxy_type(), 7);
    }

    #[test]
    fn proxy_rejects_unsupported_or_hostless_urls() {
        assert!(matches!(
            Proxy::parse("ftp://proxy.example:8443"),
            Err(ProxyParseError::UnsupportedScheme(scheme)) if scheme == "ftp"
        ));
        assert!(matches!(
            Proxy::parse("socks5://"),
            Err(ProxyParseError::MissingHost)
        ));
    }
}

/// Per-request HTTP/proxy authentication, supplied through request extensions.
#[derive(Clone, Default)]
pub struct Credentials(pub Vec<Credential>);
/// One authentication challenge response. Debug intentionally omits secrets.
#[derive(Clone)]
pub struct Credential {
    /// Authentication username.
    pub username: String,
    /// Authentication password.
    pub password: String,
    /// Authenticate to the proxy instead of the origin.
    pub proxy: bool,
}
/// Explicit wire header order, including repeated fields, through request extensions.
#[derive(Clone, Debug)]
pub struct OrderedHeaders(pub Vec<(http::HeaderName, http::HeaderValue)>);

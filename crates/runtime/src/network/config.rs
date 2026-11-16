use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct CurlConfig {
    pub impersonation_profile: String,
    pub fingerprint: Option<brimp_net::fingerprint::Fingerprint>,
    pub default_headers: bool,
    pub prefer_http3: bool,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub ca_bundle: Option<PathBuf>,
    pub proxy_options: super::ProxyOptions,
    pub queue_capacity: usize,
    pub max_response_bytes: usize,
}
impl Default for CurlConfig {
    fn default() -> Self {
        Self {
            impersonation_profile: "chrome136".into(),
            fingerprint: None,
            default_headers: false,
            prefer_http3: true,
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            ca_bundle: None,
            proxy_options: Default::default(),
            queue_capacity: 256,
            max_response_bytes: 64 * 1024 * 1024,
        }
    }
}

pub use brimp_net::{Proxy, ProxyKind, ProxyParseError};

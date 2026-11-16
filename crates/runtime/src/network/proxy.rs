//! Proxy selection is request metadata, independent of page and cookie ownership.
use super::{NetworkError, Proxy};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ProxyOptions {
    pub proxy: Option<String>,
    pub proxies: Option<BTreeMap<String, String>>,
    pub proxy_auth: Option<(String, String)>,
    pub trust_env: Option<bool>,
}

impl std::fmt::Debug for ProxyOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProxyOptions").finish_non_exhaustive()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ProxySelection {
    pub proxy: Option<Proxy>,
    pub no_proxy: String,
}
impl std::fmt::Debug for ProxySelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProxySelection").finish_non_exhaustive()
    }
}
impl ProxySelection {
    pub fn cache_key(&self) -> String {
        // Internal only: never include credential-bearing identities in events or logs.
        serde_json::to_string(&(self.proxy.as_ref().map(Proxy::url), &self.no_proxy)).unwrap()
    }
}

impl ProxyOptions {
    pub fn validate(&self) -> Result<(), NetworkError> {
        if self.proxy.is_some() && self.proxies.is_some() {
            return Err(NetworkError::InvalidRequest(
                "specify proxy or proxies, not both".into(),
            ));
        }
        for value in self
            .proxy
            .iter()
            .chain(self.proxies.iter().flat_map(|map| map.values()))
        {
            if !value.is_empty() {
                Proxy::parse(value).map_err(|e| NetworkError::InvalidProxyUrl(e.to_string()))?;
            }
        }
        Ok(())
    }
    pub fn is_empty(&self) -> bool {
        self.proxy.is_none()
            && self.proxies.is_none()
            && self.proxy_auth.is_none()
            && self.trust_env.is_none()
    }
    pub fn overlay(&mut self, other: Self) {
        if other.proxy.is_some() || other.proxies.is_some() {
            self.proxy = other.proxy;
            self.proxies = other.proxies;
            // Credentials must not follow a route onto a different proxy.
            self.proxy_auth = other.proxy_auth;
        } else if other.proxy_auth.is_some() {
            self.proxy_auth = other.proxy_auth;
        }
        if other.trust_env.is_some() {
            self.trust_env = other.trust_env;
        }
    }
    fn select(&self, url: &url::Url) -> Option<&str> {
        if let Some(proxy) = &self.proxy {
            return Some(proxy);
        }
        let map = self.proxies.as_ref()?;
        let scheme = match url.scheme() {
            "ws" => "http",
            "wss" => "https",
            scheme => scheme,
        };
        let host = url.host_str().unwrap_or("");
        [
            format!("{scheme}://{host}"),
            format!("all://{host}"),
            scheme.into(),
            "all".into(),
        ]
        .iter()
        .find_map(|key| map.get(key).map(String::as_str))
    }
    pub fn resolve(&self, url: &str, defaults: &Self) -> Result<ProxySelection, NetworkError> {
        self.validate()?;
        let url = url::Url::parse(url).map_err(invalid)?;
        let own = self.select(&url);
        let configured = own.or_else(|| defaults.select(&url));
        let trust_env = self.trust_env.or(defaults.trust_env).unwrap_or(true);
        let env_proxy = if configured.is_none() && trust_env {
            let scheme = match url.scheme() {
                "ws" => "http",
                "wss" => "https",
                scheme => scheme,
            };
            environment(&format!("{scheme}_proxy"), scheme != "http")
                .or_else(|| environment("all_proxy", true))
        } else {
            None
        };
        let value = configured.or(env_proxy.as_deref()).unwrap_or("");
        let mut proxy = if value.is_empty() {
            None
        } else {
            Some(Proxy::parse(value).map_err(|e| NetworkError::InvalidProxyUrl(e.to_string()))?)
        };
        let auth = self.proxy_auth.as_ref().or_else(|| {
            if own.is_none() {
                defaults.proxy_auth.as_ref()
            } else {
                None
            }
        });
        if let (Some(selected), Some((username, password))) = (&proxy, auth) {
            let mut value = url::Url::parse(selected.url()).map_err(invalid)?;
            value
                .set_username(username)
                .map_err(|_| invalid("invalid proxy username"))?;
            value
                .set_password(Some(password))
                .map_err(|_| invalid("invalid proxy password"))?;
            proxy = Some(
                Proxy::parse(value.as_str())
                    .map_err(|e| NetworkError::InvalidProxyUrl(e.to_string()))?,
            );
        }
        Ok(ProxySelection {
            proxy,
            no_proxy: if configured.is_none() && trust_env {
                environment("no_proxy", true).unwrap_or_default()
            } else {
                String::new()
            },
        })
    }
}
fn environment(name: &str, uppercase: bool) -> Option<String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| {
            uppercase
                .then(|| std::env::var(name.to_ascii_uppercase()).ok())
                .flatten()
                .filter(|value| !value.is_empty())
        })
}
fn invalid(error: impl std::fmt::Display) -> NetworkError {
    NetworkError::InvalidRequest(error.to_string())
}

use std::collections::HashMap;
use std::env;
use std::ffi::{CString, c_long};
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use crate::error::Error;
use crate::ffi::{
    CURLOPT_ECH, CURLOPT_FORM_BOUNDARY, CURLOPT_HTTP_VERSION, CURLOPT_HTTP2_NO_PRIORITY,
    CURLOPT_HTTP2_PSEUDO_HEADERS_ORDER, CURLOPT_HTTP2_SETTINGS, CURLOPT_HTTP2_WINDOW_UPDATE,
    CURLOPT_HTTPHEADER_ORDER, CURLOPT_SPLIT_COOKIES, CURLOPT_SSL_CERT_COMPRESSION,
    CURLOPT_SSL_CIPHER_LIST, CURLOPT_SSL_EC_CURVES, CURLOPT_SSL_ENABLE_ALPN,
    CURLOPT_SSL_ENABLE_ALPS, CURLOPT_SSL_ENABLE_TICKET, CURLOPT_SSL_PERMUTE_EXTENSIONS,
    CURLOPT_SSL_SIG_HASH_ALGS, CURLOPT_SSLVERSION, CURLOPT_STREAM_EXCLUSIVE, CURLOPT_STREAM_WEIGHT,
    CURLOPT_TLS_DELEGATED_CREDENTIALS, CURLOPT_TLS_EXTENSION_ORDER, CURLOPT_TLS_GREASE,
    CURLOPT_TLS_KEY_SHARES_LIMIT, CURLOPT_TLS_RECORD_SIZE_LIMIT,
    CURLOPT_TLS_SIGNED_CERT_TIMESTAMPS, CURLOPT_TLS_STATUS_REQUEST,
    CURLOPT_TLS_USE_NEW_ALPS_CODEPOINT, Curl, CurlOption, setopt,
};

const CURL_HTTP_VERSION_1_0: c_long = 1;
const CURL_HTTP_VERSION_1_1: c_long = 2;
const CURL_HTTP_VERSION_2_0: c_long = 3;
const CURL_HTTP_VERSION_3: c_long = 30;
const CURL_SSLVERSION_MAX_DEFAULT: c_long = 1 << 16;

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub(crate) struct Fingerprint {
    http_version: String,
    tls_version: String,
    tls_ciphers: Vec<String>,
    tls_alpn: bool,
    tls_alps: bool,
    tls_cert_compression: Vec<String>,
    tls_signature_hashes: Vec<String>,
    tls_key_shares_limit: c_long,
    tls_supported_groups: Vec<String>,
    tls_session_ticket: bool,
    tls_extension_order: String,
    tls_delegated_credentials: Vec<String>,
    tls_record_size_limit: Option<c_long>,
    tls_grease: bool,
    tls_use_new_alps_codepoint: bool,
    tls_ech: Option<String>,
    tls_permute_extensions: bool,
    headers: HashMap<String, String>,
    header_order: String,
    split_cookies: bool,
    form_boundary: String,
    http2_settings: String,
    http2_window_update: c_long,
    http2_pseudo_headers_order: String,
    http2_stream_weight: Option<c_long>,
    http2_stream_exclusive: Option<c_long>,
    http2_no_priority: bool,
}

impl Default for Fingerprint {
    fn default() -> Self {
        Self {
            http_version: "v2".to_string(),
            tls_version: "1.2".to_string(),
            tls_ciphers: Vec::new(),
            tls_alpn: false,
            tls_alps: false,
            tls_cert_compression: Vec::new(),
            tls_signature_hashes: Vec::new(),
            tls_key_shares_limit: 2,
            tls_supported_groups: Vec::new(),
            tls_session_ticket: false,
            tls_extension_order: String::new(),
            tls_delegated_credentials: Vec::new(),
            tls_record_size_limit: None,
            tls_grease: false,
            tls_use_new_alps_codepoint: false,
            tls_ech: None,
            tls_permute_extensions: false,
            headers: HashMap::new(),
            header_order: String::new(),
            split_cookies: false,
            form_boundary: String::new(),
            http2_settings: String::new(),
            http2_window_update: 0,
            http2_pseudo_headers_order: String::new(),
            http2_stream_weight: None,
            http2_stream_exclusive: None,
            http2_no_priority: false,
        }
    }
}

pub(crate) fn load_named(target: &str) -> Result<Option<Fingerprint>, Error> {
    let Some(path) = fingerprint_path() else {
        return Ok(None);
    };
    if !path.exists() {
        return Ok(None);
    }
    let payload = fs::read_to_string(&path).map_err(|error| {
        Error::Initialization(format!(
            "could not read fingerprint catalog {}: {error}",
            path.display()
        ))
    })?;
    let profiles: HashMap<String, Fingerprint> =
        serde_json::from_str(&payload).map_err(|error| {
            Error::Initialization(format!(
                "could not parse fingerprint catalog {}: {error}",
                path.display()
            ))
        })?;
    Ok(profiles.get(target).cloned())
}

fn fingerprint_path() -> Option<PathBuf> {
    if let Some(directory) = env::var_os("IMPERSONATE_CONFIG_DIR") {
        return Some(PathBuf::from(directory).join("fingerprints.json"));
    }
    if let Some(directory) = env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(directory).join("impersonate/fingerprints.json"));
    }
    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config/impersonate/fingerprints.json"))
}

pub(crate) fn apply(handle: *mut Curl, profile: &Fingerprint) -> Result<(), Error> {
    unsafe {
        setopt(
            handle,
            CURLOPT_HTTP_VERSION,
            http_version(&profile.http_version)?,
        )?;
        setopt(
            handle,
            CURLOPT_SSLVERSION,
            tls_version(&profile.tls_version)? | CURL_SSLVERSION_MAX_DEFAULT,
        )?;
        set_string_list(handle, CURLOPT_SSL_CIPHER_LIST, &profile.tls_ciphers, ":")?;

        if !profile.tls_extension_order.is_empty() && !profile.tls_permute_extensions {
            let order = profile
                .tls_extension_order
                .split('-')
                .filter(|extension| *extension != "21")
                .collect::<Vec<_>>()
                .join("-");
            set_string(handle, CURLOPT_TLS_EXTENSION_ORDER, &order)?;
        }

        setopt(
            handle,
            CURLOPT_SSL_ENABLE_ALPN,
            c_long::from(profile.tls_alpn),
        )?;
        setopt(
            handle,
            CURLOPT_SSL_ENABLE_ALPS,
            c_long::from(profile.tls_alps),
        )?;
        setopt(
            handle,
            CURLOPT_SSL_ENABLE_TICKET,
            c_long::from(profile.tls_session_ticket),
        )?;
        setopt(handle, CURLOPT_TLS_GREASE, c_long::from(profile.tls_grease))?;
        if profile.tls_use_new_alps_codepoint {
            setopt(handle, CURLOPT_SSL_ENABLE_ALPS, 1 as c_long)?;
            setopt(handle, CURLOPT_TLS_USE_NEW_ALPS_CODEPOINT, 1 as c_long)?;
        }
        setopt(handle, CURLOPT_TLS_STATUS_REQUEST, 1 as c_long)?;
        setopt(handle, CURLOPT_TLS_SIGNED_CERT_TIMESTAMPS, 1 as c_long)?;
        setopt(
            handle,
            CURLOPT_TLS_KEY_SHARES_LIMIT,
            profile.tls_key_shares_limit,
        )?;

        set_string(
            handle,
            CURLOPT_SSL_CERT_COMPRESSION,
            &profile.tls_cert_compression.join(","),
        )?;
        set_string_list(
            handle,
            CURLOPT_SSL_SIG_HASH_ALGS,
            &profile.tls_signature_hashes,
            ",",
        )?;
        let groups = profile
            .tls_supported_groups
            .iter()
            .map(|group| {
                if group == "X25519Kyber768" {
                    "X25519Kyber768Draft00"
                } else {
                    group.as_str()
                }
            })
            .collect::<Vec<_>>()
            .join(":");
        if !groups.is_empty() {
            set_string(handle, CURLOPT_SSL_EC_CURVES, &groups)?;
        }
        set_string_list(
            handle,
            CURLOPT_TLS_DELEGATED_CREDENTIALS,
            &profile.tls_delegated_credentials,
            ":",
        )?;
        setopt(
            handle,
            CURLOPT_SSL_PERMUTE_EXTENSIONS,
            c_long::from(profile.tls_permute_extensions),
        )?;
        if let Some(limit) = profile.tls_record_size_limit {
            setopt(handle, CURLOPT_TLS_RECORD_SIZE_LIMIT, limit)?;
        }
        if let Some(ech) = &profile.tls_ech {
            set_string(handle, CURLOPT_ECH, ech)?;
        }

        if !profile.http2_settings.is_empty() {
            set_string(handle, CURLOPT_HTTP2_SETTINGS, &profile.http2_settings)?;
        }
        if profile.http2_window_update != 0 {
            setopt(
                handle,
                CURLOPT_HTTP2_WINDOW_UPDATE,
                profile.http2_window_update,
            )?;
        }
        if !profile.http2_pseudo_headers_order.is_empty() {
            set_string(
                handle,
                CURLOPT_HTTP2_PSEUDO_HEADERS_ORDER,
                &profile.http2_pseudo_headers_order.replace(',', ""),
            )?;
        }
        if let Some(weight) = profile.http2_stream_weight {
            setopt(handle, CURLOPT_STREAM_WEIGHT, weight)?;
        }
        if let Some(exclusive) = profile.http2_stream_exclusive {
            setopt(handle, CURLOPT_STREAM_EXCLUSIVE, exclusive)?;
        }
        if profile.http2_no_priority {
            setopt(handle, CURLOPT_HTTP2_NO_PRIORITY, 1 as c_long)?;
        }
        if !profile.header_order.is_empty() {
            set_string(handle, CURLOPT_HTTPHEADER_ORDER, &profile.header_order)?;
        }
        setopt(
            handle,
            CURLOPT_SPLIT_COOKIES,
            c_long::from(profile.split_cookies),
        )?;
        if !profile.form_boundary.is_empty() {
            set_string(handle, CURLOPT_FORM_BOUNDARY, &profile.form_boundary)?;
        }
    }
    Ok(())
}

pub(crate) fn add_default_headers(
    profile: &Fingerprint,
    headers: &mut http::HeaderMap,
) -> Result<(), Error> {
    for (name, value) in &profile.headers {
        if name.eq_ignore_ascii_case("host") && value.is_empty() {
            continue;
        }
        if headers.contains_key(name) {
            continue;
        }
        let name = http::HeaderName::from_bytes(name.as_bytes()).map_err(|error| {
            Error::Initialization(format!("invalid fingerprint header name: {error}"))
        })?;
        let value = http::HeaderValue::from_str(value).map_err(|error| {
            Error::Initialization(format!("invalid fingerprint header value: {error}"))
        })?;
        headers.insert(name, value);
    }
    Ok(())
}

pub(crate) fn header_order(profile: &Fingerprint) -> Option<&str> {
    (!profile.header_order.is_empty()).then_some(&profile.header_order)
}

fn http_version(version: &str) -> Result<c_long, Error> {
    match version.trim().to_ascii_lowercase().as_str() {
        "v1" | "v1.0" | "1.0" => Ok(CURL_HTTP_VERSION_1_0),
        "v1.1" | "1.1" => Ok(CURL_HTTP_VERSION_1_1),
        "v2" | "h2" | "2" => Ok(CURL_HTTP_VERSION_2_0),
        "v3" | "h3" | "3" => Ok(CURL_HTTP_VERSION_3),
        value => Err(Error::Initialization(format!(
            "unsupported fingerprint HTTP version `{value}`"
        ))),
    }
}

fn tls_version(version: &str) -> Result<c_long, Error> {
    match version.trim().to_ascii_lowercase().as_str() {
        "1" | "tlsv1" => Ok(1),
        "1.0" | "tlsv1_0" => Ok(4),
        "1.1" | "tlsv1_1" => Ok(5),
        "1.2" | "tlsv1_2" => Ok(6),
        "1.3" | "tlsv1_3" => Ok(7),
        value => Err(Error::Initialization(format!(
            "unsupported fingerprint TLS version `{value}`"
        ))),
    }
}

unsafe fn set_string(handle: *mut Curl, option: CurlOption, value: &str) -> Result<(), Error> {
    let value = CString::new(value)
        .map_err(|error| Error::Initialization(format!("invalid fingerprint value: {error}")))?;
    unsafe { setopt(handle, option, value.as_ptr()) }
}

unsafe fn set_string_list(
    handle: *mut Curl,
    option: CurlOption,
    values: &[String],
    separator: &str,
) -> Result<(), Error> {
    if values.is_empty() {
        return Ok(());
    }
    unsafe { set_string(handle, option, &values.join(separator)) }
}

#[cfg(test)]
mod tests {
    use super::{Fingerprint, http_version, tls_version};

    #[test]
    fn parses_firefox_catalog_fields_and_ignores_metadata() {
        let profile: Fingerprint = serde_json::from_str(
            r#"{
                "client": "firefox",
                "http_version": "v2",
                "tls_version": "1.2",
                "tls_ciphers": ["TLS_AES_128_GCM_SHA256"],
                "tls_key_shares_limit": 3,
                "header_order": "upgrade-insecure-requests,user-agent"
            }"#,
        )
        .unwrap();

        assert_eq!(profile.tls_ciphers, ["TLS_AES_128_GCM_SHA256"]);
        assert_eq!(profile.tls_key_shares_limit, 3);
        assert_eq!(profile.header_order, "upgrade-insecure-requests,user-agent");
    }

    #[test]
    fn maps_catalog_protocol_versions_to_curl_values() {
        assert_eq!(http_version("v2").unwrap(), 3);
        assert_eq!(tls_version("1.2").unwrap(), 6);
        assert!(tls_version("ssl3").is_err());
    }
}

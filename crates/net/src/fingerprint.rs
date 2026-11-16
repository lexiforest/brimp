use std::collections::HashMap;
use std::ffi::{CString, c_long};

use serde::Deserialize;

use crate::error::Error;
use crate::ffi::*;

const CURL_HTTP_VERSION_1_0: c_long = 1;
const CURL_HTTP_VERSION_1_1: c_long = 2;
const CURL_HTTP_VERSION_2_0: c_long = 3;
const CURL_HTTP_VERSION_3: c_long = 30;
const CURL_SSLVERSION_MAX_DEFAULT: c_long = 1 << 16;

/// Editable curl-impersonate transport fingerprint, matching curl_cffi's schema.
#[derive(Clone, Debug, serde::Serialize, Deserialize)]
#[serde(default)]
pub struct Fingerprint {
    /// Client.
    pub client: String,
    /// Client version.
    pub client_version: String,
    /// Os.
    pub os: String,
    /// Os version.
    pub os_version: String,
    /// Http version.
    pub http_version: String,
    /// Tls version.
    pub tls_version: String,
    /// Tls ciphers.
    pub tls_ciphers: Vec<String>,
    /// Tls alpn.
    pub tls_alpn: bool,
    /// Tls alps.
    pub tls_alps: bool,
    /// Tls cert compression.
    pub tls_cert_compression: Vec<String>,
    /// Tls signature hashes.
    pub tls_signature_hashes: Vec<String>,
    /// Tls key shares limit.
    pub tls_key_shares_limit: c_long,
    /// Tls supported groups.
    pub tls_supported_groups: Vec<String>,
    /// Tls session ticket.
    pub tls_session_ticket: bool,
    /// Tls extension order.
    pub tls_extension_order: String,
    /// Tls delegated credentials.
    pub tls_delegated_credentials: Vec<String>,
    /// Tls record size limit.
    pub tls_record_size_limit: Option<c_long>,
    /// Tls grease.
    pub tls_grease: bool,
    /// Tls use new alps codepoint.
    pub tls_use_new_alps_codepoint: bool,
    /// Tls signed cert timestamps.
    pub tls_signed_cert_timestamps: bool,
    /// Tls ech.
    pub tls_ech: Option<String>,
    /// Tls permute extensions.
    pub tls_permute_extensions: bool,
    /// Tls trust anchors.
    pub tls_trust_anchors: Option<Vec<String>>,
    /// Headers.
    pub headers: HashMap<String, String>,
    /// Header order.
    pub header_order: String,
    /// Split cookies.
    pub split_cookies: bool,
    /// Form boundary.
    pub form_boundary: String,
    /// Http2 settings.
    pub http2_settings: String,
    /// Http2 window update.
    pub http2_window_update: c_long,
    /// Http2 pseudo headers order.
    pub http2_pseudo_headers_order: String,
    /// Http2 stream weight.
    pub http2_stream_weight: Option<c_long>,
    /// Http2 stream exclusive.
    pub http2_stream_exclusive: Option<c_long>,
    /// Http2 no priority.
    pub http2_no_priority: bool,
    /// Http3 settings.
    pub http3_settings: String,
    /// Http3 pseudo headers order.
    pub http3_pseudo_headers_order: String,
    /// Http3 tls extension order.
    pub http3_tls_extension_order: String,
    /// Http3 headers.
    pub http3_headers: HashMap<String, String>,
    /// Http3 header order.
    pub http3_header_order: String,
    /// Http3 tls supported groups.
    pub http3_tls_supported_groups: Vec<String>,
    /// Quic transport parameters.
    pub quic_transport_parameters: String,
    /// Quic cid length.
    pub quic_cid_length: Option<String>,
    /// Quic initial packet number.
    pub quic_initial_packet_number: Option<c_long>,
    /// Ws headers.
    pub ws_headers: HashMap<String, String>,
    /// Ws header order.
    pub ws_header_order: String,
    /// Ws disable session ticket.
    pub ws_disable_session_ticket: bool,
    /// Ws tls cert compression.
    pub ws_tls_cert_compression: Option<Vec<String>>,
    /// Header lang.
    pub header_lang: String,
}

impl Default for Fingerprint {
    fn default() -> Self {
        Self {
            client: "".to_owned(),
            client_version: "".to_owned(),
            os: "".to_owned(),
            os_version: "".to_owned(),
            http_version: "v2".to_owned(),
            tls_version: "1.2".to_owned(),
            tls_ciphers: Default::default(),
            tls_alpn: true,
            tls_alps: false,
            tls_cert_compression: Default::default(),
            tls_signature_hashes: Default::default(),
            tls_key_shares_limit: 2,
            tls_supported_groups: Default::default(),
            tls_session_ticket: false,
            tls_extension_order: "".to_owned(),
            tls_delegated_credentials: Default::default(),
            tls_record_size_limit: None,
            tls_grease: false,
            tls_use_new_alps_codepoint: false,
            tls_signed_cert_timestamps: false,
            tls_ech: None,
            tls_permute_extensions: false,
            tls_trust_anchors: None,
            headers: Default::default(),
            header_order: "".to_owned(),
            split_cookies: false,
            form_boundary: "".to_owned(),
            http2_settings: "".to_owned(),
            http2_window_update: 0,
            http2_pseudo_headers_order: "".to_owned(),
            http2_stream_weight: None,
            http2_stream_exclusive: None,
            http2_no_priority: false,
            http3_settings: "".to_owned(),
            http3_pseudo_headers_order: "".to_owned(),
            http3_tls_extension_order: "".to_owned(),
            http3_headers: Default::default(),
            http3_header_order: "".to_owned(),
            http3_tls_supported_groups: Default::default(),
            quic_transport_parameters: "".to_owned(),
            quic_cid_length: None,
            quic_initial_packet_number: None,
            ws_headers: Default::default(),
            ws_header_order: "".to_owned(),
            ws_disable_session_ticket: false,
            ws_tls_cert_compression: None,
            header_lang: "".to_owned(),
        }
    }
}

pub(crate) fn apply(
    handle: *mut Curl,
    profile: &Fingerprint,
    default_headers: bool,
    supplied_headers: &http::HeaderMap,
) -> Result<HeaderLists, Error> {
    let mut lists = HeaderLists::default();
    unsafe {
        setopt(
            handle,
            CURLOPT_HTTP_VERSION,
            http_version(&profile.http_version)?,
        )?;
        if !profile.tls_version.is_empty() {
            setopt(
                handle,
                CURLOPT_SSLVERSION,
                tls_version(&profile.tls_version)? | CURL_SSLVERSION_MAX_DEFAULT,
            )?;
        }
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
        setopt(
            handle,
            CURLOPT_TLS_SIGNED_CERT_TIMESTAMPS,
            c_long::from(profile.tls_signed_cert_timestamps),
        )?;
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
    unsafe {
        if let Some(anchors) = &profile.tls_trust_anchors {
            set_string(handle, CURLOPT_TLS_TRUST_ANCHORS, &anchors.join(","))?;
        }
        for (option, value) in [
            (CURLOPT_HTTP3_SETTINGS, profile.http3_settings.as_str()),
            (
                CURLOPT_HTTP3_PSEUDO_HEADERS_ORDER,
                &profile.http3_pseudo_headers_order.replace(',', ""),
            ),
            (
                CURLOPT_HTTP3_TLS_EXTENSION_ORDER,
                &profile.http3_tls_extension_order,
            ),
            (CURLOPT_HTTP3_HTTPHEADER_ORDER, &profile.http3_header_order),
            (
                CURLOPT_QUIC_TRANSPORT_PARAMETERS,
                &profile.quic_transport_parameters,
            ),
            (CURLOPT_WS_HTTPHEADER_ORDER, &profile.ws_header_order),
        ] {
            if !value.is_empty() {
                set_string(handle, option, value)?;
            }
        }
        let groups = profile
            .http3_tls_supported_groups
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
            set_string(handle, CURLOPT_HTTP3_SSL_EC_CURVES, &groups)?;
        }
        if let Some(length) = &profile.quic_cid_length {
            set_string(handle, CURLOPT_QUIC_CID_LENGTH, length)?;
        }
        if let Some(number) = profile.quic_initial_packet_number {
            setopt(handle, CURLOPT_QUIC_INITIAL_PACKET_NUMBER, number)?;
        }
        if profile.ws_disable_session_ticket {
            setopt(handle, CURLOPT_WS_SSL_DISABLE_TICKET, 1 as c_long)?;
        }
        if let Some(compression) = &profile.ws_tls_cert_compression {
            set_string(
                handle,
                CURLOPT_WS_SSL_CERT_COMPRESSION,
                &compression.join(","),
            )?;
        }
        if default_headers {
            for (option, headers) in [
                (CURLOPT_HTTP3_HTTPHEADER, &profile.http3_headers),
                (CURLOPT_WS_HTTPHEADER, &profile.ws_headers),
            ] {
                if headers.is_empty() {
                    continue;
                }
                let mut merged: HashMap<String, String> = profile
                    .headers
                    .iter()
                    .chain(headers.iter())
                    .map(|(name, value)| (name.to_ascii_lowercase(), value.clone()))
                    .collect();
                for (name, value) in supplied_headers {
                    merged.insert(
                        name.as_str().to_owned(),
                        value
                            .to_str()
                            .map_err(|error| Error::impersonation(error.to_string()))?
                            .to_owned(),
                    );
                }
                lists.set(handle, option, &merged)?;
            }
        }
    }

    Ok(lists)
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
            Error::impersonation(format!("invalid fingerprint header name: {error}"))
        })?;
        let value = http::HeaderValue::from_str(value).map_err(|error| {
            Error::impersonation(format!("invalid fingerprint header value: {error}"))
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
        "v2" | "v2tls" | "h2" | "2" => Ok(CURL_HTTP_VERSION_2_0),
        "v2_prior_knowledge" => Ok(5),
        "v3" | "h3" | "3" => Ok(CURL_HTTP_VERSION_3),
        "v3only" => Ok(31),
        value => Err(Error::impersonation(format!(
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
        value => Err(Error::impersonation(format!(
            "unsupported fingerprint TLS version `{value}`"
        ))),
    }
}

unsafe fn set_string(handle: *mut Curl, option: CurlOption, value: &str) -> Result<(), Error> {
    let value = CString::new(value)
        .map_err(|error| Error::impersonation(format!("invalid fingerprint value: {error}")))?;
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

#[derive(Default)]
pub(crate) struct HeaderLists(Vec<*mut CurlSlist>);
impl HeaderLists {
    unsafe fn set(
        &mut self,
        handle: *mut Curl,
        option: CurlOption,
        headers: &HashMap<String, String>,
    ) -> Result<(), Error> {
        if headers.is_empty() {
            return Ok(());
        }
        self.0.push(std::ptr::null_mut());
        let list = self.0.last_mut().unwrap();
        for (name, value) in headers {
            if name.eq_ignore_ascii_case("host") && value.is_empty() {
                continue;
            }
            http::HeaderName::from_bytes(name.as_bytes())
                .map_err(|error| Error::impersonation(error.to_string()))?;
            http::HeaderValue::from_str(value)
                .map_err(|error| Error::impersonation(error.to_string()))?;
            let line = CString::new(format!("{name}: {value}"))
                .map_err(|error| Error::impersonation(error.to_string()))?;
            let next = unsafe { curl_slist_append(*list, line.as_ptr()) };
            if next.is_null() {
                return Err(Error::Initialization(
                    "could not allocate fingerprint headers".into(),
                ));
            }
            *list = next;
        }
        unsafe { setopt(handle, option, *list) }
    }
}
impl Drop for HeaderLists {
    fn drop(&mut self) {
        for list in self.0.drain(..) {
            unsafe {
                curl_slist_free_all(list);
            }
        }
    }
}

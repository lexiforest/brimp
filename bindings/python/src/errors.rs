//! Python owns the public hierarchy; native failures preserve structured details.
use brimp_runtime::{network::NetworkError, runtime::AutomationError};
use pyo3::prelude::*;

pyo3::import_exception!(brimp.exceptions, RuntimeError);
pyo3::import_exception!(brimp.exceptions, InvalidInput);
pyo3::import_exception!(brimp.exceptions, JavaScriptError);
pyo3::import_exception!(brimp.exceptions, Timeout);
pyo3::import_exception!(brimp.exceptions, Closed);
pyo3::import_exception!(brimp.exceptions, Reentry);
pyo3::import_exception!(brimp.exceptions, StaleHandle);
pyo3::import_exception!(brimp.exceptions, TransportError);
pyo3::import_exception!(brimp.exceptions, ConnectionError);
pyo3::import_exception!(brimp.exceptions, DNSError);
pyo3::import_exception!(brimp.exceptions, ProxyError);
pyo3::import_exception!(brimp.exceptions, SSLError);
pyo3::import_exception!(brimp.exceptions, CertificateVerifyError);
pyo3::import_exception!(brimp.exceptions, RequestTimeout);
pyo3::import_exception!(brimp.exceptions, HTTPError);
pyo3::import_exception!(brimp.exceptions, IncompleteRead);
pyo3::import_exception!(brimp.exceptions, TooManyRedirects);
pyo3::import_exception!(brimp.exceptions, SessionClosed);
pyo3::import_exception!(brimp.exceptions, ImpersonateError);
pyo3::import_exception!(brimp.exceptions, InvalidURL);
pyo3::import_exception!(brimp.exceptions, InvalidSchema);
pyo3::import_exception!(brimp.exceptions, InvalidProxyURL);
pyo3::import_exception!(brimp.exceptions, InvalidHeader);
pyo3::import_exception!(brimp.exceptions, InterfaceError);

pub(super) fn error(error: AutomationError) -> PyErr {
    let message = error.to_string();
    match error {
        AutomationError::Network(error) => network_error(error),
        AutomationError::InvalidInput(_) => InvalidInput::new_err(message),
        AutomationError::JavaScript(_) => JavaScriptError::new_err(message),
        AutomationError::Timeout(_) => Timeout::new_err(message),
        AutomationError::Closed => Closed::new_err(message),
        AutomationError::Reentry => Reentry::new_err(message),
        AutomationError::StaleHandle => StaleHandle::new_err(message),
        AutomationError::Transport(_) => TransportError::new_err(message),
        AutomationError::HttpStatus(_) => HTTPError::new_err(message),
        _ => RuntimeError::new_err(message),
    }
}

pub(super) fn session_error(value: AutomationError) -> PyErr {
    match value {
        AutomationError::Closed => SessionClosed::new_err("session is closed"),
        other => error(other),
    }
}

pub(super) fn network_error(error: NetworkError) -> PyErr {
    let message = error.to_string();
    match error {
        NetworkError::Transfer {
            code,
            proxy_connect,
            certificate_verify,
            ..
        } => {
            let args = (message, code);
            if certificate_verify {
                return CertificateVerifyError::new_err((
                    format!("TLS certificate verification failed: {}", args.0),
                    args.1,
                ));
            }
            if proxy_connect {
                return ProxyError::new_err(args);
            }
            // CURLcode values from curl.h, matching the supported requests failures.
            match code {
                1 => InvalidSchema::new_err(args),
                3 => InvalidURL::new_err(args),
                5 | 97 => ProxyError::new_err(args),
                6 => DNSError::new_err(args),
                7 | 8 | 9 | 52 | 55 | 56 | 96 => ConnectionError::new_err(args),
                16 | 22 | 61 | 92 | 95 => HTTPError::new_err(args),
                18 => IncompleteRead::new_err(args),
                28 => RequestTimeout::new_err(args),
                35 | 53 | 54 | 58 | 59 | 66 | 77 | 82 | 83 | 90 | 91 | 98 | 101 => {
                    SSLError::new_err(args)
                }
                60 => CertificateVerifyError::new_err(args),
                45 => InterfaceError::new_err(args),
                47 => TooManyRedirects::new_err(args),
                _ => TransportError::new_err(args),
            }
        }
        NetworkError::Impersonation { code, .. } => ImpersonateError::new_err((message, code)),
        NetworkError::InvalidSchema(_) => InvalidSchema::new_err(message),
        NetworkError::InvalidUrl(_) => InvalidURL::new_err(message),
        NetworkError::InvalidProxyUrl(_) => InvalidProxyURL::new_err(message),
        NetworkError::InvalidHeader(_) => InvalidHeader::new_err(message),
        NetworkError::TooManyRedirects(_) => TooManyRedirects::new_err(message),
        NetworkError::Timeout => RequestTimeout::new_err(message),
        NetworkError::InvalidRequest(_) => InvalidInput::new_err(message),
        other => TransportError::new_err(other.to_string()),
    }
}

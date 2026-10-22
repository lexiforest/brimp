use http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header};

use crate::error::Error;

/// Parses a curl status line and ignores non-status header lines.
pub(crate) fn parse_status_line(line: &str) -> Result<Option<StatusCode>, Error> {
    if !line.starts_with("HTTP/") {
        return Ok(None);
    }
    let Some(status) = line.split_whitespace().nth(1) else {
        return Err(Error::Response(format!("invalid curl status line: {line}")));
    };
    status
        .parse::<u16>()
        .map_err(|error| Error::Response(format!("invalid status code: {error}")))
        .and_then(|status| {
            StatusCode::from_u16(status).map_err(|error| Error::Response(error.to_string()))
        })
        .map(Some)
}

/// Removes headers that no longer match libcurl's decoded response body.
pub(crate) fn normalize_decoded_response_headers(mut headers: HeaderMap) -> HeaderMap {
    headers.remove(header::CONTENT_ENCODING);
    headers.remove(header::CONTENT_LENGTH);
    headers.remove(header::TRANSFER_ENCODING);
    headers
}

/// Extracts the last complete non-informational response header block.
pub(crate) fn final_header_block(lines: &[String]) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let block = lines
        .split(|line| line.is_empty())
        .rev()
        .find(|block| block.iter().any(|line| line.starts_with("HTTP/")))
        .unwrap_or_default();
    for line in block {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let Ok(name) = HeaderName::from_bytes(name.trim().as_bytes()) else {
            continue;
        };
        let Ok(value) = HeaderValue::from_str(value.trim()) else {
            continue;
        };
        headers.append(name, value);
    }
    headers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_status_line_accepts_http_versions() {
        assert_eq!(
            parse_status_line("HTTP/2 204").unwrap(),
            Some(StatusCode::NO_CONTENT)
        );
        assert_eq!(parse_status_line("content-type: text/html").unwrap(), None);
    }

    #[test]
    fn final_header_block_uses_last_response_headers() {
        let headers = final_header_block(&[
            "HTTP/1.1 100 Continue".to_string(),
            "".to_string(),
            "HTTP/2 200".to_string(),
            "content-type: text/plain".to_string(),
            "set-cookie: a=b".to_string(),
            "".to_string(),
        ]);
        assert_eq!(
            headers.get("content-type").unwrap(),
            HeaderValue::from_static("text/plain")
        );
        assert_eq!(
            headers.get("set-cookie").unwrap(),
            HeaderValue::from_static("a=b")
        );
    }

    #[test]
    fn normalize_decoded_headers_removes_invalidated_metadata() {
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_ENCODING, HeaderValue::from_static("br"));
        headers.insert(header::CONTENT_LENGTH, HeaderValue::from_static("12"));
        headers.insert(
            header::TRANSFER_ENCODING,
            HeaderValue::from_static("chunked"),
        );
        headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/plain"));

        let headers = normalize_decoded_response_headers(headers);

        assert!(!headers.contains_key(header::CONTENT_ENCODING));
        assert!(!headers.contains_key(header::CONTENT_LENGTH));
        assert!(!headers.contains_key(header::TRANSFER_ENCODING));
        assert_eq!(
            headers.get(header::CONTENT_TYPE).unwrap(),
            HeaderValue::from_static("text/plain")
        );
    }
}
